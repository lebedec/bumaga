use crate::css::{read_css, Css};
use crate::html::{read_html, Html};
use crate::models::{ElementId, Sizes};
use crate::rendering::{get_object_value, Schema};
use crate::state::State;
use crate::styles::{create_element, default_layout, Cascade, Scrolling};
use crate::update::measure_text;
use crate::view_model::{Reaction, ViewModel};
use crate::{Component, Element, Input, Output, ViewError};
use log::error;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::ops::Add;
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AlignItems, Display, Layout, NodeId, Point, Position, PrintTree, Size, Style, TaffyTree,
};

pub struct View {
    view_model: ViewModel,
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
        tree.set_node_context(root, Some(create_element(ElementId::fake())))?;
        let body = html.children.last().cloned().expect("body must be found");
        let body =
            Component::render_node(body, &mut tree, &mut bindings, &mut locals, &mut schema)?;
        tree.add_child(root, body)?;
        let view_model = ViewModel::create(bindings, schema.value);
        let resources = resources.to_string();
        Ok(Self {
            view_model,
            tree,
            root,
            body,
            css,
            resources,
        })
    }

    pub fn update(&mut self, mut input: Input) -> Result<Output, ViewError> {
        let reactions = self.view_model.bind(&Value::Object(input.value.clone()));
        for reaction in reactions {
            self.update_tree(reaction).unwrap();
        }
        // detect viewport changes
        let [viewport_width, viewport_height] = input.viewport;
        let mut root_layout = self.tree.style(self.root).unwrap().clone();
        let root = self.tree.get_node_context_mut(self.root).unwrap();
        root_layout.size = Size {
            width: length(viewport_width),
            height: length(viewport_height),
        };

        let [viewport_width, viewport_height] = input.viewport;
        let sizes = Sizes {
            root_font_size: root.text_style.font_size,
            parent_font_size: root.text_style.font_size,
            viewport_width,
            viewport_height,
        };
        self.tree.set_style(self.root, root_layout)?;
        //
        self.apply_styles(self.body, &mut input, sizes)?;
        //
        self.tree.compute_layout_with_measure(
            self.body,
            Size::MAX_CONTENT,
            |size, space, _, view, _| measure_text(&mut input, size, space, view),
        )?;
        let mut output = Output::new();
        // TODO: rework output traverse, gathering and element save
        self.gather(self.body, &mut output);
        Ok(output)
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
                let shown = &children[start..cursor];
                let hidden = &children[cursor..end];
                for node in shown {
                    // The child is not removed from the tree entirely,
                    // it is simply no longer attached to its previous parent.
                    self.tree.remove_child(parent, *node)?;
                }
                for node in hidden {
                    self.tree.add_child(parent, *node)?;
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
        //self.state.restore(element);
        // default html styles
        match element.tag.as_str() {
            "input" => {
                layout.display = Display::Flex;
                layout.align_items = Some(AlignItems::Center);
            }
            _ => {}
        }
        let mut cascade = Cascade::new(&self.css, sizes, "./");
        cascade.apply_styles(input, node, &self.tree, parent, &mut layout, element);
        self.tree.set_style(node, layout)?;

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
                    sizes.parent_font_size = element.text_style.font_size;
                    self.apply_styles(child, input, sizes)?;
                }
            }
        }

        Ok(())
    }

    fn gather(&mut self, node: NodeId, output: &mut Output) {
        let layout = self.tree.get_final_layout(node).clone();
        let element = match self.tree.get_node_context_mut(node) {
            None => {
                error!("unable to traverse node {node:?} has no context");
                return;
            }
            Some(element) => element,
        };
        element.layout = layout;
        output.elements.push(element.clone());
        match self.tree.children(node) {
            Ok(children) => {
                for child in children {
                    self.gather(child, output);
                }
            }
            Err(error) => {
                error!("unable to traverse node {node:?}, {error:?}")
            }
        }
    }
}
