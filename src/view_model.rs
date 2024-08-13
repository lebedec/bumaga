use crate::{Element, Input, Output, ViewError};
use log::error;
use serde::de::Unexpected::Str;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap};
use taffy::{NodeId, TaffyTree};

pub type Bindings = BTreeMap<String, Vec<Binding>>;

pub trait Transformer: Fn(Value) -> Value {}

pub struct ViewModel {
    bindings: Bindings,
    model: Value,
    transformers: HashMap<String, Box<dyn Fn(Value) -> Value>>,
}

impl ViewModel {
    pub fn create(bindings: Bindings, model: Value) -> Self {
        Self {
            bindings,
            model,
            transformers: HashMap::new(),
        }
    }

    pub fn bind(&mut self, value: &Value) -> Vec<Reaction> {
        let mut reactions = vec![];
        Self::bind_value(&mut self.model, value, "", &self.bindings, &mut reactions);
        reactions
    }

    pub fn bind_value(
        mut dst: &mut Value,
        src: &Value,
        path: &str,
        bindings: &Bindings,
        reactions: &mut Vec<Reaction>,
    ) {
        match (&mut dst, src) {
            (Value::Array(current), Value::Array(next)) => {
                if current.len() != next.len() {
                    current.resize(next.len(), Value::Null);
                    Self::react(path, src, bindings, reactions);
                }
                for (index, dst) in current.iter_mut().enumerate() {
                    let src = &next[index];
                    let path = &format!("{path}[{index}]");
                    Self::bind_value(dst, src, path, bindings, reactions);
                }
            }
            (Value::Array(_), _) => {
                error!("unable to bind '{path}', must be array")
            }
            (Value::Object(object), Value::Object(src)) => {
                for (key, dst) in object.iter_mut() {
                    let src = match src.get(key) {
                        Some(src) => src,
                        None => {
                            error!("unable to bind '{path}', must be specified");
                            continue;
                        }
                    };
                    let path = if !path.is_empty() {
                        format!("{path}.{key}")
                    } else {
                        key.clone()
                    };
                    Self::bind_value(dst, src, &path, bindings, reactions);
                }
            }
            (Value::Object(_), _) => {
                error!("unable to bind '{path}', must be object")
            }
            (dst, src) => {
                if *dst != src {
                    **dst = src.clone();
                    Self::react(path, src, bindings, reactions);
                }
            }
        }
    }

    #[inline]
    fn react(path: &str, value: &Value, bindings: &Bindings, reactions: &mut Vec<Reaction>) {
        if let Some(bindings) = bindings.get(path) {
            for binding in bindings {
                reactions.push(binding.react_value_change(value))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Binding {
    Text(NodeId, usize),
    Visibility(NodeId, bool),
    Attribute(NodeId, String),
    Repeat(NodeId, usize, usize),
}

impl Binding {
    fn react_value_change(&self, value: &Value) -> Reaction {
        match self.clone() {
            Binding::Visibility(node, visible) => {
                let visible = as_boolean(value) == visible;
                Reaction::Reattach { node, visible }
            }
            Binding::Attribute(node, key) => {
                let value = as_string(value);
                Reaction::Bind { node, key, value }
            }
            Binding::Text(node, span) => {
                // TODO: interpolation ?!
                let text = as_string(value);
                Reaction::Type { node, span, text }
            }
            Binding::Repeat(parent, start, size) => {
                if let Some(value) = value.as_array() {
                    let count = value.len();
                    let count = if count > size {
                        error!("unable to repeat all items of {parent:?}");
                        size
                    } else {
                        count
                    };
                    Reaction::Repeat {
                        parent,
                        start,
                        cursor: start + count,
                        end: start + size,
                    }
                } else {
                    error!("unable to repeat, value must be array");
                    Reaction::Repeat {
                        parent,
                        start,
                        cursor: start,
                        end: start + size,
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Reaction {
    Type {
        node: NodeId,
        span: usize,
        text: String,
    },
    Reattach {
        node: NodeId,
        visible: bool,
    },
    Repeat {
        parent: NodeId,
        start: usize,
        cursor: usize,
        end: usize,
    },
    Bind {
        node: NodeId,
        key: String,
        value: String,
    },
}

pub fn as_boolean(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(number) => number.as_f64().map(|value| value != 0.0).unwrap_or(false),
        Value::String(string) => string.len() > 0,
        Value::Array(array) => array.len() > 0,
        Value::Object(_) => true,
    }
}

pub fn as_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => string.clone(),
        Value::Array(_) => "[array]".to_string(),
        Value::Object(_) => "{object}".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::html::read_html;
    use crate::rendering::Schema;
    use crate::view_model::ViewModel;
    use crate::Component;
    use serde_json::json;
    use std::collections::{BTreeMap, HashMap};
    use std::fs;
    use taffy::TaffyTree;

    #[test]
    pub fn test_something_vm() {
        let html = fs::read_to_string("./examples/shared/view.html").expect("html file exist");
        let html = read_html(&html).expect("html file valid");
        let mut tree = TaffyTree::new();
        let mut bindings = BTreeMap::new();
        let mut locals = HashMap::new();
        let mut schema = Schema::new();
        Component::render_node(html, &mut tree, &mut bindings, &mut locals, &mut schema)
            .expect("valid");
        println!("schema {:?}", schema.value);
        println!("bindings {:?}", bindings);

        let mut view_model = ViewModel::create(bindings, schema.value);
        let reactions = view_model.bind(&json!({
            "todo": "Hello world!",
            "todos": [
                "Todo A",
                "Todo B",
            ]
        }));
        println!("reactions: {reactions:?}");
    }
}
