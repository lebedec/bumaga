use crate::css::model::{PropertyKey, Shorthand, Value, Var};
use crate::css::{
    Animation, Complex, Css, Dim, Function, Keyframe, Matcher, Property, Simple, Style, Units,
};
use log::error;
use pest::error::Error;
use pest::iterators::Pair;
use pest::{Parser, Span};
use pest_derive::Parser;
use std::collections::{BTreeMap, HashMap};

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

pub fn read_css(css: &str) -> Result<Css, ReaderError> {
    let stylesheet = CssParser::parse(Rule::StyleSheet, css)?
        .next()
        .ok_or(ReaderError::EmptyStyleSheet)?;
    let mut styles = vec![];
    let mut animations = HashMap::new();
    for rule in stylesheet.into_inner() {
        match rule.as_rule() {
            Rule::Animation => {
                let mut iter = rule.into_inner();
                let name = iter.next().unwrap();
                let mut keyframes: HashMap<PropertyKey, Keyframe> = HashMap::new();
                for pair in iter {
                    let mut iter = pair.into_inner();
                    let step = iter.next().unwrap();
                    let step = match step.as_rule() {
                        Rule::Percentage => read_number(step.into_inner().next().unwrap()) as u32,
                        Rule::Keyword => match step.as_str() {
                            "from" => 0,
                            "to" => 100,
                            keyword => {
                                error!("incorrect keyframe step {keyword}");
                                0
                            }
                        },
                        _ => unreachable!(),
                    };
                    let declaration = read_declaration(iter.next().unwrap());
                    for property in declaration {
                        let keyframe = keyframes.entry(property.key).or_insert_with(|| Keyframe {
                            key: property.key,
                            frames: BTreeMap::new(),
                        });
                        // TODO: support multiple value, eliminate clone?
                        keyframe.frames.insert(step, property.get_first_shorthand());
                    }
                }
                animations.insert(
                    name.as_str().to_string(),
                    Animation {
                        keyframes: keyframes.into_values().collect(),
                    },
                );
            }
            Rule::Style => {
                let mut iter = rule.into_inner();
                let selectors_list = iter.next().unwrap();
                let mut selectors = vec![];
                for complex in selectors_list.into_inner() {
                    let mut components: Vec<Simple> = vec![];
                    for component in complex.into_inner() {
                        match component.as_rule() {
                            Rule::Compound => {
                                let is_descendant = components.len() > 0
                                    && components[components.len() - 1].as_combinator().is_none();
                                if is_descendant {
                                    components.push(Simple::Combinator(' '));
                                }
                                for simple in component.into_inner() {
                                    let simple_rule = simple.as_rule();
                                    let mut iter = simple.into_inner();
                                    let ident = iter
                                        .next()
                                        .map(|pair| pair.as_str().to_string())
                                        .unwrap_or(String::new());
                                    let component = match simple_rule {
                                        Rule::All => Simple::All,
                                        Rule::Id => Simple::Id(ident),
                                        Rule::Class => Simple::Class(ident),
                                        Rule::Type => Simple::Type(ident),
                                        Rule::Attribute => {
                                            let matcher =
                                                iter.next().map(|pair| pair.as_str()).unwrap_or("");
                                            let matcher = match matcher {
                                                "" => Matcher::Exist,
                                                "=" => Matcher::Equal,
                                                "~=" => Matcher::Include,
                                                "|=" => Matcher::DashMatch,
                                                "^=" => Matcher::Prefix,
                                                "$=" => Matcher::Suffix,
                                                "*=" => Matcher::Substring,
                                                _ => unreachable!(),
                                            };
                                            let search = iter
                                                .next()
                                                .map(|pair| match pair.as_rule() {
                                                    Rule::String => pair
                                                        .into_inner()
                                                        .next()
                                                        .unwrap()
                                                        .as_str()
                                                        .to_string(),
                                                    Rule::Ident => pair.as_str().to_string(),
                                                    _ => unreachable!(),
                                                })
                                                .unwrap_or(String::new());
                                            Simple::Attribute(ident, matcher, search)
                                        }
                                        Rule::PseudoClass => Simple::PseudoClass(ident),
                                        Rule::Root => Simple::Root,
                                        Rule::PseudoElement => Simple::PseudoElement(ident),
                                        _ => unreachable!(),
                                    };
                                    components.push(component)
                                }
                            }
                            Rule::Combinator => components.push(Simple::Combinator(
                                component.as_str().chars().next().unwrap(),
                            )),
                            _ => unreachable!(),
                        }
                    }
                    selectors.push(Complex {
                        selectors: components,
                    })
                }
                let declaration = read_declaration(iter.next().unwrap());
                styles.push(Style {
                    selectors,
                    declaration,
                })
            }
            _ => unreachable!(),
        }
    }
    Ok(Css {
        source: css.to_string(),
        styles,
        animations,
    })
}

