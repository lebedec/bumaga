use crate::css::{Complex, Matcher, Simple, Style};
use crate::Element;
use log::error;

use taffy::{NodeId, TaffyTree};

pub fn match_style(
    style: &Style,
    node: NodeId,
    tree: &TaffyTree<Element>,
    matcher: &impl PseudoClassMatcher,
) -> bool {
    style
        .selectors
        .iter()
        .any(|selector| match_complex_selector(selector, node, tree, matcher))
}

fn match_complex_selector(
    selector: &Complex,
    node: NodeId,
    tree: &TaffyTree<Element>,
    matcher: &impl PseudoClassMatcher,
) -> bool {
    let mut target = node;
    let mut element = tree.get_node_context(target).unwrap();
    for component in selector.selectors.iter().rev() {
        match component.as_combinator() {
            None => {
                if !match_simple_selector(component, element, matcher) {
                    return false;
                }
            }
            Some(combinator) => {
                if !find_next_target(combinator, &mut target, tree) {
                    return false;
                } else {
                    element = tree.get_node_context(target).unwrap();
                }
            }
        }
    }
    true
}

fn find_next_target(combinator: char, target: &mut NodeId, tree: &TaffyTree<Element>) -> bool {
    let next = match combinator {
        '>' => tree.parent(*target),
        '+' => tree
            .parent(*target)
            .and_then(|parent| tree.children(parent).ok())
            .and_then(|children| {
                children
                    .iter()
                    .skip(1)
                    .position(|child| *child == *target)
                    .and_then(|child| children.get(child - 1).cloned())
            }),
        _ => {
            // TODO: multi-target selector match
            error!("combinator {combinator:?} not supported");
            None
        }
    };
    match next {
        None => false,
        Some(next) => {
            *target = next;
            true
        }
    }
}

fn match_simple_selector(
    component: &Simple,
    element: &Element,
    matcher: &impl PseudoClassMatcher,
) -> bool {
    match component {
        Simple::All => true,
        Simple::Type(name) => element.tag.as_str() == name.as_str(),
        Simple::Id(ident) => element
            .attrs
            .get("id")
            .map(|id| id.as_str() == ident.as_str())
            .unwrap_or(false),
        Simple::Class(ident) => element
            .attrs
            .get("class")
            .map(|classes| match_class(classes, ident.as_str()))
            .unwrap_or(false),
        Simple::Attribute(name, operator, value) => {
            let value = value.as_str();
            // println!(
            //     "OPERATOR {operator:?} [{name}] {}",
            //     element.attrs.contains_key(name)
            // );
            element
                .attrs
                .get(name.as_str())
                .map(|attr| match operator {
                    Matcher::Exist => true,
                    Matcher::Equal => attr == value,
                    Matcher::Include => attr.split(" ").any(|word| word == value),
                    Matcher::DashMatch => attr == value || attr == &format!("-{value}"),
                    Matcher::Prefix => attr.starts_with(value),
                    Matcher::Substring => attr.contains(value),
                    Matcher::Suffix => attr.ends_with(value),
                })
                .unwrap_or(false)
        }
        Simple::Root => element.tag == ":root",
        Simple::PseudoClass(name) => matcher.has_pseudo_class(element, name.as_str()),
        _ => {
            error!("selector {component:?} not supported");
            false
        }
    }
}

#[inline(always)]
fn match_class(classes: &str, ident: &str) -> bool {
    classes.split(" ").any(|class| class == ident)
}

pub trait PseudoClassMatcher {
    fn has_pseudo_class(&self, element: &Element, class: &str) -> bool;
}
