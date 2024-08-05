use crate::css::model::{
    CssDimension, CssProperty, CssShorthand, CssValue, CssValues, CssVariable,
};
use log::error;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::num::ParseFloatError;

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
    declaration: Vec<Property>,
}

#[derive(Debug)]
pub struct Property {
    name: CssProperty,
    values: CssValues,
}

#[derive(Debug)]
pub struct MyAnimation<'i> {
    name: &'i str,
    keyframes: Vec<MyKeyframe<'i>>,
}

#[derive(Debug)]
pub struct MyKeyframe<'i> {
    step: &'i str,
    declaration: Vec<Property>,
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

fn read_declaration(pair: Pair<Rule>) -> Vec<Property> {
    let mut declaration = vec![];
    for property in pair.into_inner() {
        let mut iter = property.into_inner();
        let name = iter.next().unwrap();
        let values = iter.next().unwrap();

        // println!("PROP {} {values:?}", name.as_str());

        let name = CssProperty::from(name.as_span());
        let mut shorthands: Vec<CssShorthand> = values.into_inner().map(read_shorthand).collect();
        let values = if shorthands.len() == 1 {
            CssValues::One(shorthands.remove(0))
        } else {
            CssValues::Multiple(shorthands)
        };

        declaration.push(Property { name, values })
    }
    declaration
}

fn read_shorthand(pair: Pair<Rule>) -> CssShorthand {
    let values: Vec<CssValue> = pair.into_inner().map(read_value).collect();
    match values.len() {
        1 => CssShorthand::N1(values[0]),
        2 => CssShorthand::N2(values[0], values[1]),
        3 => CssShorthand::N3(values[0], values[1], values[2]),
        4 => CssShorthand::N4(values[0], values[1], values[2], values[3]),
        _ => CssShorthand::N(values),
    }
}

fn read_value(pair: Pair<Rule>) -> CssValue {
    match pair.as_rule() {
        Rule::Keyword => CssValue::Keyword(pair.as_span().into()),
        Rule::Rgba => CssValue::Color(read_color(pair)),
        Rule::Rgb => CssValue::Color(read_color(pair)),
        Rule::Color => CssValue::Color(read_color(pair)),
        Rule::Zero => CssValue::Zero,
        Rule::Percentage => CssValue::Percentage(read_number(pair) / 100.0),
        Rule::Dimension => CssValue::Dimension(read_dimension(pair)),
        Rule::Number => CssValue::Number(read_number(pair)),
        Rule::Var => CssValue::Var(read_variable(pair)),
        Rule::Calc => CssValue::Raw(pair.as_span().into()),
        Rule::Raw => match pair.as_str() {
            "inherit" => CssValue::Inherit,
            "initial" => CssValue::Initial,
            "unset" => CssValue::Unset,
            _ => {
                // println!("RAW {}", pair.as_str());
                CssValue::Raw(pair.as_span().into())
            }
        },
        _ => unreachable!(),
    }
}

fn read_dimension(pair: Pair<Rule>) -> CssDimension {
    let mut iter = pair.into_inner();
    let number = iter.next().unwrap();
    let unit = iter.next().unwrap();
    CssDimension {
        value: read_number(number),
        unit: unit.as_span().into(),
    }
}

fn read_variable(pair: Pair<Rule>) -> CssVariable {
    let mut iter = pair.into_inner();
    let name = iter.next().unwrap();
    let fallback = iter.next();
    CssVariable {
        name: name.as_span().into(),
        fallback: fallback.map(|pair| pair.as_span().into()),
    }
}

fn read_number(pair: Pair<Rule>) -> f32 {
    let number = pair.as_str();
    number.parse::<f32>().unwrap_or_else(|error| {
        error!("unable to parse dimension value {number}, {error}");
        0.0
    })
}

fn read_color(pair: Pair<Rule>) -> [u8; 4] {
    let value = pair.as_str();
    match value.len() {
        7 if value.starts_with("#") => {
            let r = u8::from_str_radix(&value[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&value[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&value[5..7], 16).unwrap_or(0);
            let a = 255;
            [r, g, b, a]
        }
        9 if value.starts_with("#") => {
            let r = u8::from_str_radix(&value[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&value[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&value[5..7], 16).unwrap_or(0);
            let a = u8::from_str_radix(&value[7..9], 16).unwrap_or(0);
            [r, g, b, a]
        }
        _ => {
            error!("unable to parse color {value}");
            [255; 4]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::css::model::{CssShorthand, CssValue};
    use crate::css::reader::read_css;
    use crate::styles::parse_presentation;
    use std::time::Instant;

    #[test]
    pub fn test_simple_rule() {
        let css = r#"
        .myClass {

            top: 0 !important;
            background-color: rgba(0, 0, 0, 0);
            background: red solid;
            margin: auto calc(0px - var(--page-padding));
            right: calc(var(--sidebar-resize-indicator-width) * -1);
            width: calc(var(--sidebar-resize-indicator-width) - var(--sidebar-resize-indicator-space))
            position: -webkit-sticky;
            transition: color 0.5s;
            margin-block-end: -1px;

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

        println!("CssShorthand {}", std::mem::size_of::<CssShorthand>());
        println!("CssValue {}", std::mem::size_of::<CssValue>());

        let present = read_css(css).expect("must be valid");

        let v = present.styles[0].declaration[0]
            .values
            .as_single()
            .as_keyword()
            .map(|span| span.as_str(css));
        println!("BACKGROUND: {:?}", v);

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

        for rul in preset.styles {
            for pr in rul.declaration {
                // println!("{:?}: {:?}", pr.name, pr.values.as_single())
            }
        }
    }
}
