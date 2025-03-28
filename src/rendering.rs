use log::{error, warn};
use std::collections::{BTreeMap, HashMap};
use taffy::{Dimension, NodeId, Size, TaffyTree};

use crate::css::read_inline_css;
use crate::html::{CallbackArgument, ElementBinding, Html, TextBinding, TextSpan};
use crate::styles::{create_element, default_layout};
use crate::view_model::{Binding, Bindings, Schema};
use crate::{BindingParams, Element, Handler, HandlerArgument, TextContent, ViewError};

pub struct Renderer {
    pub tree: TaffyTree<Element>,
    pub bindings: Bindings,
    pub locals: HashMap<String, String>,
    pub schema: Schema,
    pub templates: HashMap<String, Html>,
    pub static_id: HashMap<String, NodeId>,
    pub translator: Box<dyn RendererTranslator>,
}

impl Renderer {
    pub fn new(templates: HashMap<String, Html>, translator: Box<dyn RendererTranslator>) -> Self {
        let tree = TaffyTree::new();
        let bindings = BTreeMap::new();
        let locals = HashMap::new();
        let schema = Schema::new();
        let static_id = HashMap::new();
        Self {
            tree,
            bindings,
            locals,
            schema,
            templates,
            static_id,
            translator,
        }
    }

    pub fn render(&mut self, body: Html) -> Result<[NodeId; 2], ViewError> {
        let root = self.tree.new_leaf(default_layout())?;
        self.tree
            .set_node_context(root, Some(create_element(root)))?;
        let body = self.render_node(body)?;
        self.tree.add_child(root, body)?;
        Ok([root, body])
    }

    fn render_node(&mut self, template: Html) -> Result<NodeId, ViewError> {
        if let Some(text) = template.text {
            self.render_text(text)
        } else {
            self.render_element(template)
        }
    }

    pub(crate) fn render_text(&mut self, text: TextBinding) -> Result<NodeId, ViewError> {
        let layout = default_layout();
        let node = self.tree.new_leaf(layout)?;
        let spans = text
            .spans
            .into_iter()
            .enumerate()
            .map(|(index, span)| match span {
                TextSpan::String(span) => self.translator.translate(span),
                TextSpan::Binder(binder) => {
                    let path = self.schema.field(&binder, &mut self.locals);
                    let params = BindingParams::Text(node, index);
                    let binding = Binding {
                        params,
                        pipe: binder.pipe.clone(),
                    };
                    self.bindings.entry(path).or_default().push(binding);
                    binder.to_string()
                }
            })
            .collect();
        let text = TextContent::new(spans);
        let mut element = create_element(node);
        element.text = Some(text);
        self.tree.set_node_context(node, Some(element))?;
        Ok(node)
    }

