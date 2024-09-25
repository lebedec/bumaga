use crate::css::{
    read_css, read_declaration_block, Css, Declaration, PseudoClassMatcher, ReaderError,
};
use crate::fonts::DummyFonts;
use crate::html::{read_html, ElementBinding, Html};
use crate::rendering::Renderer;
use crate::styles::{inherit, Cascade, Scrolling, Sizes};
use crate::tree::ViewTreeExtensions;
use crate::view_model::{Reaction, ViewModel};
use crate::{
    Element, Fonts, Input, Output, PointerEvents, Transformer, ValueExtensions, ViewError,
};
use log::{error, info};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::ops::{Add, Deref, DerefMut};
use std::path::PathBuf;
use std::time::SystemTime;
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AlignItems, AvailableSpace, Display, Layout, NodeId, Point, Position, PrintTree, Size,
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
    pub visible: bool,
    pub fonts: Box<dyn Fonts>,
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
        let resources = "./todo".to_string();
        Ok(Self {
            model,
            tree,
            root,
            body,
            css,
            html_source,
            css_source,
            resources,
            visible: true,
            fonts: Box::new(fonts),
        })
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
        Ok(Self {
            model,
            tree,
            root,
            body,
            css,
            html_source,
            css_source,
            resources,
            visible: true,
            fonts: Box::new(DummyFonts),
        })
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
        self.apply_styles(self.body, &input, sizes)?;
        self.tree.compute_layout_with_measure(
            self.body,
            Size::MAX_CONTENT,
            |size, space, _, view, _| measure_text(self.fonts.as_ref(), size, space, view),
        )?;
        // TODO: clipping of viewport
        self.compute_positions_and_clipping(self.body, Point::ZERO, None)?;
        self.model.handle_output(&input, self.body, &mut self.tree)
    }

    fn compute_positions_and_clipping(
        &mut self,
        node: NodeId,
        location: Point<f32>,
        mut clipping: Option<Layout>,
    ) -> Result<(), ViewError> {
        let mut layout = self.tree.get_final_layout(node).clone();
        let style = self.tree.style(node)?;
        if style.position == Position::Relative {
            layout.location = layout.location.add(location);
        }
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
            self.compute_positions_and_clipping(child, location, clipping)?;
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
                element_text.spans[span] = text;
                self.tree.mark_dirty(node)?;
            }
            Reaction::Reattach {
                parent,
                node,
                visible,
            } => {
                if visible {
                    self.tree.add_child(parent, node)?;
                } else {
                    self.tree.remove_child(parent, node)?;
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
                attribute.spans[span] = text;
                let value = attribute.to_string();
                element.attrs.insert(key.clone(), value.clone());
                if key == "style" {
                    match read_declaration_block(&format!("{{ {} }}", value)) {
                        Ok(style) => {
                            element.style = style;
                        }
                        Err(error) => {
                            error!("unable to parse style of {}, {error:?}", element.tag);
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

    fn apply_styles(
        &mut self,
        node: NodeId,
        input: &Input,
        mut sizes: Sizes,
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

        match element.tag.as_str() {
            // TODO: move to render ?
            "body" => {
                // layout.size = Size {
                //     width: Dimension::Length(sizes.viewport_width),
                //     height: Dimension::Length(sizes.viewport_height),
                // };
                // layout.size = Size {
                //     width: Dimension::Auto,
                //     height: Dimension::Auto,
                // };
                // layout.margin = Rect {
                //     left: LengthPercentageAuto::Length(8.0),
                //     right: LengthPercentageAuto::Length(8.0),
                //     top: LengthPercentageAuto::Length(8.0),
                //     bottom: LengthPercentageAuto::Length(8.0),
                // };
                element.pointer_events = PointerEvents::None;
            }
            "input" => {
                layout.display = Display::Flex;
                layout.align_items = Some(AlignItems::Center);
            }
            _ => {}
        }
        let mut cascade = Cascade::new(&self.css, sizes, "./");
        cascade.apply_styles(input, node, &self.tree, parent, &mut layout, element, self);

        // we must update style only if changes detected to support Taffy cache system
        if self.tree.style(node)? != &layout {
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
                    self.apply_styles(child, input, sizes)?;
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
        for time in [0.0, 0.5, 1.0].map(Duration::from_secs_f32) {
            output = view
                .update(Input::new().time(time), value.clone())
                .expect("valid update");
        }

        assert_eq!(output.is_cursor_over_view, false, "cursor over view");
        assert_eq!(output.calls, vec![call("leave", "A")]);
    }
}
