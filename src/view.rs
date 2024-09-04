use crate::css::{read_css, Css, PseudoClassMatcher};
use crate::html::{read_html, ElementBinding, Html};
use crate::input::DummyFonts;
use crate::rendering::Renderer;
use crate::styles::{create_element, default_layout, inherit, Cascade, Scrolling, Sizes};
use crate::view_model::{Reaction, Schema, ViewModel};
use crate::{
    Call, Element, Fonts, Input, InputEvent, MouseButtons, Output, Transformer, ValueExtensions,
    ViewError,
};
use log::error;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::ops::{Add, Deref, DerefMut};
use std::path::PathBuf;
use std::time::SystemTime;
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AlignItems, AvailableSpace, Dimension, Display, Layout, LengthPercentageAuto, NodeId, Point,
    Position, PrintTree, Rect, Size, Style, TaffyTree,
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
}

impl View {
    pub fn from_html(path: &str) -> Result<Self, ViewError> {
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
        })
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
                    if let Some(href) = attrs.get("href") {}
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

    pub fn update(&mut self, mut input: Input) -> Result<Output, ViewError> {
        self.watch_changes();
        let reactions = self.model.bind(&Value::Object(input.value.clone()));
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
        self.apply_styles(self.body, &mut input, sizes)?;
        self.tree.compute_layout_with_measure(
            self.body,
            Size::MAX_CONTENT,
            |size, space, _, view, _| measure_text(&mut input, size, space, view),
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
            Reaction::Reattach { node, visible } => {
                let parent = self.tree.parent(node).ok_or(ViewError::ParentNotFound)?;
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
            Reaction::Bind { node, key, value } => {
                let element = self.get_element_mut(node)?;
                element.attrs.insert(key.clone(), value.eval_string());
                let node = element.node;
                match element.tag.as_str() {
                    "select" => match key.as_str() {
                        "value" => self.update_select_view(node, value)?,
                        _ => {}
                    },
                    "img" => match key.as_str() {
                        "src" => self.update_img_view(node, value.eval_string())?,
                        _ => {}
                    },
                    "input" => match key.as_str() {
                        "value" => self.update_input_view(node, value.eval_string())?,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn apply_styles(
        &mut self,
        node: NodeId,
        input: &mut Input,
        mut sizes: Sizes,
    ) -> Result<(), ViewError> {
        let parent = unsafe {
            let ptr = self
                .tree
                .parent(node)
                .and_then(|parent| self.tree.get_node_context(parent))
                .ok_or(ViewError::ParentNotFound)? as *const Element;
            // TODO:
            &*ptr
        };
        let mut layout = self.tree.style(node)?.clone();
        let element = unsafe {
            let ptr = self.get_element_mut(node)? as *mut Element;
            &mut *ptr
        };

        if element.text.is_some() {
            inherit(parent, element);
            return Ok(());
        }

        match element.tag.as_str() {
            // TODO: move to render ?
            "body" => {
                layout.size = Size {
                    width: Dimension::Length(sizes.viewport_width),
                    height: Dimension::Length(sizes.viewport_height),
                };
                layout.margin = Rect {
                    left: LengthPercentageAuto::Length(8.0),
                    right: LengthPercentageAuto::Length(8.0),
                    top: LengthPercentageAuto::Length(8.0),
                    bottom: LengthPercentageAuto::Length(8.0),
                };
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

    #[inline(always)]
    pub(crate) fn get_element_mut(&mut self, node: NodeId) -> Result<&mut Element, ViewError> {
        self.tree
            .get_node_context_mut(node)
            .ok_or(ViewError::ElementNotFound)
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

fn measure_text(
    input: &mut Input,
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
        let [width, height] = match input.fonts.as_mut() {
            None => DummyFonts.measure(&text, &element.font, max_width),
            Some(fonts) => fonts.measure(&text, &element.font, max_width),
        };
        return Size { width, height };
    }
    Size::ZERO
}

impl PseudoClassMatcher for View {
    fn has_pseudo_class(&self, element: &Element, class: &str) -> bool {
        match class {
            "hover" => element.state.hover,
            "active" => element.state.active,
            /// The :checked CSS pseudo-class represents any radio, checkbox, or option element
            /// that is checked or toggled to an "on" state.
            "checked" => element.state.checked,
            /// The :focus CSS pseudo-class represents an element (such as a form input) that
            /// has received focus. It is generally triggered when the user clicks or taps
            /// on an element or selects it with the keyboard's Tab key.
            "focus" => element.state.focus,
            /// The :blank CSS pseudo-class selects empty user input elements.
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
