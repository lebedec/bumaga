use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "html/grammar.pest"]
struct HtmlParser {}

pub type Node = u32;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dom {
    pub pos: (usize, usize),
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub text: Option<String>,
    pub children: Vec<Dom>,
}

pub fn read_html_unchecked(html: &str) -> Dom {
    read_html(html).expect("must be read html")
}

pub fn read_html(html: &str) -> Result<Dom, ReaderError> {
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
fn parse_content(pair: Pair<Rule>) -> Dom {
    match pair.as_rule() {
        Rule::Element => {
            let pos = pair.line_col();
            let mut iter = pair.into_inner();
            let tag = iter.next().unwrap().as_str();
            let attrs = iter.next().unwrap();
            let children = iter.next().unwrap();
            Dom {
                pos,
                tag: tag.to_string(),
                attrs: parse_attrs(attrs),
                text: None,
                children: children.into_inner().map(parse_content).collect(),
            }
        }
        Rule::Text => {
            let pos = pair.line_col();
            let text = pair.as_str().trim().to_string();
            Dom {
                pos,
                tag: "".to_string(),
                attrs: Default::default(),
                text: Some(text),
                children: vec![],
            }
        }
        Rule::Void => {
            let pos = pair.line_col();
            let mut iter = pair.into_inner();
            let tag = iter.next().unwrap().as_str();
            let attrs = iter.next().unwrap();
            Dom {
                pos,
                tag: tag.to_string(),
                attrs: parse_attrs(attrs),
                text: None,
                children: vec![],
            }
        }
        _ => unreachable!(),
    }
}

fn parse_attrs(pair: Pair<Rule>) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Attribute => {
                let mut iter = pair.into_inner();
                let name = iter.next().unwrap().as_str();
                // empty attribute syntax is exactly equivalent to specifying the empty string
                // https://www.w3.org/TR/2012/WD-html-markup-20120329/syntax.html#syntax-attributes
                let value = iter.next().map(|value| value.as_str()).unwrap_or("");
                attrs.insert(name.to_string(), value.to_string());
            }
            _ => unreachable!(),
        }
    }
    attrs
}

#[cfg(test)]
mod tests {
    use crate::html::reader::read_html;
    use crate::html::Dom;
    use std::collections::HashMap;
    use std::time::Instant;

    impl Dom {
        pub fn attr(mut self, attr: &str, value: &str) -> Self {
            self.attrs.insert(attr.to_string(), value.to_string());
            self
        }
    }

    fn void(tag: &str) -> Dom {
        Dom {
            tag: tag.to_string(),
            attrs: Default::default(),
            text: None,
            children: vec![],
        }
    }

    fn el(tag: &str, children: Vec<Dom>) -> Dom {
        Dom {
            tag: tag.to_string(),
            attrs: Default::default(),
            text: None,
            children,
        }
    }

    fn txt(text: &str) -> Dom {
        Dom {
            tag: "".to_string(),
            attrs: Default::default(),
            text: Some(text.to_string()),
            children: vec![],
        }
    }

    fn tag(tag: &str, text: &str) -> Dom {
        Dom {
            tag: tag.to_string(),
            attrs: Default::default(),
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

    #[derive(Debug, Clone)]
    struct MyStruct<'s> {
        tag: &'s str,
        children: Vec<MyStruct<'s>>,
    }

    fn parse_something(data: &str) -> MyStruct {
        MyStruct {
            tag: &data[..3],
            children: vec![
                MyStruct {
                    tag: &data[3..6],
                    children: vec![],
                },
                MyStruct {
                    tag: &data[6..9],
                    children: vec![],
                },
            ],
        }
    }

    #[test]
    pub fn test_giga_html() {
        let res = parse_something("0123456789abcdef");
        println!("RES {:?}", res);

        let html = include_str!("giga.html");
        let t = Instant::now();
        let document = read_html(html).expect("valid document");
        fn collect(object: Dom, stats: &mut HashMap<String, usize>) {
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