    pub(crate) fn render_bg_image(&mut self, src: String) -> Result<NodeId, ViewError> {
        let mut layout = default_layout();
        layout.size = Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Percent(1.0),
        };
        layout.min_size = Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Percent(1.0),
        };
        let node = self.tree.new_leaf(layout)?;
        let mut element = create_element(node);
        element.get_background_mut(0).image = Some(src);
        element.get_background_mut(0).is_src = true;
        self.tree.set_node_context(node, Some(element))?;
        Ok(node)
    }

    fn render_element(&mut self, template: Html) -> Result<NodeId, ViewError> {
        let layout = default_layout();
        let node = self.tree.new_leaf(layout)?;
        let mut element = create_element(node);
        element.tag = template.tag.clone();
        for binding in template.bindings {
            match binding {
                ElementBinding::None(key, value) => {
                    if key == "style" {
                        match read_inline_css(&value) {
                            Ok(style) => element.style = style,
                            Err(error) => {
                                error!(
                                    "unable to parse inline style of {}, {error:?}",
                                    element.tag
                                );
                            }
                        }
                    }
                    if key == "id" {
                        self.static_id.insert(value.clone(), node);
                    }
                    element.attrs.insert(key, value);
                }
                ElementBinding::Tag(key, binder) => {
                    let path = self.schema.field(&binder, &mut self.locals);
                    let params = BindingParams::Tag(node, key.clone());
                    let binding = Binding {
                        params,
                        pipe: binder.pipe.clone(),
                    };
                    self.bindings.entry(path).or_default().push(binding);
                    element.style_hints.dynamic_attrs.insert(key);
                }
                ElementBinding::Attribute(key, text) => {
                    if let Some(value) = text.as_simple_text() {
                        warn!(
                            "element {} attribute {} has no bindings, you can just use HTML tag",
                            element.tag, key
                        );
                        element.attrs.insert(key, value);
                        continue;
                    }
                    let spans = text
                        .spans
                        .into_iter()
                        .enumerate()
                        .map(|(index, span)| match span {
                            TextSpan::String(span) => span.to_string(),
                            TextSpan::Binder(binder) => {
                                let path = self.schema.field(&binder, &mut self.locals);
                                let params = BindingParams::Attribute(node, key.clone(), index);
                                let binding = Binding {
                                    params,
                                    pipe: binder.pipe.clone(),
                                };
                                self.bindings.entry(path).or_default().push(binding);
                                binder.to_string()
                            }
                        })
                        .collect();
                    let attribute = TextContent::new(spans);
                    element.style_hints.dynamic_attrs.insert(key.clone());
                    if key == "class" {
                        element.style_hints.has_dynamic_classes = true;
                    }
                    if key == "id" {
                        element.style_hints.has_dynamic_id = true;
                    }
                    element.attrs.insert(key.clone(), attribute.to_string());
                    element.attrs_bindings.insert(key, attribute);
                }
                ElementBinding::Callback(event, arguments) => {
                    let arguments = arguments
                        .into_iter()
                        .map(|argument| match argument {
                            CallbackArgument::Keyword(key) => HandlerArgument::Keyword(key),
                            CallbackArgument::Event => HandlerArgument::Event,
                            CallbackArgument::Binder(binder) => {
                                let path = self.schema.field(&binder, &mut self.locals);
                                let pipe = binder.pipe.clone();
                                HandlerArgument::Binder { path, pipe }
                            }
                        })
                        .collect();
                    element.listeners.insert(event, Handler { arguments });
                }
                // used on other rendering stages
                ElementBinding::Alias(_, _) => {}
                ElementBinding::Repeat(_, _, _) => {}
                ElementBinding::Visibility(_, _) => {}
            }
        }
        let mut children = vec![];
        let mut hidden = vec![];
        match element.tag.as_str() {
            // void elements
            "img" => {
                children.extend(self.render_img(&mut element)?);
            }
            "input" => {}
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
                for child in template.children {
                    let child = if let Some((id, mut bindings)) = child.as_template_link() {
                        let mut template = self
                            .templates
                            .get(&id)
                            .ok_or(ViewError::TemplateNotFound(id))?
                            .clone();
                        // handle link bindings first
                        bindings.extend(template.bindings);
                        template.bindings = bindings;
                        template
                    } else {
                        child
                    };

                    let mut overridden = HashMap::new();
                    for binding in &child.bindings {
                        if let ElementBinding::Alias(name, binder) = binding {
                            let path = self.schema.field(binder, &mut self.locals);
                            overridden.insert(
                                name.to_string(),
                                self.locals.insert(name.to_string(), path),
                            );
                        }
                    }

                    if let Some((visible, binder)) = child.as_visibility() {
                        let path = self.schema.field(&binder, &self.locals);
                        let pipe = binder.pipe.clone();
                        let child_id = self.render_node(child)?;
                        children.push(child_id);
                        hidden.push(child_id);
                        let params = BindingParams::Visibility(node, child_id, visible);
                        let binding = Binding { params, pipe };
                        self.bindings.entry(path).or_default().push(binding);
                    } else if let Some((name, count, binder)) = child.as_repeat() {
                        let array = self.schema.field(binder, &self.locals);
                        let start = children.len();
                        let params = BindingParams::Repeat(node, start, count);
                        let binding = Binding {
                            params,
                            pipe: binder.pipe.clone(),
                        };
                        self.bindings
                            .entry(array.clone())
                            .or_default()
                            .push(binding);
                        let overridden = self.locals.remove(name);
                        for n in 0..count {
                            let path = self.schema.index(binder, n, &self.locals);
                            self.locals.insert(name.to_string(), path);
                            let child = child.clone();
                            let child = self.render_node(child)?;
                            children.push(child);
                        }
                        if let Some(overridden) = overridden {
                            self.locals.insert(name.to_string(), overridden);
                        } else {
                            self.locals.remove(name);
                        }
                    } else {
                        let child = self.render_node(child)?;
                        children.push(child);
                    }

                    for (key, value) in overridden {
                        if let Some(value) = value {
                            self.locals.insert(key, value);
                        } else {
                            self.locals.remove(&key);
                        }
                    }
                }
            }
        }
        element.children = children.clone();
        self.tree.set_node_context(node, Some(element))?;
        self.tree.set_children(node, &children)?;
        // todo:
        Ok(node)
    }
}

pub trait RendererTranslator {
    fn translate(&self, span: String) -> String;
}

pub struct FakeTranslator;

impl FakeTranslator {
    pub fn new() -> Box<dyn RendererTranslator> {
        Box::new(FakeTranslator)
    }
}

impl RendererTranslator for FakeTranslator {
    fn translate(&self, span: String) -> String {
        span
    }
}
