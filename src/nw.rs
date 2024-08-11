use log::error;
use serde::de::Unexpected::Str;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap};

type Bindings = BTreeMap<String, Vec<Binding>>;

trait Transformer: Fn(Value) -> Value {}

pub struct ViewModel {
    bindings: Bindings,
    state: Value,
    transformers: HashMap<String, Box<dyn Fn(Value) -> Value>>,
}

impl ViewModel {
    pub fn create(bindings: Bindings, state: Value) -> Self {
        Self {
            bindings,
            state,
            transformers: HashMap::new(),
        }
    }

    pub fn bind(&mut self, value: &Value) -> Vec<String> {
        let mut reactions = vec![];
        Self::bind_value(&mut self.state, value, "", &self.bindings, &mut reactions);
        reactions
    }

    pub fn bind_value(
        mut dst: &mut Value,
        src: &Value,
        path: &str,
        bindings: &Bindings,
        reactions: &mut Vec<String>,
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
                    let path = &format!("{path}.{key}");
                    Self::bind_value(dst, src, path, bindings, reactions);
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
    fn react(path: &str, value: &Value, bindings: &Bindings, reactions: &mut Vec<String>) {
        if let Some(bindings) = bindings.get(path) {
            for binding in bindings {
                reactions.push(binding.react_value_change(value))
            }
        }
    }
}

pub enum Binding {
    Text(usize, usize),
    Visibility(usize, bool),
    Attribute(usize, String),
    Repeat(usize, usize, usize),
}

impl Binding {
    fn react_value_change(&self, value: &Value) -> String {
        match self {
            Binding::Visibility(element, is_visible) => {
                if as_boolean(value) == *is_visible {
                    format!("show el{element}")
                } else {
                    format!("hide el{element}")
                }
            }
            Binding::Attribute(element, key) => {
                format!("set el{element} attribute {key}={value}")
            }
            Binding::Text(element, n) => {
                // TODO: interpolation ?!
                let value = as_string(value);
                format!("interpolate el{element} text arg{n}={value}")
            }
            Binding::Repeat(parent, start, size) => {
                if let Some(value) = value.as_array() {
                    let count = value.len();
                    let count = if count > *size {
                        error!("unable to repeat all items of {parent}");
                        *size
                    } else {
                        count
                    };
                    format!("repeat p{parent} {start}..{count}..{size}")
                } else {
                    error!("unable to repeat, value must be array");
                    format!("repeat p{parent} {start}..{start}..{size}")
                }
            }
        }
    }
}

pub enum Reaction {
    Type {
        id: usize,
        text: String,
    },
    Show {
        id: usize,
    },
    Hide {
        id: usize,
    },
    Repeat {
        id: usize,
    },
    Bind {
        id: usize,
        key: String,
        value: String,
    },
    Style {
        id: usize,
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
