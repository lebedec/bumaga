use crate::css::{match_style, read_css, read_inline_css, Css, PseudoClassMatcher};
use crate::html::{read_html, ElementBinding, Html};
use crate::metrics::ViewMetrics;
use crate::rendering::{Renderer, RendererTranslator};
use crate::styles::{inherit, Cascade, Scrolling, Sizes, Variables};
use crate::tree::ViewTreeExtensions;
use crate::view_model::{Reaction, ViewModel};
use crate::{BindingParams, Element, ElementStyle, Fonts, Input, Output, Transformer, ViewError};
use log::error;
use mesura::GaugeValue;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::{Add, Deref};
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{AvailableSpace, Layout, NodeId, Point, PrintTree, Size, TaffyTree};

pub struct View {
    model: ViewModel,
    pub(crate) tree: TaffyTree<Element>,
    root: NodeId,
    body: NodeId,
    css: Css,
    resources: String,
    pub fonts: Box<dyn Fonts>,
    metrics: ViewMetrics,
    identified: HashMap<String, NodeId>,
}

impl View {
    pub fn from_html(
        html: &str,
        css: &str,
        resources: &str,
        fonts: impl Fonts + 'static,
        translator: Box<dyn RendererTranslator>,
    ) -> Result<Self, ViewError> {
        let html = read_html(html)?;
        let css = read_css(css)?;

        let mut body = Html::empty();
        let mut templates = HashMap::new();
        for child in html.children {
            if child.tag == "link" {
                let mut attrs = HashMap::new();
                for binding in &child.bindings {
                    if let ElementBinding::None(key, value) = binding {
                        attrs.insert(key.clone(), value.as_str());
                    }
                }
            }
            if child.tag == "template" {
                let mut id = None;
                for binding in &child.bindings {
                    if let ElementBinding::None(key, value) = binding {
                        if key == "id" {
                            id = Some(value.clone());
                        }
                    }
                }
                if let Some(id) = id {
                    if child.children.len() == 1 {
                        templates.insert(format!("#{id}"), child.children[0].clone());
                    }
                }
                continue;
            }
            if child.tag == "body" {
                body = child;
                break;
            }
        }

        let mut renderer = Renderer::new(templates, translator);
        let [root, body] = renderer.render(body)?;
        let bindings = renderer.bindings;
        let schema = renderer.schema;
        let tree = renderer.tree;
        let identified = renderer.static_id;
        let model = ViewModel::create(bindings, schema.value);
        let mut view = Self {
            model,
            tree,
            root,
            body,
            css,
            resources: resources.to_string(),
            fonts: Box::new(fonts),
            metrics: ViewMetrics::new(),
            identified,
        };
        view.calculate_elements_stylesheet(body)?;
        view.apply_default_bindings_state()?;
        Ok(view)
    }

    pub fn fonts(mut self, fonts: impl Fonts + 'static) -> Self {
        self.fonts = Box::new(fonts);
        self
    }

    pub fn pipe(mut self, name: &str, transformer: Transformer) -> Self {
        self.model
            .transformers
            .insert(name.to_string(), transformer);
        self
    }

    pub fn update(&mut self, input: Input, value: Value) -> Result<Output, ViewError> {
        self.metrics.updates.inc();
        let reactions = self.model.bind(&value);
        for reaction in reactions {
            self.update_tree(reaction)?;
        }
        // detect viewport changes
        let [viewport_width, viewport_height] = input.viewport;
        let mut root_layout = self.tree.style(self.root).unwrap().clone();
        if root_layout.size.width != length(viewport_width)
            && root_layout.size.height != length(viewport_height)
        {
            root_layout.size = Size {
                width: length(viewport_width),
                height: length(viewport_height),
            };
            self.tree.set_style(self.root, root_layout)?;
        }
        let sizes = Sizes {
            root_font_size: 16.0,
            parent_font_size: 16.0,
            parent_color: [0; 4],
            viewport_width,
            viewport_height,
        };
        self.apply_styles(self.body, &input, sizes, Variables::default())?;
        self.tree.compute_layout_with_measure(
            self.body,
            Size::MAX_CONTENT,
            |size, space, _, view, _| measure_text(self.fonts.as_ref(), size, space, view),
        )?;
        // TODO: clipping of viewport
        self.compute_final_positions_and_clipping(self.body, Point::ZERO, 1.0, None)?;
        self.model.handle_output(&input, self.body, &mut self.tree)
    }

