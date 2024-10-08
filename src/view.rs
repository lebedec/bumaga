use crate::css::{
    match_style, read_css, read_inline_css, Css, PseudoClassMatcher,
};
use crate::fonts::DummyFonts;
use crate::html::{read_html, ElementBinding, Html};
use crate::metrics::ViewMetrics;
use crate::rendering::Renderer;
use crate::styles::{inherit, Cascade, Scrolling, Sizes, Variables};
use crate::tree::ViewTreeExtensions;
use crate::view_model::{Reaction, ViewModel};
use crate::{
    Element, ElementStyle, Fonts, Input, Output, Transformer,
    ViewError,
};
use log::error;
use mesura::GaugeValue;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::ops::{Add, Deref};
use std::path::PathBuf;
use std::time::SystemTime;
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AvailableSpace, Layout, NodeId, Point, PrintTree, Size,
    TaffyTree,
};

pub struct View {
    model: ViewModel,
    pub(crate) tree: TaffyTree<Element>,
    root: NodeId,
    body: NodeId,
    css: Css,
    html_source: Source,
    css_source: Source,
    resources: String,
    pub fonts: Box<dyn Fonts>,
    metrics: ViewMetrics,
}

impl View {
    pub fn from_html(path: &str, fonts: impl Fonts + 'static) -> Result<Self, ViewError> {
        let mut html_source = Source::file(path);
        let html = html_source.get_content()?;
        let html = read_html(&html)?;
        // TODO: rework
        let mut css_files = vec![];
        let css_base_directory = html_source.folder();
        let mut body = Html::empty();
        for child in html.children {
            if child.tag == "link" {
                let mut attrs = HashMap::new();
                for binding in &child.bindings {
                    if let ElementBinding::None(key, value) = binding {
                        attrs.insert(key.clone(), value.as_str());
                    }
                }
                if attrs.get("rel") == Some(&"stylesheet") {
                    if let Some(href) = attrs.get("href") {
                        let mut file = css_base_directory.clone();
                        file.push(href);
                        css_files.push(file);
                    }
                }
            }
            if child.tag == "body" {
                body = child;
                break;
            }
        }
        let mut css_source = Source::files(css_files);
        let css = css_source.get_content()?;
        let css = read_css(&css)?;
        //
        let mut renderer = Renderer::new();
        let [root, body] = renderer.render(body)?;
        let bindings = renderer.bindings;
        let schema = renderer.schema;
        let tree = renderer.tree;
        let model = ViewModel::create(bindings, schema.value);
        let resources = css_base_directory.display().to_string();
        let mut view = Self {
            model,
            tree,
            root,
            body,
            css,
            html_source,
            css_source,
            resources,
            fonts: Box::new(fonts),
            metrics: ViewMetrics::new(),
        };
        view.calculate_elements_stylesheet(body)?;
        Ok(view)
    }

    pub fn fonts(mut self, fonts: impl Fonts + 'static) -> Self {
        self.fonts = Box::new(fonts);
        self
    }

    pub fn compile(html: &str, css: &str, resources: &str) -> Result<Self, ViewError> {
        let html = Source::memory(html);
        let css = Source::memory(css);
        Self::create(html, css, resources)
    }

    pub fn watch(html: &str, css: &str, resources: &str) -> Result<Self, ViewError> {
        let html = Source::file(html);
        let css = Source::file(css);
        Self::create(html, css, resources)
    }

