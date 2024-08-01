use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use taffy::TaffyTree;

#[derive(Parser)]
#[grammar = "html/grammar.pest"]
struct Html {}

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

#[derive(Debug)]
enum Content {
    Element {
        tag: String,
        attrs: HashMap<String, String>,
        children: Vec<Content>,
    },
    Text {
        tag: String,
        attrs: HashMap<String, String>,
        text: String,
    },
}

pub fn read_html(content: &str) -> Result<TaffyTree<Node>, ReaderError> {
    let document = Html::parse(Rule::Document, content)?
        .next()
        .ok_or(ReaderError::EmptyDocument)?;

    println!("DOC {:?}", document);
    let content = parse_content(document);
    println!("content {:?}", content);

    let tree = TaffyTree::new();
    Ok(tree)
}

/// NOTE:
/// Pest parser guarantees that pairs will contain only rules defined in grammar.
/// So, knowing the exact order of rules and it parameters we can unwrap iterators
/// without error handling. Macro unreachable! can be used for the same reason.
fn parse_content(pair: Pair<Rule>) -> Content {
    match pair.as_rule() {
        Rule::Element => {
            let mut iter = pair.into_inner();
            let tag = iter.next().unwrap().as_str();
            let attrs = iter.next().unwrap();
            let children = iter.next().unwrap();
            Content::Element {
                tag: tag.to_string(),
                attrs: parse_attrs(attrs),
                children: children.into_inner().map(parse_content).collect(),
            }
        }
        Rule::Text => {
            let mut iter = pair.into_inner();
            let tag = iter.next().unwrap().as_str();
            let attrs = iter.next().unwrap();
            let text = iter.next().unwrap().as_str();
            Content::Text {
                tag: tag.to_string(),
                attrs: parse_attrs(attrs),
                text: text.to_string(),
            }
        }
        Rule::Void => {
            let mut iter = pair.into_inner();
            let tag = iter.next().unwrap().as_str();
            let attrs = iter.next().unwrap();
            Content::Element {
                tag: tag.to_string(),
                attrs: parse_attrs(attrs),
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

    // https://www.w3.org/TR/2012/WD-html-markup-20120329/syntax.html#syntax-elements
    const VOID_TAGS: [&str; 16] = [
        "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
        "meta", "param", "source", "track", "wbr",
    ];

    #[test]
    pub fn test_simple_error() {
        let document = read_html("<");
        println!("{}", document.is_err());
        assert!(document.is_err());
    }

    #[test]
    pub fn test_no_childs_parsing() {
        let document = read_html("<div></div>").expect("valid document");
        assert_eq!(2, document.total_node_count());
    }

    #[test]
    pub fn test_one_child_parsing() {
        let document = read_html("<div><span>Hello world!</span></div>").expect("valid document");
        assert_eq!(2, document.total_node_count());
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
        assert_eq!(2, document.total_node_count());
    }

    #[test]
    pub fn test_text_parsing() {
        let document = read_html("<div>Hello world!</div>").expect("valid document");
        assert_eq!(2, document.total_node_count());
    }

    #[test]
    pub fn test_void_parsing() {
        for html in VOID_TAGS.map(|tag| format!("<{tag} />")) {
            let document = read_html(&html).expect("valid document");
            assert_eq!(2, document.total_node_count());
        }
    }

    #[test]
    pub fn test_void_one_attr_parsing() {
        let document = read_html(r#"<link href="..." />"#).expect("valid document");
        assert_eq!(2, document.total_node_count());
    }

    #[test]
    pub fn test_void_three_attr_parsing() {
        let html = r#"<link href="..." disabled *="single" />"#;
        let document = read_html(html).expect("valid document");
        assert_eq!(2, document.total_node_count());
    }

    pub fn test_complex_document_parsing() {
        let html = "
            <div>
                <link />
                <h1>Header</h1>
                <div></div>
            </div>
        ";
    }
}