    fn compute_final_positions_and_clipping(
        &mut self,
        node: NodeId,
        location: Point<f32>,
        mut opacity: f32,
        mut clipping: Option<Layout>,
    ) -> Result<(), ViewError> {
        self.metrics.elements_shown.inc();
        let mut layout = self.tree.get_final_layout(node).clone();
        layout.location = layout.location.add(location);
        let element = self.tree.get_element_mut(node)?;
        element.opacity = opacity * element.self_opacity;
        element.position = [layout.location.x, layout.location.y];
        element.size = [layout.size.width, layout.size.height];
        element.content_size = [layout.content_size.width, layout.content_size.height];
        element.scrolling = Scrolling::ensure(&layout, &element.scrolling);
        element.clipping = clipping;
        let mut location = layout.location;
        if let Some(scrolling) = element.scrolling.as_ref() {
            clipping = Some(layout.clone());
            location.x -= scrolling.x;
            location.y -= scrolling.y;
        }
        opacity = element.opacity;
        for child in self.tree.children(node)? {
            self.compute_final_positions_and_clipping(child, location, opacity, clipping)?;
        }
        Ok(())
    }

    fn update_tree(&mut self, reaction: Reaction) -> Result<(), ViewError> {
        match reaction {
            Reaction::Type { node, span, text } => {
                let element_text = self
                    .tree
                    .get_node_context_mut(node)
                    .and_then(|element| element.text.as_mut())
                    .ok_or(ViewError::ElementTextContentNotFound)?;
                element_text.set(span, text);
                self.tree.mark_dirty(node)?;
            }
            Reaction::Reattach {
                parent,
                node,
                visible,
            } => {
                let definition = self
                    .tree
                    .get_element(parent)
                    .map(|parent| parent.children.clone())?;
                let definition_index = definition
                    .iter()
                    .position(|child| child == &node)
                    .ok_or(ViewError::ChildNotFound(node))?;
                let current = self.tree.children(parent)?;
                let current_index = current.iter().position(|child| child == &node);
                if visible {
                    if current_index.is_some() {
                        // nothing to do, already visible
                    } else {
                        let mut index = current.len().min(definition_index);
                        while index > 0 {
                            index -= 1;
                            let sibling = current[index];
                            let sibling_index = definition
                                .iter()
                                .position(|child| child == &sibling)
                                .ok_or(ViewError::ChildNotFound(sibling))?;
                            if sibling_index < definition_index {
                                index += 1;
                                break;
                            }
                        }
                        self.tree.insert_child_at_index(parent, index, node)?;
                    }
                } else {
                    if let Some(current_index) = current_index {
                        self.tree.remove_child_at_index(parent, current_index)?;
                    } else {
                        // nothing to do, already hidden
                    }
                }
            }
            Reaction::Repeat {
                parent,
                start,
                cursor,
                end,
            } => {
                let children = self
                    .tree
                    .get_element_mut(parent)
                    .map(|parent| parent.children.clone())?;
                let visible = self.tree.children(parent)?;
                let shown = &children[start..cursor];
                let hidden = &children[cursor..end];
                for node in shown {
                    if !visible.contains(node) {
                        self.tree.add_child(parent, *node)?;
                    }
                }
                for node in hidden {
                    if visible.contains(node) {
                        self.tree.remove_child(parent, *node)?;
                    }
                }
            }
            Reaction::Tag { node, key, tag } => {
                let element = self.tree.get_element_mut(node)?;
                if tag {
                    element.attrs.insert(key.clone(), key.clone());
                } else {
                    element.attrs.remove(&key);
                };
            }
            Reaction::Bind {
                node,
                key,
                span,
                text,
            } => {
                let element = self.tree.get_element_mut(node)?;
                let attribute = element
                    .attrs_bindings
                    .get_mut(&key)
                    .ok_or(ViewError::AttributeBindingNotFound(key.clone()))?;
                attribute.set(span, text);
                let value = attribute.to_string();
                element.attrs.insert(key.clone(), value.clone());
                if key == "style" {
                    match read_inline_css(&value) {
                        Ok(style) => element.style = style,
                        Err(error) => {
                            error!("unable to parse styles of {}, {error:?}", element.tag);
                        }
                    }
                }
                if key == "id" {
                    self.identified.insert(value.clone(), node);
                }
                match (element.tag.as_str(), key.as_str()) {
                    ("img", "src") => self.model.update_img_src(node, value, &mut self.tree)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    // Why this function is needed?
    // We need all elements to add in layout tree via rendering function.
    // Apply reaction after rendering closest way to perform "clean" binding with default values.
    // But this can degrade performance of View creation.
    fn apply_default_bindings_state(&mut self) -> Result<(), ViewError> {
        let mut reactions = vec![];
        for bindings in self.model.bindings.values() {
            for binding in bindings {
                match binding.params {
                    BindingParams::Visibility(parent, node, _) => {
                        reactions.push(Reaction::Reattach {
                            parent,
                            node,
                            visible: false,
                        })
                    }
                    _ => {}
                }
            }
        }
        for reaction in reactions {
            self.update_tree(reaction)?;
        }
        Ok(())
    }

    fn calculate_elements_stylesheet(&mut self, node: NodeId) -> Result<(), ViewError> {
        struct Matcher;
        impl PseudoClassMatcher for Matcher {
            fn has_pseudo_class(&self, _element: &Element, _class: &str) -> bool {
                true
            }
        }
        for style in self.css.styles.iter() {
            let matches_ignoring_pseudo = match_style(style, node, &self.tree, &Matcher);
            let element = self.tree.get_element_mut(node)?;
            let hints = &element.style_hints;
            let has_pseudo = style.has_pseudo_class_selector();
            let is_static = !hints.has_dynamic_properties()
                || (!style.has_attrs_selector(&hints.dynamic_attrs)
                    && (!hints.has_dynamic_classes || !style.has_class_selector())
                    && (!hints.has_dynamic_id || !style.has_id_selector()));
            if matches_ignoring_pseudo {
                if is_static && !has_pseudo {
                    element.styles.push(ElementStyle::Static(style.clone()));
                } else {
                    element.styles.push(ElementStyle::Dynamic(style.clone()));
                }
            } else {
                if is_static {
                    // discard, we do not handle styles that will never be applied
                } else {
                    element.styles.push(ElementStyle::Dynamic(style.clone()));
                }
            }
        }
        let children = self.tree.children(node)?;
        for child in children {
            self.calculate_elements_stylesheet(child)?;
        }
        Ok(())
    }

    fn apply_styles(
        &mut self,
        node: NodeId,
        input: &Input,
        mut sizes: Sizes,
        variables: Variables,
    ) -> Result<(), ViewError> {
        let parent = unsafe {
            let ptr = self
                .tree
                .parent(node)
                .and_then(|parent| self.tree.get_node_context(parent))
                .ok_or(ViewError::ParentNotFound(node))? as *const Element;
            // TODO:
            &*ptr
        };
        let mut layout = self.tree.style(node)?.clone();
        let element = unsafe {
            let ptr = self.tree.get_element_mut(node)? as *mut Element;
            &mut *ptr
        };

        if element.text.is_some() {
            inherit(parent, element);
            return Ok(());
        }

        self.metrics.cascades.inc();
        let mut cascade = Cascade::new(&self.css, sizes, variables);
        cascade.apply_styles(input, node, &self.tree, parent, &mut layout, element, self);
        let stats = cascade.stats;
        self.metrics.styles.set(self.css.styles.len());
        let cascade_metrics = &mut self.metrics.cascade;
        cascade_metrics.matches_static.add(stats.matches_static);
        cascade_metrics.matches_dynamic.add(stats.matches_dynamic);
        cascade_metrics.apply_ok.add(stats.apply_ok);
        cascade_metrics.apply_error.add(stats.apply_error);
        let variables = cascade.take_variables();

        // we must update styles only if changes detected to support Taffy cache system
        if self.tree.style(node)? != &layout {
            self.metrics.layouts.inc();
            self.tree.set_style(node, layout)?;
        }

        // self.tree.set_node_context(node, Some(element));

        match element.tag.as_str() {
            "img" => {
                // self.render_img(current_id, &element, tree);
            }
            "input" => {
                // let text = element.html.attrs.get("value").cloned().unwrap_or_default();
                // self.render_input(text, current_id, &element, tree);
            }
            "area" => {}
            "base" => {}
            "br" => {}
            "col" => {}
            "command" => {}
            "embed" => {}
            "hr" => {}
            "keygen" => {}
            "link" => {}
            "meta" => {}
            "param" => {}
            "source" => {}
            "track" => {}
            "wbr" => {}
            _ => {
                let children = self.tree.children(node)?;
                for child in children {
                    sizes.parent_font_size = element.font.size;
                    sizes.parent_color = element.color;
                    self.apply_styles(child, input, sizes, variables.clone())?;
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn get_element_by_id(&self, id: &str) -> Option<&Element> {
        self.identified
            .get(id)
            .and_then(|node| self.tree.get_element(*node).ok())
    }

    pub fn body(&self) -> Fragment {
        let element = self
            .tree
            .get_node_context(self.body)
            .expect("body must be configured");
        Fragment {
            element,
            tree: &self.tree,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Fragment<'t> {
    pub element: &'t Element,
    pub tree: &'t TaffyTree<Element>,
}

impl Fragment<'_> {
    pub fn children(&self) -> Vec<Fragment> {
        match self.tree.children(self.element.node) {
            Ok(children) => children
                .iter()
                .map(|node| {
                    let element = self.tree.get_node_context(*node).unwrap();
                    Fragment {
                        element,
                        tree: self.tree,
                    }
                })
                .collect(),
            Err(error) => {
                error!("unable to traverse fragment, {error:?}");
                vec![]
            }
        }
    }
}

impl Deref for Fragment<'_> {
    type Target = Element;

    fn deref(&self) -> &Self::Target {
        self.element
    }
}

fn measure_text<F: Fonts + ?Sized>(
    fonts: &F,
    size: Size<Option<f32>>,
    space: Size<AvailableSpace>,
    element: Option<&mut Element>,
) -> Size<f32> {
    if let Size {
        width: Some(width),
        height: Some(height),
    } = size
    {
        return Size { width, height };
    }
    let element = match element {
        None => return Size::ZERO,
        Some(element) => element,
    };
    if let Some(text) = element.text.as_ref().map(|text| text.to_string()) {
        let max_width = size.width.map(Some).unwrap_or_else(|| match space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width),
        });
        let [width, height] = fonts.measure(&text, &element.font, max_width);
        return Size { width, height };
    }
    Size::ZERO
}

impl PseudoClassMatcher for View {
    fn has_pseudo_class(&self, element: &Element, class: &str) -> bool {
        match class {
            "hover" => element.state.hover,
            "active" => element.state.active,
            // The :checked CSS pseudo-class represents any radio, checkbox, or option element
            // that is checked or toggled to an "on" state.
            "checked" => element.state.checked,
            // The :focus CSS pseudo-class represents an element (such as a form input) that
            // has received focus. It is generally triggered when the user clicks or taps
            // on an element or selects it with the keyboard's Tab key.
            "focus" => element.state.focus,
            // The :blank CSS pseudo-class selects empty user input elements.
            "blank" => false,
            _ => {
                error!("unable to match unknown pseudo class {class}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::FakeTranslator;
    use crate::testing::setup_tests_logging;
    use crate::*;
    use serde_json::json;
    use std::time::Duration;

    fn view(html: &str, css: &str) -> View {
        setup_tests_logging();
        View::from_html(html, css, "./assets", DummyFonts, FakeTranslator::new())
            .expect("view valid and compiling complete")
    }

    fn input(time: f32) -> Input {
        Input::new().time(Duration::from_secs_f32(time))
    }

    #[test]
    pub fn test_template_with_array_alias() {
        let css = "";
        let html = r##"<html>
            <template id="my-component">
                <div *item="5 {items}" @id="{item}"></div>
            </template>
            <body>
                <div id="start"></div>
                <link href="#my-component" +items="{object.items}" />
                <div id="end"></div>
            </body>
        </html>"##;
        let mut view = view(html, css);
        let value = json!({
            "object": {
                "items": ["a", "b", "c"]
            }
        });
        view.update(Input::new(), value).unwrap();
        let body = view.body();
        let div = body.children();
        assert_eq!(5, div.len(), "elements count");
        assert_eq!(div[0].attrs.get("id"), Some(&"start".to_string()));
        assert_eq!(div[1].attrs.get("id"), Some(&"a".to_string()), "a id");
        assert_eq!(div[2].attrs.get("id"), Some(&"b".to_string()), "b id");
        assert_eq!(div[3].attrs.get("id"), Some(&"c".to_string()), "c id");
        assert_eq!(div[4].attrs.get("id"), Some(&"end".to_string()), "end id");
    }

    #[test]
    pub fn test_template_with_repeat() {
        let css = "";
        let html = r##"<html>
            <template id="my-component">
                <div @id="{item}"></div>
            </template>
            <body>
                <div id="start"></div>
                <link href="#my-component" *item="5 {items}" />
                <div id="end"></div>
            </body>
        </html>"##;
        let mut view = view(html, css);
        let value = json!({
            "items": ["a", "b", "c"]
        });
        view.update(Input::new(), value).unwrap();
        let body = view.body();
        let div = body.children();
        assert_eq!(5, div.len(), "elements count");
        assert_eq!(div[0].attrs.get("id"), Some(&"start".to_string()));
        assert_eq!(div[1].attrs.get("id"), Some(&"a".to_string()), "a id");
        assert_eq!(div[2].attrs.get("id"), Some(&"b".to_string()), "b id");
        assert_eq!(div[3].attrs.get("id"), Some(&"c".to_string()), "c id");
        assert_eq!(div[4].attrs.get("id"), Some(&"end".to_string()), "end id");
    }

    #[test]
    pub fn test_apply_complex_style_with_data_attributes() {
        let css = r#"
            .slot {
                position: absolute;
                left: 0;
                width: 10px;
                height: 10px;
            }
            .slot.placeholder {
                width: 20px;
                height: 20px;
            }
            .slot[data-function="Primary"] {
                left: 10px;
                width: 30px;
            }
            .slot[data-target] {
                width: 40px;
            }
        "#;
        let html = r#"<html>
        <body>
            <div @data-function="{function}" #data-target="{is_target}" class="slot placeholder"></div>
        </body>
        </html>"#;
        let value = json!({
            "function": "Primary",
            "is_target": true
        });
        let mut view = view(html, css);
        view.update(Input::new(), value).unwrap();
        let body = view.body();
        let div = body.children()[0];

        assert_eq!(div.position, [10.0, 0.0], "position");
        assert_eq!(div.size, [40.0, 20.0], "size")
    }

    #[test]
    pub fn test_url_path_resolving() {
        let css = r#"
            div {
                background-image: url("./images/icon.png");
            }
        "#;
        let html = r#"<html><body><div></div></body></html>"#;
        let mut view = view(html, css);
        view.update(Input::new(), json!({})).unwrap();
        let body = view.body();
        let div = body.children()[0];
        assert_eq!(
            div.backgrounds[0].image,
            Some("./images/icon.png".to_string())
        );
    }

    #[test]
    pub fn test_element_position_after_conditional_rerender() {
        let css = r#"
            div {
                height: 10px;
            }
        "#;
        let html = r#"
        <html>
        <body>
            <div ?="{test_a}" id="a"></div>
            <div ?="{test_b}" id="b"></div>
            <div ?="{test_c}" id="c"></div>
        </body>
        </html>"#;
        let mut view = view(html, css);

        let value = json!({"test_a": true, "test_b": false, "test_c": true});
        view.update(Input::new(), value).unwrap();
        let value = json!({"test_a": true, "test_b": true, "test_c": true});
        view.update(Input::new(), value).unwrap();

        let body = view.body();
        let children = body.children();
        let a = children[0];
        let b = children[1];
        let c = children[2];
        assert_eq!(a.attrs.get("id"), Some(&"a".to_string()), "a id");
        assert_eq!(a.position, [0.0, 0.0], "a position");
        assert_eq!(b.attrs.get("id"), Some(&"b".to_string()), "b id");
        assert_eq!(b.position, [0.0, 10.0], "b position");
        assert_eq!(c.attrs.get("id"), Some(&"c".to_string()), "c id");
        assert_eq!(c.position, [0.0, 20.0], "c position");
    }

    #[test]
    pub fn test_relative_position_in_relative_fragment() {
        let css = r#"
            body {
                padding-left: 15px;
                padding-top: 17px;
            }
            .panel {
                position: relative;
                padding: 8px;
            }
            .container {
                position: relative;
            }
            .item {
                position: relative;
                width: 32px;
                height: 32px;
            }
        "#;
        let html = r#"<html>
        <body>
            <div class="panel">
                <div class="container">
                    <div class="item"></div>
                </div>
            </div>
        </body>
        </html>"#;
        let mut view = view(html, css);
        view.update(Input::new(), json!({})).unwrap();
        let body = view.body();
        let panel = body.children()[0];
        let container = panel.children()[0];
        let item = container.children()[0];

        assert_eq!(body.size, [63.0, 65.0]);
        assert_eq!(panel.position, [15.0, 17.0]);
        assert_eq!(container.position, [23.0, 25.0]);
        assert_eq!(container.size, [32.0, 32.0]);
        assert_eq!(item.position, [23.0, 25.0]);
    }

    #[test]
    pub fn test_relative_position_in_absolute_fragment_after_relative() {
        let css = r#"
            body { }
            .relative {
                width: 10px;
                height: 10px;
            }
            .panel {
                position: absolute;
                left: 15px;
                top: 17px;
                padding: 8px;
            }
            .container {
                position: relative;
            }
            .item {
                position: relative;
                width: 32px;
                height: 32px;
            }
        "#;
        let html = r#"<html>
        <body>
            <div class="relative"></div>
            <div class="panel">
                <div class="container">
                    <div class="item"></div>
                </div>
            </div>
        </body>
        </html>"#;
        let mut view = view(html, css);
        view.update(Input::new(), json!({})).unwrap();
        let body = view.body();
        let panel = body.children()[1];
        let container = panel.children()[0];
        let item = container.children()[0];

        assert_eq!(body.size, [10.0, 10.0]);
        assert_eq!(panel.position, [15.0, 17.0]);
        assert_eq!(container.position, [23.0, 25.0]);
        assert_eq!(container.size, [32.0, 32.0]);
        assert_eq!(item.position, [23.0, 25.0]);
    }

    #[test]
    pub fn test_relative_position_after_negative_condition_binding() {
        let css = r#"
            .container {
                width: 48px;
                height: 48px;
                padding: 8px;
            }
            .item {
                width: 32px;
                height: 32px;
            }
        "#;
        let html = r#"<html>
        <body>
            <div class="container">
                <div !="{condition}" class="item"></div>
            </div>
        </body>
        </html>"#;
        let mut view = view(html, css);
        let value = json!({
            "condition": false
        });
        view.update(Input::new(), value).unwrap();
        let body = view.body();
        let container = body.children()[0];
        let item = container.children()[0];

        assert_eq!(container.size, [48.0, 48.0]);
        assert_eq!(item.position, [8.0, 8.0]);
    }

    #[test]
    pub fn test_visibility() {
        // The visibility shows or hides an element without changing the layout of a document.
        let css = r#"
            div {
                width: 32px;
                height: 32px;
            }
            .visible {
                visibility: visible;
            }
            .hidden {
                visibility: hidden;
            }
        "#;
        let html = r#"<html>
        <body>
            <div class="visible"></div>
            <div class="hidden" ^onmousedown="B"></div>
            <div class="visible" ^onmousedown="C"></div>
        </body>
        </html>"#;
        let user_input = vec![
            InputEvent::MouseMove([20.0, 33.0]),
            InputEvent::MouseButtonDown(MouseButtons::Left),
            InputEvent::MouseMove([20.0, 65.0]),
            InputEvent::MouseButtonDown(MouseButtons::Left),
        ];
        let mut view = view(html, css);
        let value = json!({});
        let output = view.update(Input::new().events(user_input), value).unwrap();
        let body = view.body();
        let div = &body.children();
        assert_eq!(div[0].position, [0.0, 0.0]);
        assert_eq!(div[0].visible, true, "0-th visible");
        assert_eq!(div[1].position, [0.0, 32.0]);
        assert_eq!(div[1].visible, false, "1-th hidden");
        assert_eq!(div[2].position, [0.0, 64.0]);
        assert_eq!(div[2].visible, true, "2-th visible");
        assert_eq!(vec!["C".to_string()], output.messages);
    }

    #[test]
    pub fn test_nested_positive_condition_binding_with_nullable() {
        let html = r#"
        <html>
            <body>
                <div ?="{visible}" +item="{nested}">
                    <header>Nested Item</header>
                    <div ?="{item.prop_a}">Property A: {item.prop_a}</div>
                    <div ?="{item.prop_b}">Property B: {item.prop_b}</div>
                </div>
            </body>
        </html>"#;
        let values = [
            json!({"visible": true, "nested": {"prop_a": 0, "prop_b": 42}}),
            json!({"visible": false, "nested": null}),
        ];
        let mut view = view(html, "");
        for value in values {
            view.update(Input::new(), value).unwrap();
        }
        let body = view.body();
        assert_eq!(body.children().len(), 0);
    }

    #[test]
    pub fn test_null_object_condition_rendering() {
        let html = r#"
        <html>
        <body>
            <div id="a" ?="{object}">{object.name}</div>
            <div id="b"></div>
        </body>
        </html>"#;
        let mut view = view(html, "");
        view.update(Input::new(), json!({"object": null})).unwrap();
        let body = view.body();
        let children = body.children();
        let b = children[0];
        assert_eq!(children.len(), 1);
        assert_eq!(b.attrs.get("id"), Some(&"b".to_string()));
    }

    #[test]
    pub fn test_transition_simple_forward_by_style() {
        let css = r#"
            div {
                width: 0px;
                height: 20px;
                transition: width 1s;
            }
        "#;
        let html = r#"
        <html>
            <body>
                <div @style="width: {width}px;"></div>
            </body>
        </html>"#;
        let timeline = [
            (0.1, json!({ "width": 0})),
            (0.1, json!({ "width": 0})),
            (0.1, json!({ "width": 100 })),
            (0.1, json!({ "width": 100 })),
            (0.1, json!({ "width": 100 })),
            (0.8, json!({ "width": 100 })),
            (0.1, json!({ "width": 100 })),
        ];
        let mut view = view(html, css);

        let mut changes: Vec<f32> = vec![];
        for (time, value) in timeline {
            view.update(input(time), value).unwrap();
            let [width, _height] = view.body().children()[0].size;
            changes.push(width);
        }

        assert_eq!(changes, [0.0, 0.0, 0.0, 10.0, 20.0, 100.0, 100.0]);
    }

    #[test]
    pub fn test_transition_simple_forward_by_class() {
        let css = r#"
            div {
                width: 0px;
                height: 20px;
                transition: width 1s;
            }
            div.open {
                width: 100px;
            }
        "#;
        let html = r#"
        <html>
            <body>
                <div @class="{class}"></div>
            </body>
        </html>"#;
        let timeline = [
            (0.1, json!({ "class": ""})),
            (0.1, json!({ "class": ""})),
            (0.1, json!({ "class": "open" })),
            (0.1, json!({ "class": "open" })),
            (0.1, json!({ "class": "open" })),
            (0.8, json!({ "class": "open" })),
            (0.1, json!({ "class": "open" })),
        ];
        let mut view = view(html, css);

        let mut changes: Vec<f32> = vec![];
        for (time, value) in timeline {
            view.update(input(time), value).unwrap();
            let [width, _height] = view.body().children()[0].size;
            changes.push(width);
        }

        assert_eq!(changes, [0.0, 0.0, 0.0, 10.0, 20.0, 100.0, 100.0]);
    }

    #[test]
    pub fn test_transition_simple_mixed_by_class() {
        let css = r#"
            div {
                width: 0px;
                height: 20px;
                transition: width 1s;
            }
            div.open {
                width: 100px;
            }
        "#;
        let html = r#"
        <html>
            <body>
                <div @class="{class}"></div>
            </body>
        </html>"#;
        let timeline = [
            (0.1, json!({ "class": ""})),
            (0.1, json!({ "class": "open" })),
            (0.1, json!({ "class": "open" })),
            (0.1, json!({ "class": "" })),
            (0.1, json!({ "class": "" })),
            (0.8, json!({ "class": "" })),
            (0.1, json!({ "class": "" })),
        ];
        let mut view = view(html, css);

        let mut changes: Vec<f32> = vec![];
        for (time, value) in timeline {
            view.update(input(time), value).unwrap();
            let [width, _height] = view.body().children()[0].size;
            changes.push(width);
        }

        assert_eq!(changes, [0.0, 0.0, 10.0, 20.0, 18.0, 2.0, 0.0]);
    }

    #[test]
    pub fn test_none_pointer_events() {
        let css = r#"
            body {
                pointer-events: none;
            }
            div {
                pointer-events: auto;
                width: 32px;
                height: 32px;
            }
        "#;
        let html = r#"<html>
        <body ^onmouseenter="enter {body}" ^onmouseleave="leave {body}">
            <div ^onmouseenter="enter {a}" ^onmouseleave="leave {a}"></div>
        </body>
        </html>"#;
        let value = json!({
            "body": "Body",
            "a": "A",
        });
        let mut view = view(html, css);

        let user_input = vec![
            InputEvent::MouseMove([20.0, 20.0]),
            InputEvent::MouseMove([20.0, 40.0]),
        ];
        let mut output = Output::new();
        for event in user_input {
            output = view
                .update(Input::new().event(event), value.clone())
                .expect("valid update");
        }

        assert_eq!(output.is_input_captured, false, "cursor over view");
        assert_eq!(output.messages, vec![msg("leave", "A")]);
    }

    #[test]
    pub fn test_mouse_click_event() {
        let css = r#"
            div {
                width: 32px;
                height: 32px;
            }
        "#;
        let html = r#"<html>
        <body>
            <div ^onclick="Hello {name}"></div>
        </body>
        </html>"#;
        let value = json!({ "name": "Alice" });
        let mut view = view(html, css);

        let user_input = vec![
            InputEvent::MouseMove([20.0, 20.0]),
            InputEvent::MouseButtonDown(MouseButtons::Left),
            InputEvent::MouseButtonUp(MouseButtons::Left),
        ];
        let mut output = Output::new();
        for event in user_input {
            output = view
                .update(Input::new().event(event), value.clone())
                .expect("valid update");
        }
        assert_eq!(output.is_input_captured, true, "cursor over view");
        assert_eq!(output.messages, vec![msg("Hello", "Alice")]);
    }

    #[test]
    pub fn test_mouse_enter_leave_events_forward() {
        let css = r#"
            div {
                width: 32px;
                height: 32px;
            }
        "#;
        let html = r#"<html>
        <body>
            <div ^onmouseenter="enter {a}" ^onmouseleave="leave {a}"></div>
            <div ^onmouseenter="enter {b}" ^onmouseleave="leave {b}"></div>
        </body>
        </html>"#;
        let value = json!({
            "a": "A",
            "b": "B"
        });
        let mut view = view(html, css);

        let user_input = vec![
            InputEvent::MouseMove([100.0, 20.0]),
            InputEvent::MouseMove([20.0, 20.0]),
            InputEvent::MouseMove([20.0, 40.0]),
        ];
        let mut messages = vec![];
        for event in user_input {
            let output = view
                .update(Input::new().event(event), value.clone())
                .expect("valid update");
            messages.extend(output.messages);
        }

        assert_eq!(
            messages,
            vec![msg("enter", "A"), msg("leave", "A"), msg("enter", "B")]
        );
    }

    #[test]
    pub fn test_mouse_enter_leave_over_border() {
        let css = r#"
            div {
                width: 32px;
                height: 32px;
            }
        "#;
        let html = r#"<html>
        <body>
            <div ^onmouseenter="enter {a}" ^onmouseleave="leave {a}"></div>
            <div ^onmouseenter="enter {b}" ^onmouseleave="leave {b}"></div>
        </body>
        </html>"#;
        let value = json!({
            "a": "A",
            "b": "B"
        });
        let mut view = view(html, css);

        let user_input = vec![
            InputEvent::MouseMove([20.0, 20.0]),
            InputEvent::MouseMove([20.0, 32.0]),
            InputEvent::MouseMove([20.0, 40.0]),
        ];
        let mut messages = vec![];
        for event in user_input {
            let output = view
                .update(Input::new().event(event), value.clone())
                .expect("valid update");
            messages.extend(output.messages);
        }

        assert_eq!(
            messages,
            vec![msg("enter", "A"), msg("leave", "A"), msg("enter", "B")]
        );
    }

    #[test]
    pub fn test_mouse_enter_leave_events_backward() {
        let css = r#"
            div {
                width: 32px;
                height: 32px;
            }
        "#;
        let html = r#"<html>
        <body>
            <div ^onmouseenter="enter {a}" ^onmouseleave="leave {a}"></div>
            <div ^onmouseenter="enter {b}" ^onmouseleave="leave {b}"></div>
        </body>
        </html>"#;
        let value = json!({
            "a": "A",
            "b": "B"
        });
        let mut view = view(html, css);

        let user_input = vec![
            InputEvent::MouseMove([100.0, 40.0]),
            InputEvent::MouseMove([20.0, 40.0]),
            InputEvent::MouseMove([20.0, 20.0]),
        ];
        let mut messages = vec![];
        for event in user_input {
            let output = view
                .update(Input::new().event(event), value.clone())
                .expect("valid update");
            messages.extend(output.messages);
        }
        assert_eq!(
            messages,
            vec![msg("enter", "B"), msg("leave", "B"), msg("enter", "A")]
        );
    }

    #[test]
    pub fn test_mouse_enter_leave_events_via_animation() {
        let css = r#"
            div {
                width: 32px;
                height: 32px;
                animation: 1s linear HeightAnimation;
            }
            @keyframes HeightAnimation {
                0% {
                    height: 32px;
                }
                50% {
                    height: 64px;
                }
                100% {
                    height: 32px;
                }
            }
        "#;
        let html = r#"<html>
        <body>
            <div ^onmouseenter="enter {a}" ^onmouseleave="leave {a}"></div>
        </body>
        </html>"#;
        let value = json!({
            "a": "A",
        });
        let mut view = view(html, css);
        let initial_mouse_input = Input::new().event(InputEvent::MouseMove([20.0, 40.0]));
        view.update(initial_mouse_input, value.clone())
            .expect("valid update");

        let mut output = Output::new();
        for time in [0.0, 0.49, 1.0].map(Duration::from_secs_f32) {
            output = view
                .update(Input::new().time(time), value.clone())
                .expect("valid update");
        }

        assert_eq!(output.is_input_captured, false, "cursor over view");
        assert_eq!(output.messages, vec![msg("leave", "A")]);
    }

    fn msg(key: &str, value: &str) -> Value {
        json!({
            key: value
        })
    }
}
