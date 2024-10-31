use crate::css::model::{ComputedValue, PropertyKey, Shorthand};
use crate::css::{
    Animation, Complex, Css, Declaration, Definition, Dim, Function, Keyframe, Matcher, Property,
    Simple, Style, Units, Variable,
};
use log::error;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

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

pub fn read_inline_css(block: &str) -> Result<Vec<Declaration>, ReaderError> {
    let block = CssParser::parse(Rule::Declarations, block)?
        .next()
        .ok_or(ReaderError::EmptyStyleSheet)?;
    Ok(read_declarations(block))
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
                let mut keyframes = vec![];
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
                    let decls = iter.next().unwrap().into_inner().next().unwrap();
                    let declaration = read_declarations(decls);
                    keyframes.push(Keyframe { step, declaration });
                }
                let name = name.as_str().to_string();
                animations.insert(name.clone(), Animation { name, keyframes });
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

                let decls = iter.next().unwrap().into_inner().next().unwrap();
                let declaration = read_declarations(decls);
                styles.push(Style {
                    selectors,
                    declaration,
                })
            }
            _ => unreachable!(),
        }
    }
    Ok(Css { styles, animations })
}

fn read_declarations(pair: Pair<Rule>) -> Vec<Declaration> {
    let mut declarations = vec![];
    for property in pair.into_inner() {
        let mut iter = property.into_inner();
        let name = iter.next().unwrap();
        let shorthands = iter.next().unwrap();
        // println!("PROP {} {values:?}", name.as_str());
        let key = name.as_str();
        let declaration = if key.starts_with("--") {
            let values: Vec<Shorthand> = shorthands
                .into_inner()
                .map(|value| read_shorthand(value))
                .collect();
            Declaration::Variable(Variable {
                key: key.to_string(),
                shorthand: values[0].clone(),
            })
        } else {
            let key = match PropertyKey::parse(key) {
                Some(key) => key,
                None => {
                    error!("unable to read property {key}, not supported");
                    continue;
                }
            };
            let values = shorthands
                .into_inner()
                .map(|value| read_shorthand(value))
                .collect();
            Declaration::Property(Property { key, values })
        };
        declarations.push(declaration)
    }
    declarations
}

fn read_shorthand(pair: Pair<Rule>) -> Shorthand {
    pair.into_inner().map(read_value_def).collect()
}

fn read_value_def(pair: Pair<Rule>) -> Definition {
    match pair.as_rule() {
        Rule::Var => {
            let mut iter = pair.into_inner();
            let name = iter.next().unwrap().as_str().to_string();
            Definition::Var(name)
        }
        Rule::Function => {
            let mut iter = pair.into_inner();
            let name = iter.next().unwrap().as_str().to_string();
            let mut arguments = vec![];
            while let Some(arg) = iter.next() {
                arguments.push(read_value_def(arg));
            }
            Definition::Function(Function { name, arguments })
        }
        Rule::Explicit => {
            Definition::Explicit(read_explicit_value(pair.into_inner().next().unwrap()))
        }
        _ => unreachable!(),
    }
}

fn read_explicit_value(pair: Pair<Rule>) -> ComputedValue {
    match pair.as_rule() {
        Rule::Keyword => ComputedValue::Keyword(pair.as_str().to_string()),
        Rule::Color => ComputedValue::Color(read_color(pair)),
        Rule::Zero => ComputedValue::Zero,
        Rule::Time => ComputedValue::Time(read_seconds(pair)),
        Rule::Percentage => ComputedValue::Percentage(read_percentage(pair)),
        Rule::Dimension => ComputedValue::Dimension(read_dimension(pair)),
        Rule::Number => ComputedValue::Number(read_number(pair)),
        Rule::String => ComputedValue::Str(pair.into_inner().next().unwrap().as_str().to_string()),
        _ => unreachable!(),
    }
}

