use serde_json::Value;

pub trait ValueExtensions {
    fn as_string(&self) -> String;
}

impl ValueExtensions for Value {
    fn as_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Bool(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => value.clone(),
            Value::Array(_) => String::from("[array]"),
            Value::Object(_) => String::from("[object]"),
        }
    }
}
