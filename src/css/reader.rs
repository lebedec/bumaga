use crate::css::model::{CssDimension, CssProperty, CssValue, CssVariable};
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
    name: CssProperty,
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
                    let declaration = read_declaration(iter.next().unwrap());
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
                let declaration = read_declaration(iter.next().unwrap());
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

fn read_declaration<'i>(pair: Pair<'i, Rule>) -> Vec<Property<'i>> {
    let mut declaration = vec![];
    for property in pair.into_inner() {
        let mut iter = property.into_inner();
        let name = iter.next().unwrap().as_span();

        let name = CssProperty::from(name);
        let v = iter.next().unwrap();

        let value = v.as_str();
        //println!("VALUE {value} {v:?}");
        declaration.push(Property { name, value })
    }
    declaration
}

fn read_value<'i>(pair: Pair<'i, Rule>) -> CssValue {
    match pair.as_rule() {
        Rule::Rgba => CssValue::Color(read_color(pair)),
        Rule::Rgb => CssValue::Color(read_color(pair)),
        Rule::Color => CssValue::Color(read_color(pair)),
        Rule::Zero => CssValue::Zero,
        Rule::Percentage => CssValue::Percentage(read_number(pair) / 100.0),
        Rule::Dimension => CssValue::Dimension(read_dimension(pair)),
        Rule::Number => CssValue::Number(read_number(pair)),
        Rule::Var => CssValue::Var(read_variable(pair)),
        Rule::Raw => match pair.as_str() {
            "inherit" => CssValue::Inherit,
            "initial" => CssValue::Initial,
            "unset" => CssValue::Unset,
            _ => CssValue::Raw(pair.as_span().into()),
        },
        _ => CssValue::Raw(pair.as_span().into()),
    }
}

fn read_dimension<'i>(pair: Pair<'i, Rule>) -> CssDimension {
    unimplemented!()
}

fn read_variable<'i>(pair: Pair<'i, Rule>) -> CssVariable {
    unimplemented!()
}

fn read_number<'i>(pair: Pair<'i, Rule>) -> f32 {
    unimplemented!()
}

fn read_color<'i>(pair: Pair<'i, Rule>) -> u32 {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use crate::css::reader::read_css;
    use crate::styles::parse_presentation;
    use std::time::Instant;

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
