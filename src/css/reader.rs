use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "css/css.pest"]
struct CssParser {}

#[derive(Debug)]
pub enum ReaderError {
    Parsing(Error<Rule>),
    EmptyStyleSheet,
    Generic(String),
}

impl From<Error<Rule>> for ReaderError {
    fn from(error: Error<Rule>) -> Self {
        Self::Parsing(error)
    }
}

#[derive(Debug)]
pub struct MyStyle<'i> {
    selectors: Vec<MySelector<'i>>,
    declaration: Vec<Property<'i>>,
}

#[derive(Debug)]
pub struct Property<'i> {
    name: &'i str,
    value: &'i str,
}

#[derive(Debug)]
pub struct MyAnimation<'i> {
    name: &'i str,
    keyframes: Vec<MyKeyframe<'i>>,
}

#[derive(Debug)]
pub struct MyKeyframe<'i> {
    step: &'i str,
    declaration: Vec<Property<'i>>,
}

#[derive(Debug)]
pub struct MyPresentation<'i> {
    styles: Vec<MyStyle<'i>>,
    animations: Vec<MyAnimation<'i>>,
}

#[derive(Debug)]
pub struct MySelector<'i> {
    components: Vec<MyComponent<'i>>,
}

#[derive(Debug)]
pub enum MyComponent<'i> {
    Selector(&'i str),
    Combinator(&'i str),
}

impl MyComponent<'_> {
    pub fn as_combinator(&self) -> Option<&str> {
        match self {
            MyComponent::Combinator(combinator) => Some(combinator),
            _ => None,
        }
    }
}

// Used to optimize frequently used or complex values.
// At same time provides ease parsing.
pub enum CssValue {
    Inherit,
    Initial,
    Unset,
    Dimension { value: f32, unit: MyStr },
    Color { value: u32 },
    Unparsed(Span),
}

#[derive(Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy)]
struct MyStr {
    start: usize,
    end: usize,
}

pub struct Bundle {
    data: String,
    values: Vec<CssValue>,
}

impl Bundle {
    pub fn new() -> Self {
        Self {
            data: "abc123456789".to_string(),
            values: vec![
                CssValue::Initial,
                CssValue::Dimension {
                    value: 1.32,
                    unit: MyStr { start: 0, end: 4 },
                },
            ],
        }
    }

    pub fn string(&self, str: MyStr) -> &str {
        &self.data[str.start..str.end]
    }
}

fn do_something(bundle: &Bundle) {
    for value in &bundle.values {
        match value {
            CssValue::Inherit => {}
            CssValue::Initial => {}
            CssValue::Unset => {}
            CssValue::Dimension { value, unit } => {
                let unit = bundle.string(*unit);
                let v = match unit {
                    "px" => *value,
                    "rem" => *value * 16.0,
                    _ => *value,
                };
                println!("value {v} {unit}")
            }
            _ => {}
        }
    }
}

fn parse_declaration<'i>(pair: Pair<'i, Rule>) -> Vec<Property<'i>> {
    let mut declaration = vec![];
    for property in pair.into_inner() {
        let mut iter = property.into_inner();
        let name = iter.next().unwrap().as_str();
        let value = iter.next().unwrap().as_str();
        declaration.push(Property { name, value })
    }
    declaration
}

pub fn read_css(css: &str) -> Result<MyPresentation, ReaderError> {
    let stylesheet = CssParser::parse(Rule::StyleSheet, css)?
        .next()
        .ok_or(ReaderError::EmptyStyleSheet)?;
    let mut styles = vec![];
    let mut animations = vec![];
    for rule in stylesheet.into_inner() {
        match rule.as_rule() {
            Rule::Animation => {
                let mut iter = rule.into_inner();
                let name = iter.next().unwrap().as_str();
                let mut keyframes = vec![];
                for pair in iter {
                    let mut iter = pair.into_inner();
                    let step = iter.next().unwrap().as_str();
                    let declaration = parse_declaration(iter.next().unwrap());
                    keyframes.push(MyKeyframe { step, declaration })
                }
                animations.push(MyAnimation { name, keyframes })
            }
            Rule::Style => {
                let mut iter = rule.into_inner();
                let selectors_list = iter.next().unwrap();
                let mut selectors = vec![];
                for complex in selectors_list.into_inner() {
                    let mut components: Vec<MyComponent> = vec![];
                    for component in complex.into_inner() {
                        match component.as_rule() {
                            Rule::Compound => {
                                let is_descendant = components.len() > 0
                                    && components[components.len() - 1].as_combinator().is_none();
                                if is_descendant {
                                    components.push(MyComponent::Combinator(" "));
                                }
                                for simple in component.into_inner() {
                                    components.push(MyComponent::Selector(simple.as_str()))
                                }
                            }
                            Rule::Combinator => {
                                components.push(MyComponent::Combinator(component.as_str()))
                            }
                            _ => unreachable!(),
                        }
                    }
                    selectors.push(MySelector { components })
                }
                let declaration = parse_declaration(iter.next().unwrap());
                styles.push(MyStyle {
                    selectors,
                    declaration,
                })
            }
            _ => unreachable!(),
        }
    }
    Ok(MyPresentation { styles, animations })
}

#[cfg(test)]
mod tests {
    use crate::css::reader::{do_something, read_css, Bundle};
    use crate::styles::parse_presentation;
    use std::time::Instant;

    #[test]
    pub fn test_something() {
        let bundle = Bundle::new();
        do_something(&bundle)
    }

    #[test]
    pub fn test_simple_rule() {
        let css = r#"
        .myClass {
            background: red;
        }
        #myId {
            background: red;
        }
        div {
            background: red;
        }
        #myContainer > div > span {
            background: red;
        }
        .myA.myB {
            background: red;
        }
        .myA .myB {
            background: red;
        }
        input:focus {
            background: red;
        }
        dd:last-of-type {
            background: red;
        }
        di:last-child {
            background: red;
        }
        .todo[data-done="true"]:hover {
            background: red;
        }
        .todo:nth-child(even) {
            background: red;
        }

        @keyframes HeightAnimation {
            0% {
                height: 3rem;
                background-color: #394651;
            }
            50% {
                height: 4rem;
                background-color: green;
            }
            100% {
                height: 3rem;
                background-color: #394651;
            }
        }

        "#;
        let present = read_css(css).expect("must be valid");
        println!("{:?}", present);
        assert_eq!(11, present.styles.len());
        assert_eq!(1, present.animations.len())
    }

    #[test]
    pub fn test_giga_css() {
        let css = include_str!("giga.css");

        let t = Instant::now();
        let presentation = parse_presentation(css);
        println!("lightning CSS: {:?}", t.elapsed()); // ~ 6ms
        assert_eq!(90, presentation.rules.len());

        let t = Instant::now();
        let preset = read_css(css).expect("must be valid");
        println!("pest CSS (wip): {:?}", t.elapsed()); // ~ 5ms
        assert_eq!(90, preset.styles.len());
    }
}
