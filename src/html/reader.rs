use crate::view_model::Reaction::Bind;
use log::error;
use pest::error::Error;
use pest::iterators::Pair;
use pest::pratt_parser::Op;
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

#[derive(Debug, Clone, PartialEq)]
pub enum TextSpan {
    String(String),
    Binder(Binder),
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub target: String,
    pub path: Vec<String>,
    pub pipe: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binder {
    pub path: Vec<String>,
    pub pipe: Vec<String>,
}

impl Binder {
    pub fn to_string(&self) -> String {
        let path = self.path.join(".");
        if self.pipe.len() > 0 {
            let pipe = self.pipe.join(" | ");
            format!("{{ {path} | {pipe} }}")
        } else {
            format!("{{ {path} }}")
        }
    }
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
    use crate::html::reader::read_html;
    use crate::html::Html;
    use std::collections::HashMap;
    use std::time::Instant;

    impl Html {
        pub fn attr(mut self, attr: &str, value: &str) -> Self {
            self.attrs.insert(attr.to_string(), value.to_string());
            self
        }
    }

    static mut COUNTER: usize = 0;

    fn index() -> usize {
        unsafe {
            COUNTER += 1;
            COUNTER
        }
    }

    fn void(tag: &str) -> Html {
        Html {
            index: index(),
            tag: tag.to_string(),
            attrs: Default::default(),
            bindings: vec![],
            text: None,
            text: None,
            children: vec![],
        }
    }

    fn el(tag: &str, children: Vec<Html>) -> Html {
        Html {
            index: index(),
            tag: tag.to_string(),
            attrs: Default::default(),
            bindings: vec![],
            text: None,
            text: None,
            children,
        }
    }

    fn txt(text: &str) -> Html {
        Html {
            index: index(),
            tag: "".to_string(),
            attrs: Default::default(),
            bindings: vec![],
            text: Some(text.to_string()),
            text: None,
            children: vec![],
        }
    }

    fn tag(tag: &str, text: &str) -> Html {
        Html {
            index: index(),
            tag: tag.to_string(),
            attrs: Default::default(),
            bindings: vec![],
            text: None,
            text: None,
            children: vec![txt(text)],
        }
    }

    // https://www.w3.org/TR/2012/WD-html-markup-20120329/syntax.html#syntax-elements
    const VOID_TAGS: [&str; 16] = [
        "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
        "meta", "param", "source", "track", "wbr",
    ];

    #[test]
    pub fn test_simple_error() {
        let document = read_html("<");
        assert!(document.is_err());
    }

    #[test]
    pub fn test_no_childs_parsing() {
        let document = read_html("<div></div>").expect("valid document");
        assert_eq!(document, el("div", vec![]))
    }

    #[test]
    pub fn test_one_child_parsing() {
        let document = read_html("<div><span>Hello world!</span></div>").expect("valid document");
        assert_eq!(
            document,
            el("div", vec![el("span", vec![txt("Hello world!")])])
        )
    }

    #[test]
    pub fn test_three_child_parsing() {
        let html = "
            <div>
                <link />
                <h1>Header</h1>
                <div></div>
            </div>
        ";
        let document = read_html(html).expect("valid document");
        assert_eq!(
            document,
            el(
                "div",
                vec![
                    void("link"),
                    el("h1", vec![txt("Header")]),
                    el("div", vec![])
                ]
            )
        )
    }

    #[test]
    pub fn test_text_parsing() {
        let document = read_html("<div>Hello world!</div>").expect("valid document");
        assert_eq!(document, el("div", vec![txt("Hello world!")]))
    }

    #[test]
    pub fn test_combined_content_parsing() {
        let html = "
            <div>
                String
                <h1>Header</h1>
                <link />
            </div>
        ";
        let document = read_html(html).expect("valid document");
        assert_eq!(
            document,
            el(
                "div",
                vec![txt("String"), el("h1", vec![txt("Header")]), void("link")]
            )
        )
    }

    #[test]
    pub fn test_void_parsing() {
        for tag in VOID_TAGS {
            let document = read_html(&format!("<{tag} />")).expect("valid document");
            assert_eq!(document, void(tag));
            let document = read_html(&format!("<{tag}>")).expect("valid document");
            assert_eq!(document, void(tag));
        }
    }

    #[test]
    pub fn test_void_one_attr_parsing() {
        let document = read_html(r#"<link href="..." />"#).expect("valid document");
        assert_eq!(document, void("link").attr("href", "..."))
    }

    #[test]
    pub fn test_void_three_attr_parsing() {
        let html = r#"<link href="..." disabled *="single" />"#;
        let document = read_html(html).expect("valid document");
        assert_eq!(
            document,
            void("link")
                .attr("href", "...")
                .attr("disabled", "")
                .attr("*", "single")
        );
    }

    #[test]
    pub fn test_giga_html() {
        let html = include_str!("giga.html");
        let t = Instant::now();
        let document = read_html(html).expect("valid document");
        fn collect(object: Html, stats: &mut HashMap<String, usize>) {
            *stats.entry(object.tag.clone()).or_insert(0) += 1;
            for child in object.children {
                collect(child, stats);
            }
        }
        let mut stats = HashMap::new();
        collect(document, &mut stats);
        assert!(100 > t.elapsed().as_millis(), "parsing time (ms)");
        println!("el: {:?}", t.elapsed());
        assert_eq!(Some(&139), stats.get("div"), "div elements");
        assert_eq!(Some(&5), stats.get("input"), "input elements");
        assert_eq!(Some(&302), stats.get(""), "text nodes")
    }

    #[test]
    pub fn test_complex_document_parsing() {
        let html = r#"
            <html>
            <meta http-equiv="content-type" content="text/html; charset=UTF-8">
            <link href="style.css" rel="stylesheet" />
            <body>
            <div class="panel">
                <header>
                    Bumaga Todo
                    <span>Streamline Your Day, the Bumaga Way!</span>
                </header>
                <div *="todos" class="todo" data-done="todos|done" onclick="finish(todos)">
                    <span>{todos}</span>
                    <div>×</div>
                </div>
                <input value="todo" oninput="update" onchange="append"/>
            </div>
            </body>
            </html>"#;
        let document = read_html(html).expect("valid document");

        assert_eq!(
            document,
            el(
                "html",
                vec![
                    void("meta")
                        .attr("http-equiv", "content-type")
                        .attr("content", "text/html; charset=UTF-8"),
                    void("link")
                        .attr("href", "style.css")
                        .attr("rel", "stylesheet"),
                    el(
                        "body",
                        vec![el(
                            "div",
                            vec![
                                el(
                                    "header",
                                    vec![
                                        txt("Bumaga Todo"),
                                        tag("span", "Streamline Your Day, the Bumaga Way!")
                                    ]
                                ),
                                el("div", vec![tag("span", "{todos}"), tag("div", "×")])
                                    .attr("*", "todos")
                                    .attr("class", "todo")
                                    .attr("data-done", "todos|done")
                                    .attr("onclick", "finish(todos)"),
                                void("input")
                                    .attr("value", "todo")
                                    .attr("oninput", "update")
                                    .attr("onchange", "append")
                            ]
                        )
                        .attr("class", "panel")]
                    )
                ]
            )
        )
    }
}