    pub fn create(
        mut html_source: Source,
        mut css_source: Source,
        resources: &str,
    ) -> Result<Self, ViewError> {
        let html = html_source.get_content()?;
        let css = css_source.get_content()?;
        let html = read_html(&html)?;
        let css = read_css(&css)?;
        let mut renderer = Renderer::new();
        // TODO: remove cloned, take ownership
        for child in &html.children {
            if child.tag == "link" {
                let mut attrs = HashMap::new();
                for binding in &child.bindings {
                    if let ElementBinding::None(key, value) = binding {
                        attrs.insert(key.clone(), value.as_str());
                    }
                }
                if attrs.get("rel") == Some(&"stylesheet") {
                    if let Some(_href) = attrs.get("href") {}
                }
            }
        }
        let body = html
            .children
            .last()
            .cloned()
            .ok_or(ViewError::BodyNotFound)?;
        //
        let [root, body] = renderer.render(body)?;
        let bindings = renderer.bindings;
        let schema = renderer.schema;
        let tree = renderer.tree;
        let model = ViewModel::create(bindings, schema.value);
        let resources = resources.to_string();
        let mut view = Self {
            model,
            tree,
            root,
            body,
            css,
            html_source,
            css_source,
            resources,
            fonts: Box::new(DummyFonts),
            metrics: ViewMetrics::new(),
        };
        view.calculate_elements_stylesheet(body)?;
        Ok(view)
    }

    pub fn pipe(mut self, name: &str, transformer: Transformer) -> Self {
        self.model
            .transformers
            .insert(name.to_string(), transformer);
        self
    }

    fn watch_changes(&mut self) {
        if self.html_source.detect_changes() || self.css_source.detect_changes() {
            let view = View::create(
                self.html_source.clone(),
                self.css_source.clone(),
                &self.resources,
            );
            match view {
                Ok(mut view) => {
                    view.model.transformers = self.model.transformers.clone();
                    self.model = view.model;
                    self.tree = view.tree;
                    self.root = view.root;
                    self.body = view.body;
                    self.css = view.css;
                }
                Err(error) => {
                    error!("unable to handle view changes, {error:?}")
                }
            }
        }
    }

