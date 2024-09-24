use crate::view_model::Binder;

use log::error;
use pest::error::Error;
use pest::iterators::Pair;

use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "html/html.pest"]
struct HtmlParser {}

#[derive(Debug)]
pub enum ReaderError {
    Parsing(Error<Rule>),
    EmptyDocument,
    Generic(String),
}

impl From<Error<Rule>> for ReaderError {
    fn from(error: Error<Rule>) -> Self {
        Self::Parsing(error)
    }
}

/// The Document Object Model (DOM) is an interface that treats an HTML document as a tree structure
/// wherein each node is an object representing a part of the document.
#[derive(Debug, Clone, PartialEq)]
pub struct Html {
    pub tag: String,
    pub bindings: Vec<ElementBinding>,
    pub text: Option<TextBinding>,
    pub children: Vec<Html>,
}

impl Html {
    pub fn empty() -> Self {
        Html {
            tag: "".to_string(),
            bindings: vec![],
            text: None,
            children: vec![],
        }
    }

    pub fn as_visibility(&self) -> Option<(bool, &Binder)> {
        for binding in &self.bindings {
            if let ElementBinding::Visibility(visible, binder) = binding {
                return Some((*visible, binder));
            }
        }
        None
    }

    pub fn as_repeat(&self) -> Option<(&str, usize, &Binder)> {
        for binding in &self.bindings {
            if let ElementBinding::Repeat(name, count, binder) = binding {
                return Some((name, *count, binder));
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementBinding {
    None(String, String),
    Alias(String, Binder),
    Attribute(String, Binder),
    Repeat(String, usize, Binder),
    Callback(String, String, Binder),
    Visibility(bool, Binder),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextBinding {
    pub spans: Vec<TextSpan>,
}

impl TextBinding {
    pub fn string(value: &str) -> Self {
        Self {
            spans: vec![TextSpan::String(value.to_string())],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextSpan {
    String(String),
    Binder(Binder),
}

pub fn read_html_unchecked(html: &str) -> Html {
    read_html(html).expect("must be read html")
}

pub fn read_html(html: &str) -> Result<Html, ReaderError> {
    let document = HtmlParser::parse(Rule::Document, html)?
        .next()
        .ok_or(ReaderError::EmptyDocument)?;
    let content = parse_content(document);
    Ok(content)
}

/// NOTE:
/// Pest parser guarantees that pairs will contain only rules defined in grammar.
/// So, knowing the exact order of rules and it parameters we can unwrap iterators
/// without error handling. Macro unreachable! can be used for the same reason.
fn parse_content(pair: Pair<Rule>) -> Html {
    match pair.as_rule() {
        Rule::Element => {
            let mut iter = pair.into_inner();
            let tag = iter.next().unwrap().as_str();
            let attrs = iter.next().unwrap();
            let children = iter.next().unwrap();
            let bindings = parse_element_bindings(attrs);
            let mut attrs = HashMap::new();
            for binding in &bindings {
                match binding {
                    ElementBinding::None(key, value) => {
                        attrs.insert(key.to_string(), value.to_string());
                    }
                    _ => {}
                }
            }
            Html {
                tag: tag.to_string(),
                bindings,
                text: None,
                children: children
                    .into_inner()
                    .map(|child| parse_content(child))
                    .collect(),
            }
        }
        Rule::Text => {
            let mut spans = vec![];
            for span in pair.into_inner() {
                match span.as_rule() {
                    Rule::String => spans.push(TextSpan::String(span.as_str().to_string())),
                    Rule::Binder => spans.push(TextSpan::Binder(parse_binder(span))),
                    _ => unreachable!(),
                }
            }
            let text = TextBinding { spans };
            Html {
                tag: "".to_string(),
                bindings: vec![],
                text: Some(text),
                children: vec![],
            }
        }
        Rule::Void => {
            let mut iter = pair.into_inner();
            let tag = iter.next().unwrap().as_str();
            let attrs = iter.next().unwrap();

            let bindings = parse_element_bindings(attrs);
            let mut attrs = HashMap::new();
            for binding in &bindings {
                match binding {
                    ElementBinding::None(key, value) => {
                        attrs.insert(key.to_string(), value.to_string());
                    }
                    _ => {}
                }
            }

            Html {
                tag: tag.to_string(),
                bindings,
                text: None,
                children: vec![],
            }
        }
        Rule::Script => Html {
            tag: "script".to_string(),
            bindings: vec![],
            text: None,
            children: vec![],
        },
        _ => unreachable!(),
    }
}

fn parse_binder(pair: Pair<Rule>) -> Binder {
    let mut path = vec![];
    let mut pipe = vec![];
    for next in pair.into_inner() {
        match next.as_rule() {
            Rule::Getter => {
                path = next
                    .into_inner()
                    .map(|key| key.as_str().to_string())
                    .collect();
            }
            Rule::Transformer => pipe.push(next.as_str().to_string()),
            _ => unreachable!(),
        }
    }
    Binder { path, pipe }
}

fn parse_element_bindings(pair: Pair<Rule>) -> Vec<ElementBinding> {
    let mut bindings = vec![];
    for pair in pair.into_inner() {
        let rule = pair.as_rule();
        let mut iter = pair.into_inner();
        let name = iter.next().unwrap().as_str().to_string();
        let binding = match rule {
            Rule::RepeatBinding => {
                let count = iter.next().unwrap().as_str();
                let count = count.parse::<usize>().unwrap_or_else(|error| {
                    error!("unable to parse repeat count {count}, {error}");
                    0
                });
                let binder = parse_binder(iter.next().unwrap());
                ElementBinding::Repeat(name, count, binder)
            }
            Rule::AliasBinding => {
                let binder = parse_binder(iter.next().unwrap());
                ElementBinding::Alias(name, binder)
            }
            Rule::AttributeBinding => {
                let binder = parse_binder(iter.next().unwrap());
                ElementBinding::Attribute(name, binder)
            }
            Rule::CallbackBinding => {
                let function = iter.next().unwrap().as_str().to_string();
                let binder = parse_binder(iter.next().unwrap());
                ElementBinding::Callback(name, function, binder)
            }
            Rule::VisibilityBinding => {
                let visible = name == "?";
                let binder = parse_binder(iter.next().unwrap());
                ElementBinding::Visibility(visible, binder)
            }
            Rule::DoubleQuoted => {
                let value = iter.next().unwrap().as_str().to_string();
                ElementBinding::None(name, value)
            }
            Rule::Unquoted => {
                let value = iter.next().unwrap().as_str().to_string();
                ElementBinding::None(name, value)
            }
            Rule::Empty => {
                // empty attribute syntax is exactly equivalent to specifying the empty string
                // https://www.w3.org/TR/2012/WD-html-markup-20120329/syntax.html#syntax-attributes
                ElementBinding::None(name, "".to_string())
            }
            _ => unreachable!(),
        };
        bindings.push(binding);
    }
    bindings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_binding_text_one_span() {
        let html = html(r#"<div>Hello, {name}</div>"#);
        assert_eq!(html.children[0].text, text(&[t("Hello, "), b("name")]))
    }

    #[test]
    pub fn test_binding_text_multiple_spans() {
        let html = html(r#"<div>Hello, {first} {last}</div>"#);
        let expected = text(&[t("Hello, "), b("first"), b("last")]);
        assert_eq!(html.children[0].text, expected)
    }

    #[test]
    pub fn test_binding_control_if() {
        let html = html(r#"<input ?={visible} />"#);
        assert_eq!(html.bindings, [if_("visible")])
    }

    #[test]
    pub fn test_binding_control_else() {
        let html = html(r#"<input !={visible} />"#);
        assert_eq!(html.bindings, [else_("visible")])
    }

    #[test]
    pub fn test_binding_attribute() {
        let html = html(r#"<input [value]={name} />"#);
        assert_eq!(html.bindings, [attr("value", "name")])
    }

    #[test]
    pub fn test_binding_attribute_style() {
        let html = html(r#"<input [style]="top: {pivot.x}px" />"#);
        assert_eq!(html.bindings, [attr("value", "name")])
    }

    #[test]
    pub fn test_binding_repeat() {
        let html = html(r#"<option {option}*10={options}></option>"#);
        assert_eq!(html.bindings, [repeat("option", 10, "options")])
    }

    #[test]
    pub fn test_binding_callback() {
        let html = html(r#"<input [onchange]~change={this} />"#);
        assert_eq!(html.bindings, vec![cb("onchange", "change", "this")])
    }

    #[test]
    pub fn test_binding_callback_name_with_underscore() {
        let html = html(r#"<input [onchange]~change_something={this} />"#);
        assert_eq!(
            html.bindings,
            vec![cb("onchange", "change_something", "this")]
        )
    }

    fn cb(event: &str, handler: &str, path: &str) -> ElementBinding {
        ElementBinding::Callback(
            event.to_string(),
            handler.to_string(),
            Binder {
                path: vec![path.to_string()],
                pipe: vec![],
            },
        )
    }

    fn repeat(name: &str, count: usize, path: &str) -> ElementBinding {
        ElementBinding::Repeat(
            name.to_string(),
            count,
            Binder {
                path: vec![path.to_string()],
                pipe: vec![],
            },
        )
    }

    fn attr(key: &str, path: &str) -> ElementBinding {
        ElementBinding::Attribute(
            key.to_string(),
            Binder {
                path: vec![path.to_string()],
                pipe: vec![],
            },
        )
    }

    fn if_(path: &str) -> ElementBinding {
        ElementBinding::Visibility(
            true,
            Binder {
                path: vec![path.to_string()],
                pipe: vec![],
            },
        )
    }

    fn else_(path: &str) -> ElementBinding {
        ElementBinding::Visibility(
            false,
            Binder {
                path: vec![path.to_string()],
                pipe: vec![],
            },
        )
    }

    fn text(spans: &[TextSpan]) -> Option<TextBinding> {
        Some(TextBinding {
            spans: spans.to_vec(),
        })
    }

    fn t(text: &str) -> TextSpan {
        TextSpan::String(text.to_string())
    }

    fn b(path: &str) -> TextSpan {
        TextSpan::Binder(Binder {
            path: vec![path.to_string()],
            pipe: vec![],
        })
    }

    fn html(html: &str) -> Html {
        read_html(html).expect("HTML valid and parsing complete")
    }
}