fn read_declaration(pair: Pair<Rule>) -> Vec<Property> {
    let mut declaration = vec![];
    for property in pair.into_inner() {
        let mut iter = property.into_inner();
        let name = iter.next().unwrap();
        let shorthands = iter.next().unwrap();

        // println!("PROP {} {values:?}", name.as_str());
        let id = name.as_span().start();
        let name = name.as_str();
        let key = match PropertyKey::parse(name) {
            Some(key) => key,
            None => {
                error!("unable to read property {name}, not supported");
                continue;
            }
        };
        let values = shorthands
            .into_inner()
            .map(|value| read_shorthand(value))
            .collect();
        declaration.push(Property { id, key, values })
    }
    declaration
}

fn read_shorthand(pair: Pair<Rule>) -> Shorthand {
    pair.into_inner().map(read_value).collect()
}

fn read_value(pair: Pair<Rule>) -> Value {
    match pair.as_rule() {
        Rule::Keyword => Value::Keyword(pair.as_str().to_string()),
        Rule::Rgba => Value::Color(read_color(pair)),
        Rule::Rgb => Value::Color(read_color(pair)),
        Rule::Color => Value::Color(read_color(pair)),
        Rule::Zero => Value::Zero,
        Rule::Time => Value::Time(read_seconds(pair)),
        Rule::Percentage => Value::Percentage(read_percentage(pair)),
        Rule::Dimension => Value::Dimension(read_dimension(pair)),
        Rule::Number => Value::Number(read_number(pair)),
        Rule::Var => Value::Var(read_variable(pair)),
        Rule::Calc => Value::Unparsed(pair.as_str().to_string()),
        Rule::String => Value::String(pair.into_inner().next().unwrap().as_str().to_string()),
        Rule::Function => {
            let mut iter = pair.into_inner();
            let name = iter.next().unwrap().as_str().to_string();
            let args = iter.next().unwrap();
            let mut iter = args.into_inner();
            let mut arguments = vec![];
            while let Some(arg) = iter.next() {
                arguments.push(read_value(arg));
            }
            Value::Function(Function { name, arguments })
        }
        Rule::Raw => match pair.as_str() {
            "inherit" => Value::Inherit,
            "initial" => Value::Initial,
            "unset" => Value::Unset,
            _ => {
                // println!("RAW {}", pair.as_str());
                Value::Unparsed(pair.as_str().to_string())
            }
        },
        _ => unreachable!(),
    }
}

fn read_dimension(pair: Pair<Rule>) -> Dim {
    let mut iter = pair.into_inner();
    let number = iter.next().unwrap();
    let unit = iter.next().unwrap().as_str();
    Dim {
        value: read_number(number),
        unit: Units::parse(unit).unwrap_or_else(|| {
            error!("unable to read dimension unit {unit}, not supported");
            Units::Px
        }),
    }
}

fn read_percentage(pair: Pair<Rule>) -> f32 {
    let mut iter = pair.into_inner();
    let value = read_number(iter.next().unwrap());
    value / 100.0
}

fn read_seconds(pair: Pair<Rule>) -> f32 {
    let mut iter = pair.into_inner();
    let value = read_number(iter.next().unwrap());
    let unit = iter.next().unwrap().as_str();
    match unit {
        "s" => value,
        "ms" => value / 1000.0,
        _ => unreachable!(),
    }
}

