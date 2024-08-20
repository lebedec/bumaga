use serde_json::Value;
use std::num::ParseIntError;

pub trait ValueExtensions {
    fn eval_array(&self) -> Vec<String>;
    fn eval_u64(&self) -> u64;
    fn eval_string(&self) -> String;
    fn as_boolean(&self) -> bool;
}

impl ValueExtensions for Value {
    fn eval_array(&self) -> Vec<String> {
        match self {
            Value::Array(array) => array.iter().map(|value| value.eval_string()).collect(),
            _ => vec![self.eval_string()],
        }
    }

    fn eval_u64(&self) -> u64 {
        match self {
            Value::Null => 0,
            Value::Bool(value) => {
                if *value {
                    1
                } else {
                    0
                }
            }
            Value::Number(number) => match number.as_f64() {
                None => 0,
                Some(number) => number as u64,
            },
            Value::String(string) => string.parse::<u64>().unwrap_or(0),
            Value::Array(_) => 0,
            Value::Object(_) => 0,
        }
    }

    fn eval_string(&self) -> String {
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
