use crate::css::{match_style, read_css, Css};
use crate::{Component, Element};
use taffy::{NodeId, TaffyTree, TraversePartialTree};

impl Component {
    pub fn query(&self, selector: &str) -> Option<&Element> {
        unimplemented!()
        // let selector = read_css(selector).expect("must be valid css");
        // fn search(tree: &TaffyTree<Element>, node: NodeId, selector: &Css) -> Option<NodeId> {
        //     let css = &selector.source;
        //     for style in &selector.styles {
        //         if match_style(css, &style, node, tree) {
        //             return Some(node);
        //         }
        //     }
        //     for child in tree.child_ids(node) {
        //         if let Some(node) = search(tree, child, selector) {
        //             return Some(node);
        //         }
        //     }
        //     None
        // }
        // search(&self.tree, self.root, &selector).and_then(|id| self.tree.get_node_context(id))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Component, Element, Input};

    fn document(html: &str) -> Component {
        let mut component = Component::compile(html, "", "");
        let input = Input::new();
        component.update(input);
        component
    }

    trait ElementAssert {
        fn tag(&self) -> Option<&str>;
        fn id(&self) -> Option<&str>;
    }

    impl ElementAssert for Option<&Element> {
        fn tag(&self) -> Option<&str> {
            self.map(|element| element.html.tag.as_str())
        }

        fn id(&self) -> Option<&str> {
            self.and_then(|element| element.html.attrs.get("id").map(|value| value.as_str()))
        }
    }

    #[test]
    pub fn test_match_root_element() {
        let document = document("<div><span>Content</span></div>");
        let element = document.query(":root {}");
        assert_eq!(Some(":root"), element.tag())
    }

    #[test]
    pub fn test_match_id() {
        let document = document(r#"<div><a id="link"><span>Content</span></a></div>"#);
        let element = document.query("#link {}");
        assert_eq!(Some("a"), element.tag())
    }

    #[test]
    pub fn test_match_type() {
        let document = document(r#"<div><a id="link"><span>Content</span></a></div>"#);
        let element = document.query("a {}");
        assert_eq!(Some("link"), element.id())
    }

    #[test]
    pub fn test_match_class() {
        let document = document(r#"<div><a class="link"><span>Content</span></a></div>"#);
        let element = document.query(".link {}");
        assert_eq!(Some("a"), element.tag())
    }

    #[test]
    pub fn test_match_attribute() {
        let document = document(r#"<div><a hidden><span>Content</span></a></div>"#);
        let element = document.query("[hidden] {}");
        assert_eq!(Some("a"), element.tag())
    }
}
