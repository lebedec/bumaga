use crate::css::{Complex, Matcher, Simple, Style};
use crate::Element;
use log::error;
use std::collections::{HashMap, HashSet};
use taffy::{NodeId, TaffyTree, TraversePartialTree};

pub fn match_style(css: &str, style: &Style, node: NodeId, tree: &TaffyTree<Element>) -> bool {
    style
        .selectors
        .iter()
        .any(|selector| match_complex_selector(css, selector, node, tree))
}

fn match_complex_selector(
    css: &str,
    selector: &Complex,
    node: NodeId,
    tree: &TaffyTree<Element>,
) -> bool {
    let mut target = node;
    selector
        .selectors
        .iter()
        .rev() // CSS components match order
        .all(|component| match component.as_combinator() {
            None => match_simple_selector(css, component, target, tree),
            Some(combinator) => find_next_target(combinator, &mut target, tree),
        })
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
    css: &str,
    component: &Simple,
    node: NodeId,
    tree: &TaffyTree<Element>,
) -> bool {
    let html = match tree.get_node_context(node) {
        Some(element) => &element.html,
        None => {
            error!("unable to match selector for node {node:?}, html context not found");
            return false;
        }
    };
    match component {
        Simple::All => true,
        Simple::Type(name) => html.tag.as_str() == name.as_str(css),
        Simple::Id(ident) => html
            .attrs
            .get("id")
            .map(|id| id.as_str() == ident.as_str(css))
            .unwrap_or(false),
        Simple::Class(ident) => html
            .attrs
            .get("class")
            .map(|classes| match_class(classes, ident.as_str(css)))
            .unwrap_or(false),
        Simple::Attribute(name, operator, value) => {
            let value = value.as_str(css);
            html.attrs
                .get(name.as_str(css))
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
        Simple::Root => tree.parent(node).is_none(),
        Simple::PseudoClass(name) => html.pseudo_classes.contains(name.as_str(css)),
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