    pub fn update(&mut self, input: Input, value: Value) -> Result<Output, ViewError> {
        self.metrics.updates.inc();
        self.watch_changes();
        let reactions = self.model.bind(&value);
        for reaction in reactions {
            self.update_tree(reaction).unwrap();
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
        self.compute_final_positions_and_clipping(self.body, Point::ZERO, None)?;
        self.model.handle_output(&input, self.body, &mut self.tree)
    }

    fn compute_final_positions_and_clipping(
        &mut self,
        node: NodeId,
        location: Point<f32>,
        mut clipping: Option<Layout>,
    ) -> Result<(), ViewError> {
        self.metrics.elements_shown.inc();
        let mut layout = self.tree.get_final_layout(node).clone();
        layout.location = layout.location.add(location);
        let element = self.tree.get_node_context_mut(node).unwrap();
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
        for child in self.tree.children(node).unwrap() {
            self.compute_final_positions_and_clipping(child, location, clipping)?;
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
                let now = self.tree.children(parent)?;
                if visible {
                    if !now.contains(&node) {
                        self.tree.add_child(parent, node)?;
                    }
                } else {
                    if now.contains(&node) {
                        self.tree.remove_child(parent, node)?;
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
                match (element.tag.as_ref(), key.as_ref()) {
                    ("option", "selected") => {
                        self.model
                            .update_option_selected(node, tag, &mut self.tree)?
                    }
                    ("input", "disabled") => {
                        self.model
                            .update_input_disabled(node, tag, &mut self.tree)?
                    }
                    _ => {}
                }
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
                match (element.tag.as_str(), key.as_str()) {
                    ("img", "src") => self.model.update_img_src(node, value, &mut self.tree)?,
                    ("input", "value") => {
                        self.model.update_input_value(node, value, &mut self.tree)?
                    }
                    _ => {}
                }
            }
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
        let mut cascade = Cascade::new(&self.css, sizes, variables, &self.resources);
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
                    self.apply_styles(child, input, sizes, variables.clone())?;
                }
            }
        }

        Ok(())
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

#[derive(Clone)]
enum Source {
    Memory(String),
    File(PathBuf, SystemTime),
    Files(Vec<(PathBuf, SystemTime)>),
}

impl Source {
    fn memory(content: &str) -> Self {
        Self::Memory(content.to_string())
    }

    fn file(path: &str) -> Self {
        Self::File(PathBuf::from(path), SystemTime::UNIX_EPOCH)
    }

    fn files(files: Vec<PathBuf>) -> Self {
        Self::Files(
            files
                .into_iter()
                .map(|path| (path, SystemTime::UNIX_EPOCH))
                .collect(),
        )
    }

    fn folder(&self) -> PathBuf {
        match self {
            Source::Memory(_) => PathBuf::from("."),
            Source::File(path, _) => {
                let mut path = path.clone();
                path.pop();
                path
            }
            Source::Files(files) => {
                let mut path = files[0].0.clone();
                path.pop();
                path
            }
        }
    }

    fn get_content(&mut self) -> Result<String, ViewError> {
        match self {
            Source::Memory(content) => Ok(content.clone()),
            Source::File(path, modified) => {
                *modified = Self::modified(path);
                fs::read_to_string(path).map_err(ViewError::from)
            }
            Source::Files(files) => {
                let mut content = String::new();
                for (path, modified) in files.iter_mut() {
                    *modified = Self::modified(path);
                    content += &fs::read_to_string(path).map_err(ViewError::from)?;
                }
                Ok(content)
            }
        }
    }

    fn detect_changes(&mut self) -> bool {
        match self {
            Source::Memory(_) => false,
            Source::File(path, modified) => {
                let timestamp = Self::modified(&path);
                if *modified < timestamp {
                    *modified = timestamp;
                    true
                } else {
                    false
                }
            }
            Source::Files(files) => {
                for (path, modified) in files.iter_mut() {
                    let timestamp = Self::modified(&path);
                    if *modified < timestamp {
                        *modified = timestamp;
                        return true;
                    }
                }
                false
            }
        }
    }

    fn modified(path: &PathBuf) -> SystemTime {
        match fs::metadata(path).and_then(|meta| meta.modified()) {
            Ok(modified) => modified,
            Err(error) => {
                error!("unable to get {} metadata, {error:?}", path.display());
                SystemTime::now()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::setup_tests_logging;
    use crate::{Call, InputEvent};
    use serde::Serialize;
    use serde_json::json;
    use std::time::Duration;

    fn call<T: Serialize>(function: &str, value: T) -> Call {
        Call {
            function: function.to_string(),
            arguments: vec![serde_json::to_value(value).expect("valid value")],
        }
    }

    fn view(html: &str, css: &str) -> View {
        setup_tests_logging();
        View::compile(html, css, "./assets").expect("view valid and compiling complete")
    }

    fn input(time: f32) -> Input {
        Input::new().time(Duration::from_secs_f32(time))
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
            Some("./assets/./images/icon.png".to_string())
        );
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
        let mut view = View::compile(html, css, "").expect("view valid");

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

        assert_eq!(output.is_cursor_over_view, false, "cursor over view");
        assert_eq!(output.calls, vec![call("leave", "A")]);
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
        let mut view = View::compile(html, css, "").expect("view valid");

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

        assert_eq!(output.is_cursor_over_view, true, "cursor over view");
        assert_eq!(output.calls, vec![call("leave", "A"), call("enter", "B")]);
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
        let mut view = View::compile(html, css, "").expect("view valid");

        let user_input = vec![
            InputEvent::MouseMove([20.0, 40.0]),
            InputEvent::MouseMove([20.0, 20.0]),
        ];
        let mut output = Output::new();
        for event in user_input {
            output = view
                .update(Input::new().event(event), value.clone())
                .expect("valid update");
        }
        assert_eq!(output.is_cursor_over_view, true, "cursor over view");
        assert_eq!(output.calls, vec![call("leave", "B"), call("enter", "A")]);
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
        let mut view = View::compile(html, css, "").expect("view valid");
        let initial_mouse_input = Input::new().event(InputEvent::MouseMove([20.0, 40.0]));
        view.update(initial_mouse_input, value.clone())
            .expect("valid update");

        let mut output = Output::new();
        for time in [0.0, 0.49, 1.0].map(Duration::from_secs_f32) {
            output = view
                .update(Input::new().time(time), value.clone())
                .expect("valid update");
        }

        assert_eq!(output.is_cursor_over_view, false, "cursor over view");
        assert_eq!(output.calls, vec![call("leave", "A")]);
    }
}
