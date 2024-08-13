use crate::css::{read_css, Css, PseudoClassMatcher};
use crate::html::{read_html, Html};
use crate::input::DummyFonts;
use crate::models::Sizes;
use crate::state::State;
use crate::styles::{create_element, default_layout, inherit, Cascade, Scrolling};
use crate::view_model::{Reaction, Schema, ViewModel};
use crate::{Call, Component, Element, Fonts, Input, InputEvent, MouseButtons, ViewError};
use log::error;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::ops::{Add, Deref, DerefMut};
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AlignItems, AvailableSpace, Display, Layout, NodeId, Point, Position, PrintTree, Size, Style,
    TaffyTree,
};

pub struct View {
    model: ViewModel,
    tree: TaffyTree<Element>,
    root: NodeId,
    body: NodeId,
    css: Css,
    resources: String,
}

impl View {
    pub fn compile(html: &str, css: &str, resources: &str) -> Result<Self, ViewError> {
        let css = read_css(css)?;
        let html = read_html(&html)?;
        let mut tree = TaffyTree::new();
        let mut bindings = BTreeMap::new();
        let mut locals = HashMap::new();
        let mut schema = Schema::new();
        let root = tree.new_leaf(default_layout())?;
        tree.set_node_context(root, Some(create_element(root)))?;
        let body = html.children.last().cloned().expect("body must be found");
        let body =
            Component::render_node(body, &mut tree, &mut bindings, &mut locals, &mut schema)?;
        tree.add_child(root, body)?;
        let model = ViewModel::create(bindings, schema.value);
        let resources = resources.to_string();
        Ok(Self {
            model,
            tree,
            root,
            body,
            css,
            resources,
        })
    }

    pub fn update(&mut self, mut input: Input) -> Result<Vec<Call>, ViewError> {
        let reactions = self
            .model
            .bind(&Value::Object(input.value.clone()), &input.transformers);
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
        self.compute_positions_and_clipping(self.body, Point::ZERO);
        self.model.handle_output(&input, self.body, &mut self.tree)
    }

    pub fn compute_positions_and_clipping(&mut self, node: NodeId, parent: Point<f32>) {
        let layout = self.tree.get_final_layout(node);
        let position = layout.location.add(parent);
        let size = [layout.size.width, layout.size.height];
        let element = self.tree.get_node_context_mut(node).unwrap();
        element.position = [position.x, position.y];
        element.size = size;
        for child in self.tree.children(node).unwrap() {
            self.compute_positions_and_clipping(child, position);
        }
    }

    pub fn update_tree(&mut self, reaction: Reaction) -> Result<(), ViewError> {
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
                    .tree
                    .get_node_context(parent)
                    .map(|element| element.children.clone())
                    .ok_or(ViewError::ElementNotFound)?;
                let visible = self.tree.children(parent)?;
                let shown = &children[start..cursor];
                let hidden = &children[cursor..end];
                for node in shown {
                    if !visible.contains(node) {
                        self.tree.add_child(parent, *node)?;
                    }
                }
                for node in hidden {
                    self.tree.remove_child(parent, *node)?;
                }
            }
            Reaction::Bind { node, key, value } => {
                let element = self
                    .tree
                    .get_node_context_mut(node)
                    .ok_or(ViewError::ElementNotFound)?;
                element.attrs.insert(key, value);
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
            let ptr = self
                .tree
                .get_node_context_mut(node)
                .ok_or(ViewError::ElementNotFound)? as *mut Element;
            &mut *ptr
        };

        if element.text.is_some() {
            inherit(parent, element);
            return Ok(());
        }

        match element.tag.as_str() {
            // TODO: move to render ?
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
        let element = self.tree.get_node_context(self.body).unwrap();
        let layout = self.tree.get_final_layout(self.body);
        Fragment {
            element,
            layout,
            tree: &self.tree,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Fragment<'t> {
    pub element: &'t Element,
    /// The final result of a layout algorithm, describes size and position of element
    pub layout: &'t Layout,
    pub tree: &'t TaffyTree<Element>,
}

impl Fragment<'_> {
    pub fn children(&self) -> Vec<Fragment> {
        match self.tree.children(self.element.node) {
            Ok(children) => children
                .iter()
                .map(|node| {
                    let layout = self.tree.get_final_layout(*node);
                    let element = self.tree.get_node_context(*node).unwrap();
                    Fragment {
                        element,
                        layout,
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
    if let Some(text) = element.text.as_ref().map(|text| text.spans.join(" ")) {
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
            /// The :focus CSS pseudo-class represents an element (such as a form input) that
            /// has received focus. It is generally triggered when the user clicks or taps
            /// on an element or selects it with the keyboard's Tab key.
            "focus" => self.model.focus == Some(element.node),
            /// The :blank CSS pseudo-class selects empty user input elements (e.g. <input>)
            "blank" => element
                .state
                .value
                .as_ref()
                .map(|value| value.is_empty())
                .unwrap_or(true),
            _ => {
                error!("unable to match unknown pseudo class {class}");
                false
            }
        }
    }
}