fn read_dimension(pair: Pair<Rule>) -> Dim {
    let mut iter = pair.into_inner();
    let number = iter.next().unwrap();
    let unit = iter.next().unwrap().as_str();
    let value = read_number(number);
    let unit = Units::parse(unit).unwrap_or_else(|| {
        error!("unable to read dimension unit {unit}, not supported");
        Units::Px
    });
    Dim::new(value, unit)
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

    #[test]
    pub fn test_inline_style_one_property() {
        let css = style("width: 0;");
        assert_eq!(css, [prop(PropertyKey::Width, &[zero()])])
    }

    #[test]
    pub fn test_inline_style_no_semicolumn() {
        let css = style("width: 0");
        assert!(css.is_empty());
    }

    #[test]
    pub fn test_inline_style_two_properties() {
        let css = style("width: 0; height: 0;");
        assert_eq!(
            css,
            [
                prop(PropertyKey::Width, &[zero()]),
                prop(PropertyKey::Height, &[zero()])
            ]
        )
    }

    #[test]
    pub fn test_component_value_var() {
        let css = css("div { background-color: var(--main-bg-color); }");
        assert_eq!(css.first_short(), [var("--main-bg-color")]);
    }

    #[test]
    pub fn test_component_value_shorthand() {
        let css = css("div { padding: 40px 30px; }");
        assert_eq!(css.first_short(), [px(40), px(30)]);
    }

    #[test]
    pub fn test_component_value_shorthand_var() {
        let css = css("div { padding: 40px var(--padding); }");
        assert_eq!(css.first_short(), [px(40), var("--padding")]);
    }

    #[test]
    pub fn test_component_value_list() {
        let css = css("div { font-family: monospaced, sans-serif; }");
        assert_eq!(css.first_short(), [kw("monospaced")]);
    }

    #[test]
    pub fn test_component_value_list_with_var() {
        let css = css("div { font-family: var(--default-font), sans-serif; }");
        assert_eq!(css.first_short(), [var("--default-font")]);
    }

    #[test]
    pub fn test_component_value_list_with_shorthands() {
        let css = css("div { box-shadow: -1em 0 0.4em olive, 3px 3px red; }");
        assert_eq!(css.first_short(), [em(-1.0), zero(), em(0.4), kw("olive")]);
    }

    #[test]
    pub fn test_component_value_list_with_shorthands_with_var() {
        let css = css("div { box-shadow:  -1em 0 0.4em var(--color), 3px 3px red; }");
        assert_eq!(
            css.first_short(),
            [em(-1.0), zero(), em(0.4), var("--color")]
        );
    }

    #[test]
    pub fn test_component_value_function_url_string() {
        let css = css(r#"div { mask-image: url("masks.svg#mask1"); }"#);
        assert_eq!(css.first_short(), &[func("url", &[s("masks.svg#mask1")])]);
    }

    #[test]
    pub fn test_component_value_function_single_value() {
        let css = css("div { transform: translateX(20px); }");
        assert_eq!(css.first_short(), &[func("translateX", &[px(20)])]);
    }

    #[test]
    pub fn test_component_value_function_single_value_var() {
        let css = css("div { transform: translateX(var(--offset)); }");
        assert_eq!(css.first_short(), &[func("translateX", &[var("--offset")])]);
    }

    #[test]
    pub fn test_component_value_function_multiple_values() {
        let css = css("div { transform: translate(20px, 40px); }");
        assert_eq!(css.first_short(), &[func("translate", &[px(20), px(40)])]);
    }

    #[test]
    pub fn test_component_value_function_multiple_values_var() {
        let css = css("div { transform: translate(var(--offset), 40px); }");
        assert_eq!(
            css.first_short(),
            &[func("translate", &[var("--offset"), px(40)])]
        );
    }

    #[test]
    pub fn test_component_value_rgba() {
        let css = css("div { background-color: rgba(133, 155, 155, 0.5); }");
        let expected = &[func("rgba", &[n(133), n(155), n(155), f(0.5)])];
        assert_eq!(css.first_short(), expected);
    }

    #[test]
    pub fn test_zero_value() {
        let css = css("div { padding: 0; }");
        assert_eq!(css.first_short(), &[zero()]);
    }

    #[test]
    pub fn test_zero_percent_value() {
        let css = css("div { border-radius: 0%;}");
        assert_eq!(css.first_short(), &[perc(0.0)]);
    }

    #[test]
    pub fn test_percent_value() {
        let css = css("div { border-radius: 50%;}");
        assert_eq!(css.first_short(), &[perc(0.5)]);
    }

    #[test]
    pub fn test_string_value() {
        let css = css(r#"div { content: "abc"; }"#);
        assert_eq!(css.first_short(), &[s("abc")]);
    }

    #[test]
    pub fn test_matcher_exist() {
        let css = css(r#"[data-something] {}"#);
        let selectors = style_selectors(&css);

        assert_eq!(
            selectors[0],
            &Simple::Attribute("data-something".to_string(), Matcher::Exist, "".to_string())
        );
    }

    #[test]
    pub fn test_matcher_equal_string() {
        let css = css(r#"[data-something="abc"] {}"#);
        let selectors = style_selectors(&css);
        assert_eq!(
            selectors[0],
            &Simple::Attribute(
                "data-something".to_string(),
                Matcher::Equal,
                "abc".to_string()
            )
        );
    }

    #[test]
    pub fn test_matcher_equal_ident() {
        let css = css(r#"[data-something=abc] {}"#);
        let selectors = style_selectors(&css);
        assert_eq!(
            selectors[0],
            &Simple::Attribute(
                "data-something".to_string(),
                Matcher::Equal,
                "abc".to_string()
            )
        );
    }

    #[test]
    pub fn test_animation_shorthand() {
        let css = css("div { animation: 1s linear HeightAnimation; }");
        let expected = &[ts(1.0), kw("linear"), kw("HeightAnimation")];
        assert_eq!(css.first_short(), expected);
    }

    #[test]
    pub fn test_animation_simple_keyframes() {
        let css = css(r#"
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
        "#);

        let animation = Animation {
            name: "HeightAnimation".to_string(),
            keyframes: vec![
                Keyframe {
                    step: 0,
                    declaration: vec![prop(PropertyKey::LineHeight, &[f(1.0)])],
                },
                Keyframe {
                    step: 50,
                    declaration: vec![prop(PropertyKey::LineHeight, &[f(2.0)])],
                },
                Keyframe {
                    step: 100,
                    declaration: vec![prop(PropertyKey::LineHeight, &[f(3.0)])],
                },
            ],
        };
        let animations = HashMap::from([("HeightAnimation".to_string(), animation)]);

        assert_eq!(css.animations, animations);
    }

    fn style_selectors(css: &Css) -> Vec<&Simple> {
        css.styles[0].selectors[0].selectors.iter().collect()
    }

    trait TestCss {
        fn first_property(&self) -> (PropertyKey, &[Definition]);
        fn first_short(&self) -> &[Definition];
    }

    impl TestCss for Css {
        fn first_property(&self) -> (PropertyKey, &[Definition]) {
            match &self.styles[0].declaration[0] {
                Declaration::Variable(_) => {
                    panic!("first declaration not property");
                }
                Declaration::Property(property) => (property.key, &property.values[0]),
            }
        }

        fn first_short(&self) -> &[Definition] {
            self.first_property().1
        }
    }

    fn style(css: &str) -> Vec<Declaration> {
        read_inline_css(css).expect("inline CSS valid and parsing complete")
    }

    fn css(css: &str) -> Css {
        read_css(css).expect("CSS valid and parsing complete")
    }

    fn prop(key: PropertyKey, shorthand: &[Definition]) -> Declaration {
        Declaration::Property(Property {
            key,
            values: vec![shorthand.to_vec()],
        })
    }

    fn px(value: i32) -> Definition {
        Definition::Explicit(ComputedValue::Dimension(Dim::new(value as f32, Units::Px)))
    }

    fn em(value: f32) -> Definition {
        Definition::Explicit(ComputedValue::Dimension(Dim::new(value, Units::Em)))
    }

    fn var(value: &str) -> Definition {
        Definition::Var(value.to_string())
    }

    fn kw(value: &str) -> Definition {
        Definition::Explicit(ComputedValue::Keyword(value.to_string()))
    }

    fn f(value: f32) -> Definition {
        Definition::Explicit(ComputedValue::Number(value))
    }

    fn n(value: i32) -> Definition {
        Definition::Explicit(ComputedValue::Number(value as f32))
    }

    fn zero() -> Definition {
        Definition::Explicit(ComputedValue::Zero)
    }

    fn perc(value: f32) -> Definition {
        Definition::Explicit(ComputedValue::Percentage(value))
    }

    fn func(name: &str, arguments: &[Definition]) -> Definition {
        Definition::Function(Function {
            name: name.to_string(),
            arguments: arguments.to_vec(),
        })
    }

    fn s(value: &str) -> Definition {
        Definition::Explicit(ComputedValue::Str(value.to_string()))
    }

    fn ts(value: f32) -> Definition {
        Definition::Explicit(ComputedValue::Time(value))
    }
}
