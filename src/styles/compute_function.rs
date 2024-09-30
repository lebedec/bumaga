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
            ("rgb", [Number(r), Number(g), Number(b)]) => {
                Color([*r as u8, *g as u8, *b as u8, 255])
            }
            ("rgba", [Number(r), Number(g), Number(b), Number(a)]) => {
                Color([*r as u8, *g as u8, *b as u8, (a * 255.0) as u8])
            }
            ("url", [Str(path)]) => Str(format!("{}/{}", self.resources, path)),
            _ => {
                error!("unable to compute function {name}({arguments:?}), not supported");
                ComputedValue::Error
            }
        };
        shorthand.push(computed_value);
    }
}
