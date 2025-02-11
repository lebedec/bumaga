use crate::css::ComputedValue::{Color, Number, Str};
use crate::css::{ComputedValue, Function};
use crate::styles::Cascade;
use log::error;

impl<'c> Cascade<'c> {
    pub(crate) fn compute_function(&self, function: &Function, shorthand: &mut Vec<ComputedValue>) {
        let name = function.name.as_str();
        let mut arguments = vec![];
        self.compute_shorthand(&function.arguments, &mut arguments);
        let computed_value = match (name, arguments.as_slice()) {
            ("rgb", [r, g, b]) => {
                Color([map_c(r), map_c(g), map_c(b), 255])
            }
            ("rgba", [r, g, b, a]) => {
                Color([map_c(r), map_c(g), map_c(b), map_a(a)])
            }
            // ("url", [Str(path)]) => Str(format!("{}/{}", self.resources, path)),
            ("url", [Str(path)]) => Str(path.to_string()),
            _ => {
                error!("unable to compute function {name}({arguments:?}), not supported");
                ComputedValue::Error
            }
        };
        shorthand.push(computed_value);
    }
}

#[inline(always)]
fn map_c(value: &ComputedValue) -> u8 {
    match value {
        ComputedValue::Zero => 0,
        ComputedValue::Percentage(value) => (*value * 255.0) as u8,
        Number(value) => *value as u8,
        _ => 0
    }
}

#[inline(always)]
fn map_a(value: &ComputedValue) -> u8 {
    match value {
        ComputedValue::Zero => 0,
        ComputedValue::Percentage(value) => (*value * 255.0) as u8,
        Number(value) => (*value * 255.0) as u8,
        _ => 0
    }
}
