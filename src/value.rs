use serde_json::Value;

pub trait ValueExtensions {
    fn as_string(&self) -> String;
    fn as_boolean(&self) -> bool;
}

impl ValueExtensions for Value {
    fn as_string(&self) -> String {
        match self {
            Value::Null => "".to_string(),
            Value::Bool(value) => value.to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(string) => string.clone(),
            Value::Array(_) => "[array]".to_string(),
            Value::Object(_) => "{object}".to_string(),
        }
    }

    fn as_boolean(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(value) => *value,
            Value::Number(number) => number.as_f64().map(|value| value != 0.0).unwrap_or(false),
            Value::String(string) => string.len() > 0,
            Value::Array(array) => array.len() > 0,
            Value::Object(_) => true,
        }
    }
}
