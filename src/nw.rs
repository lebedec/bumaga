use log::error;
use serde::de::Unexpected::Str;
use serde_json::{Map, Value};

pub fn merge_value(mut dst: &mut Value, src: &Value, path: &str, changes: &mut Vec<String>) {
    match (&mut dst, src) {
        (Value::Null, Value::Null) => {}
        (Value::Bool(dst), Value::Bool(src)) => {
            if dst != src {
                changes.push(format!("{path} b {dst} => {src}"));
                *dst = *src;
            }
        }
        (Value::Number(dst), Value::Number(src)) => {
            if dst != src {
                changes.push(format!("{path} n {dst} => {src}"));
                *dst = src.clone();
            }
        }
        (Value::String(ref mut dst), Value::String(src)) => {
            if dst != src {
                changes.push(format!("{path} s {dst} => {src}"));
                *dst = src.clone();
            }
        }
        (&mut Value::Null, _)
        | (Value::Number(_), _)
        | (Value::String(_), _)
        | (Value::Bool(_), _) => {
            if src.is_object() || src.is_array() {
                error!("unable to merge, '{path}' must not be object or array");
                return;
            }
            changes.push(format!("{path} a {dst} => {src}"));
            *dst = src.clone();
        }
        (Value::Array(dst), Value::Array(src)) => {
            for (index, dst) in dst.iter_mut().enumerate() {
                let src = src.get(index).unwrap_or(&Value::Null);
                let path = &format!("{path}[{index}]");
                merge_value(dst, src, path, changes);
            }
        }
        (Value::Array(_), _) => {
            error!("unable to merge, '{path}' must be array");
        }
        (Value::Object(dst), Value::Object(src)) => {
            for (key, dst) in dst.iter_mut() {
                let src = src.get(key).unwrap_or(&Value::Null);
                let path = &format!("{path}.{key}");
                merge_value(dst, src, path, changes);
            }
        }
        (Value::Object(dst), Value::Null) => {
            for (key, dst) in dst.iter_mut() {
                let path = &format!("{path}.{key}");
                merge_value(dst, &Value::Null, path, changes);
            }
        }
        (Value::Object(_), _) => {
            error!("unable to merge, '{path}' must be object")
        }
    };
}

pub struct Controller {
    conditions: Vec<Condition>,
    spans: Vec<Editor>,
    repeats: Vec<Repeat>,
}

pub struct Repeat {
    id: usize,
    getter: String,
    current: Vec<bool>,
}

pub struct Condition {
    id: usize,
    getter: String,
    value: bool,
    test: bool,
}

pub struct Editor {
    id: usize,
    getter: String,
    text: String,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            conditions: vec![],
            spans: vec![],
            repeats: vec![],
        }
    }
}

struct MyInput {}

pub fn get_value<'a>(value: &'a Map<String, Value>, path: &str) -> Option<&'a Value> {
    unimplemented!()
}

pub fn as_boolean(value: &Value) -> bool {
    unimplemented!()
}

pub fn as_array(value: &Value) -> &Vec<Value> {
    unimplemented!()
}

pub fn as_string(value: &Value) -> &str {
    unimplemented!()
}

pub fn update(input: MyInput, value: &Map<String, Value>) {
    let mut controller = Controller::new();
    let mut reactions = vec![];
    for repeat in controller.repeats.iter_mut() {
        let value = match get_value(value, &repeat.getter) {
            None => continue,
            Some(value) => value,
        };
        let items = as_array(value);
        for (index, item) in items.iter().enumerate() {}
    }
    for condition in controller.conditions.iter_mut() {
        let value = match get_value(value, &condition.getter) {
            None => continue,
            Some(value) => value,
        };
        let value = as_boolean(value);
        if condition.value != value {
            if value == condition.test {
                reactions.push(Reaction::Show { id: condition.id })
            } else {
                reactions.push(Reaction::Hide { id: condition.id })
            }
            condition.value = value;
        }
    }
    for editor in controller.spans.iter_mut() {
        let value = match get_value(value, &editor.getter) {
            None => continue,
            Some(value) => value,
        };
        let text = as_string(value);
        if text != editor.text {
            editor.text = text.to_string();
            reactions.push(Reaction::Type {
                id: editor.id,
                text: text.to_string(),
            })
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

impl Reaction {
    pub fn is_style_changes(&self) -> bool {
        true
    }

    pub fn is_render_changes(&self) -> bool {
        match self {
            Reaction::Show { .. } => true,
            Reaction::Hide { .. } => true,
            Reaction::Repeat { .. } => true,
            _ => false,
        }
    }

    pub fn is_layout_changes(&self) -> bool {
        true
    }
}