fn read_variable(pair: Pair<Rule>) -> Var {
    let mut iter = pair.into_inner();
    let name = iter.next().unwrap();
    let fallback = iter.next();
    Var {
        name: name.as_str().to_string(),
        fallback: fallback.map(|pair| pair.as_str().to_string()),
    }
}

fn read_number(pair: Pair<Rule>) -> f32 {
    let number = pair.as_str();
    number.parse::<f32>().unwrap_or_else(|error| {
        error!("unable to parse number value {number}, {error}");
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
            if value.starts_with("rgb") {
                let mut iter = pair.into_inner();
                let r: u8 = iter.next().unwrap().as_str().parse().unwrap_or(0);
                let g: u8 = iter.next().unwrap().as_str().parse().unwrap_or(0);
                let b: u8 = iter.next().unwrap().as_str().parse().unwrap_or(0);
                let a: f32 = iter
                    .next()
                    .map(|a| a.as_str().parse().unwrap_or(1.0))
                    .unwrap_or(1.0);
                let a = (255.0 * a.max(0.0).min(1.0)) as u8;
                [r, g, b, a]
            } else {
                error!("unable to parse color {value}");
                [255; 4]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style_values<const N: usize>(css: &Css) -> [Value; N] {
        let mut values = [const { Value::Unset }; N];
        for i in 0..N {
            values[i] = css.styles[0].declaration[i].values[0][0].clone()
        }
        values
    }

    fn style_selectors(css: &Css) -> Vec<&Simple> {
        css.styles[0].selectors[0].selectors.iter().collect()
    }

    #[test]
    pub fn test_zero_value() {
        let css = "div { left: 0; width: 0; }";
        let css = read_css(css).expect("valid css");
        let [left, width] = style_values(&css);
        assert_eq!(left, Value::Zero, "left");
        assert_eq!(width, Value::Zero, "width");
    }

    #[test]
    pub fn test_percent_value() {
        let css = "div { width: 0%; border-radius: 50%;}";
        let css = read_css(css).expect("valid css");
        let [width, radius] = style_values(&css);
        assert_eq!(width, Value::Percentage(0.0), "width");
        assert_eq!(radius, Value::Percentage(0.5), "radius");
    }

    #[test]
    pub fn test_string_value() {
        let css = r#"div { content: "abc"; }"#;
        let css = read_css(css).expect("valid css");
        let [content] = style_values(&css);
        if let Value::String(value) = content {
            assert_eq!(value, "abc", "string literal");
        } else {
            assert!(false, "value type")
        }
    }

    #[test]
    pub fn test_string_matcher() {
        let css = r#"[data-something="abc"] {}"#;
        let css = read_css(css).expect("valid css");
        let selectors = style_selectors(&css);
        if let Simple::Attribute(key, matcher, value) = selectors[0] {
            assert_eq!(key, "data-something", "key");
            assert_eq!(*matcher, Matcher::Equal, "matcher");
            assert_eq!(value, "abc", "value");
        } else {
            assert!(false, "selector type")
        }
    }

    #[test]
    pub fn test_ident_matcher() {
        let css = r#"[data-something=abc] {}"#;
        let css = read_css(css).expect("valid css");
        let selectors = style_selectors(&css);
        if let Simple::Attribute(key, matcher, value) = selectors[0] {
            assert_eq!(key, "data-something", "key");
            assert_eq!(*matcher, Matcher::Equal, "matcher");
            assert_eq!(value, "abc", "value");
        } else {
            assert!(false, "selector type")
        }
    }

    #[test]
    pub fn test_animation_shorthand() {
        let css = "div { animation: 1s linear HeightAnimation; }";
        let css = read_css(css).expect("valid css");

        let animation = &css.styles[0].declaration[0].values[0];

        assert_eq!(animation.len(), 3);
    }

    #[test]
    pub fn test_simple_keyframes() {
        let css = r#"
            @keyframes HeightAnimation {
                0% {
                    line-height: 1.0;
                }
                50% {
                    line-height: 2.0;
                }
                100% {
                    line-height: 3.0;
                }
            }
        "#;
        let css = read_css(css).expect("valid css");

        let animation = Animation {
            keyframes: vec![Keyframe {
                key: PropertyKey::LineHeight,
                frames: BTreeMap::from([
                    (0, vec![Value::Number(1.0)]),
                    (50, vec![Value::Number(2.0)]),
                    (100, vec![Value::Number(3.0)]),
                ]),
            }],
        };
        let animations = HashMap::from([("HeightAnimation".to_string(), animation)]);

        assert_eq!(css.animations, animations);
    }

    // #[test]
    // pub fn test_root_selector() {
    //     let css = r#"
    //
    //     [dir=rtl] .next {
    //         float: left;
    //         right: unset;
    //         left: var(--page-padding);
    //     }
    //
    //     /* Use the correct buttons for RTL layouts*/
    //     [dir=rtl] .previous i.fa-angle-left:before {
    //         content: "\f105";
    //     }
    //
    //
    //     :root {
    //         right: calc(var(--sidebar-resize-indicator-width) * -1);
    //         transform: rotate(20deg) translate(30px, 20px) rotate(var(--my-var));
    //         content: "\f105";
    //         transform: rotate(var(--my-var));
    //         height: calc(10px - 10px);
    //         background: rgba(0, 0, 0, 0);
    //
    //     }"#;
    //     let present = read_css(css).expect("must be valid");
    // }
    //
    // #[test]
    // pub fn test_simple_rule() {
    //     let css = r#"
    //     .myClass {
    //
    //         top: 0 !important;
    //         background-color: rgba(0, 0, 0, 0);
    //         background: red solid;
    //         /*
    //         margin: auto calc(0px - var(--page-padding));
    //         right: calc(var(--sidebar-resize-indicator-width) * -1);
    //         width: calc(var(--sidebar-resize-indicator-width) - var(--sidebar-resize-indicator-space))
    //         */
    //         position: -webkit-sticky;
    //         transition: color 0.5s;
    //         margin-block-end: -1px;
    //
    //     }
    //     #myId {
    //         background: red;
    //     }
    //     div {
    //         background: red;
    //     }
    //     #myContainer > div > span {
    //         background: red;
    //     }
    //     .myA.myB {
    //         background: red;
    //     }
    //     .myA .myB {
    //         background: red;
    //     }
    //     input:focus {
    //         background: red;
    //     }
    //     dd:last-of-type {
    //         background: red;
    //     }
    //     di:last-child {
    //         background: red;
    //     }
    //     .todo[data-done="true"]:hover {
    //         background: red;
    //     }
    //     .todo:nth-child(even) {
    //         background: red;
    //     }
    //
    //     @keyframes HeightAnimation {
    //         0% {
    //             height: 3rem;
    //             background-color: #394651;
    //         }
    //         50% {
    //             height: 4rem;
    //             background-color: green;
    //         }
    //         100% {
    //             height: 3rem;
    //             background-color: #394651;
    //         }
    //     }
    //
    //     "#;
    //
    //     println!("CssShorthand {}", std::mem::size_of::<Shorthand>());
    //     println!("CssValue {}", std::mem::size_of::<Value>());
    //
    //     let present = read_css(css).expect("must be valid");
    //
    //     println!("{:?}", present.styles);
    //     assert_eq!(11, present.styles.len());
    //     assert_eq!(1, present.animations.len())
    // }
    //
    // #[test]
    // pub fn test_giga_css() {
    //     let css = include_str!("giga.css");
    //
    //     // let t = Instant::now();
    //     // let presentation = parse_presentation(css);
    //     // println!("lightning CSS: {:?}", t.elapsed()); // ~ 6ms
    //     // assert_eq!(90, presentation.rules.len());
    //
    //     let t = Instant::now();
    //     let preset = read_css(css).expect("must be valid");
    //     println!("pest CSS (wip): {:?}", t.elapsed()); // ~ 5ms
    //     assert_eq!(90, preset.styles.len());
    //
    //     for rul in preset.styles {
    //         for pr in rul.declaration {
    //             // println!("{:?}: {:?}", pr.name, pr.values.as_single())
    //         }
    //     }
    // }
}
