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

    pub fn as_template_link(&self) -> Option<(String, Vec<ElementBinding>)> {
        if self.tag == "link" {
            let mut bindings = vec![];
            let mut id = None;
            for binding in &self.bindings {
                if let ElementBinding::None(name, value) = binding {
                    if name == "href" {
                        id = Some(value.clone());
                        continue;
                    }
                }
                bindings.push(binding.clone());
            }
            if let Some(id) = id {
                return Some((id, bindings));
            }
        }
        None
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
    Tag(String, Binder),
    Attribute(String, TextBinding),
    Repeat(String, usize, Binder),
    Callback(String, Vec<CallbackArgument>),
    Visibility(bool, Binder),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallbackArgument {
    Keyword(String),
    Event,
    Binder(Binder),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextBinding {
    pub spans: Vec<TextSpan>,
}

impl TextBinding {
    #[inline(always)]
    pub fn as_simple_text(&self) -> Option<String> {
        match self.spans.as_slice() {
            [TextSpan::String(text)] => Some(text.to_string()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextSpan {
    String(String),
    Binder(Binder),
}

pub fn read_html(html: &str) -> Result<Html, ReaderError> {
    let document = HtmlParser::parse(Rule::Document, html)?
        .next()
        .ok_or(ReaderError::EmptyDocument)?;
    let content = parse_content(document, true);
    Ok(content)
}

/// NOTE:
/// Pest parser guarantees that pairs will contain only rules defined in grammar.
/// So, knowing the exact order of rules and it parameters we can unwrap iterators
/// without error handling. Macro unreachable! can be used for the same reason.
fn parse_content(pair: Pair<Rule>, is_last_content: bool) -> Html {
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
            let children: Vec<Pair<Rule>> = children.into_inner().collect();
            let children_count = children.len();
            Html {
                tag: tag.to_string(),
                bindings,
                text: None,
                children: children
                    .into_iter()
                    .enumerate()
                    .map(|(index, child)| parse_content(child, index + 1 == children_count))
                    .collect(),
            }
        }
        Rule::Text => {
            let mut prefetch = vec![];
            for span in pair.into_inner() {
                match span.as_rule() {
                    Rule::String => prefetch.push(TextSpan::String(span.as_str().to_string())),
                    Rule::Binder => prefetch.push(TextSpan::Binder(parse_binder(span))),
                    _ => unreachable!(),
                }
            }
            let count = prefetch.len();
            let mut spans = vec![];

            for index in 0..count {
                let _is_next_binding = if let Some(TextSpan::Binder(_)) = prefetch.get(index + 1) {
                    true
                } else {
                    false
                };
                let span = prefetch[index].clone();
                match span {
                    TextSpan::String(string) => {
                        let mut text = String::new();
                        for (index, fragment) in string.split("\n").enumerate() {
                            if index != 0 {
                                let fragment = fragment.trim_start();
                                if !fragment.is_empty() || !is_last_content {
                                    text.push(' ');
                                    text.push_str(fragment);
                                }
                            } else {
                                text.push_str(fragment);
                            }
                        }
                        if !text.is_empty() {
                            spans.push(TextSpan::String(text));
                        }
                    }
                    span => spans.push(span),
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
                    error!("unable to parse repeat count '{count}', {error}");
                    0
                });
                let binder = parse_binder(iter.next().unwrap());
                ElementBinding::Repeat(name, count, binder)
            }
            Rule::AliasBinding => {
                let binder = parse_binder(iter.next().unwrap());
                ElementBinding::Alias(name, binder)
            }
            Rule::TagBinding => {
                let binder = parse_binder(iter.next().unwrap());
                ElementBinding::Tag(name, binder)
            }
            Rule::AttributeBinding => {
                let mut spans = vec![];
                for span in iter {
                    match span.as_rule() {
                        Rule::DoubleQuotedAttributeString => {
                            spans.push(TextSpan::String(span.as_str().to_string()))
                        }
                        Rule::Binder => spans.push(TextSpan::Binder(parse_binder(span))),
                        _ => unreachable!(),
                    }
                }
                let text = TextBinding { spans };
                ElementBinding::Attribute(name, text)
            }
            Rule::CallbackBinding => {
                let mut arguments = vec![];
                for pair in iter {
                    let argument = match pair.as_rule() {
                        Rule::Key => CallbackArgument::Keyword(pair.as_str().to_string()),
                        Rule::Binder => CallbackArgument::Binder(parse_binder(pair)),
                        Rule::Event => CallbackArgument::Event,
                        _ => unreachable!(),
                    };
                    arguments.push(argument);
                }
                ElementBinding::Callback(name, arguments)
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
    use crate::testing::setup_tests_logging;

    #[test]
    pub fn test_ignore_script_tag() {
        let html = html(
            r#"<script>
                function doSomething() {}
                for (let i = 0; i < 27; i++) {
                    doSomething();
                }
            </script>"#,
        );
        assert_eq!(html.children.len(), 0);
    }

    #[test]
    pub fn test_parse_img_tag() {
        let html = html(r#"<img alt="member.png" src="./images/member.png"/>"#);
        assert_eq!(html.tag, "img");
    }

    #[test]
    pub fn test_binding_binder_with_whitespaces() {
        let html = html(r#"<div>{ name }</div>"#);
        assert_eq!(html.children[0].text, text(&[b("name")]))
    }

    #[test]
    pub fn test_binding_text_one_span() {
        let html = html(r#"<div>Hello, {name}</div>"#);
        assert_eq!(html.children[0].text, text(&[t("Hello, "), b("name")]))
    }

    #[test]
    pub fn test_binding_text_whitespace_before_sibling_element() {
        let html = html(r#"<div>Hello, {binding} <span>Element Text</span></div>"#);
        assert_eq!(
            html.children[0].text,
            text(&[t("Hello, "), b("binding"), t(" ")])
        )
    }

    #[test]
    pub fn test_binding_text_multiple_spans_with_whitespaces() {
        let html = html(r#"<div>Hello,  {first}  {last}</div>"#);
        let expected = text(&[t("Hello,  "), b("first"), t("  "), b("last")]);
        assert_eq!(html.children[0].text, expected)
    }

    #[test]
    pub fn test_binding_text_no_space() {
        let html = html(r#"<div>+{property}</div>"#);
        let expected = text(&[t("+"), b("property")]);
        assert_eq!(html.children[0].text, expected)
    }

    #[test]
    pub fn test_binding_text_multiple_spans_no_space() {
        let html = html(r#"<div>Hello, {first}{last}</div>"#);
        let expected = text(&[t("Hello, "), b("first"), b("last")]);
        assert_eq!(html.children[0].text, expected)
    }

    #[test]
    pub fn test_binding_text_ends_with_whitespace() {
        let html = html(r#"<div>Hello, {binding} </div>"#);
        assert_eq!(html.children[0].text, text(&[t("Hello, "), b("binding"), t(" ")]))
    }

    #[test]
    pub fn test_binding_text_multilines() {
        let html = html(
            r#"<div>
                Line 1
                Hello, {world}!
                Line 3
            </div>"#,
        );
        let expected = text(&[t("Line 1 Hello, "), b("world"), t("! Line 3")]);
        assert_eq!(html.children[0].text, expected)
    }

    #[test]
    pub fn test_binding_text_multilines_before_sibling() {
        let html = html(
            r#"<div>
                Line 1
                Hello, {world}!
                <div>something</div>
            </div>"#,
        );
        let expected = text(&[t("Line 1 Hello, "), b("world"), t("! ")]);
        assert_eq!(html.children[0].text, expected)
    }

    #[test]
    pub fn test_binding_text_empty_multilines() {
        let html = html(
            r#"<div>
                {world}
            </div>"#,
        );
        let expected = text(&[b("world")]);
        assert_eq!(html.children[0].text, expected)
    }

    #[test]
    pub fn test_binding_alias() {
        let html = html(r#"<input +option="{context.config.option}" />"#);
        assert_eq!(html.bindings, [al("option", "context.config.option")])
    }

    #[test]
    pub fn test_binding_tag() {
        let html = html(r#"<input #disabled="{disabled}" />"#);
        assert_eq!(html.bindings, [tag("disabled", "disabled")])
    }

    #[test]
    pub fn test_binding_control_if() {
        let html = html(r#"<input ?="{visible}" />"#);
        assert_eq!(html.bindings, [if_("visible")])
    }

    #[test]
    pub fn test_binding_control_else() {
        let html = html(r#"<input !="{visible}" />"#);
        assert_eq!(html.bindings, [else_("visible")])
    }

    #[test]
    pub fn test_binding_attribute() {
        let html = html(r#"<input @value="{name}" />"#);
        assert_eq!(html.bindings, [attr("value", &[b("name")])])
    }

    #[test]
    pub fn test_binding_attribute_style() {
        let html = html(r#"<input @styles="top: {pivot.x}px;" />"#);
        let expected = [attr("styles", &[t("top: "), b("pivot.x"), t("px;")])];
        assert_eq!(html.bindings, expected)
    }

    #[test]
    pub fn test_binding_attribute_style_multiple_properties() {
        let html = html(r#"<input @styles="width: {width}px; height: {height}px;" />"#);
        let expected = [attr(
            "styles",
            &[
                t("width: "),
                b("width"),
                t("px; height: "),
                b("height"),
                t("px;"),
            ],
        )];
        assert_eq!(html.bindings, expected)
    }

    #[test]
    pub fn test_binding_repeat() {
        let html = html(r#"<option *option="10 {options}"></option>"#);
        assert_eq!(html.bindings, [repeat("option", 10, "options")])
    }

    #[test]
    pub fn test_binding_repeat_single_number_count() {
        let html = html(r#"<div *effect="8 {effects}"> {effect} </div>"#);
        assert_eq!(html.bindings, [repeat("effect", 8, "effects")])
    }

    #[test]
    pub fn test_binding_repeat_with_shorthand() {
        let html = html(r#"<option *_="10 {options}"></option>"#);
        assert_eq!(html.bindings, [repeat("_", 10, "options")])
    }

    #[test]
    pub fn test_binding_callback_binding_argument() {
        let html = html(r#"<button ^onclick="{my_data}"></button>"#);
        let binding = ElementBinding::Callback(
            "onclick".into(),
            vec![CallbackArgument::Binder(binder("my_data"))],
        );
        assert_eq!(html.bindings, vec![binding])
    }

    #[test]
    pub fn test_binding_callback_event_argument() {
        let html = html(r#"<button ^onclick="MyMessage $event"></button>"#);
        let binding = ElementBinding::Callback(
            "onclick".into(),
            vec![
                CallbackArgument::Keyword("MyMessage".into()),
                CallbackArgument::Event,
            ],
        );
        assert_eq!(html.bindings, vec![binding])
    }

    fn repeat(name: &str, count: usize, path: &str) -> ElementBinding {
        ElementBinding::Repeat(name.to_string(), count, binder(path))
    }

    fn attr(key: &str, spans: &[TextSpan]) -> ElementBinding {
        ElementBinding::Attribute(
            key.to_string(),
            TextBinding {
                spans: spans.to_vec(),
            },
        )
    }

    fn al(name: &str, path: &str) -> ElementBinding {
        ElementBinding::Alias(name.to_string(), binder(path))
    }

    fn tag(name: &str, path: &str) -> ElementBinding {
        ElementBinding::Tag(name.to_string(), binder(path))
    }

    fn if_(path: &str) -> ElementBinding {
        ElementBinding::Visibility(true, binder(path))
    }

    fn else_(path: &str) -> ElementBinding {
        ElementBinding::Visibility(false, binder(path))
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
        TextSpan::Binder(binder(path))
    }

    fn binder(path: &str) -> Binder {
        Binder {
            path: path.split(".").map(ToString::to_string).collect(),
            pipe: vec![],
        }
    }

    fn html(html: &str) -> Html {
        setup_tests_logging();
        read_html(html).expect("HTML valid and parsing complete")
    }
}
