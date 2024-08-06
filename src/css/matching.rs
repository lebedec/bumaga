use crate::css::{MyComponent, MyMatcher, MySelector, MyStyle};
use crate::Element;
use log::error;
use std::collections::{HashMap, HashSet};
use taffy::{NodeId, TaffyTree, TraversePartialTree};

pub fn match_style(css: &str, style: &MyStyle, node: NodeId, tree: &TaffyTree<Element>) -> bool {
    style
        .selectors
        .iter()
        .any(|selector| match_complex_selector(css, selector, node, tree))
}

fn match_complex_selector(
    css: &str,
    selector: &MySelector,
    node: NodeId,
    tree: &TaffyTree<Element>,
) -> bool {
    let mut target = node;
    selector
        .components
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
    component: &MyComponent,
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
        MyComponent::All => true,
        MyComponent::Type(name) => html.tag.as_str() == name.as_str(css),
        MyComponent::Id(ident) => html
            .attrs
            .get("id")
            .map(|id| id.as_str() == ident.as_str(css))
            .unwrap_or(false),
        MyComponent::Class(ident) => html
            .attrs
            .get("class")
            .map(|classes| match_class(classes, ident.as_str(css)))
            .unwrap_or(false),
        MyComponent::Attribute(name, operator, value) => {
            let value = value.as_str(css);
            html.attrs
                .get(name.as_str(css))
                .map(|attr| match operator {
                    MyMatcher::Exist => true,
                    MyMatcher::Equal => attr == value,
                    MyMatcher::Include => attr.split(" ").any(|word| word == value),
                    MyMatcher::DashMatch => attr == value || attr == &format!("-{value}"),
                    MyMatcher::Prefix => attr.starts_with(value),
                    MyMatcher::Substring => attr.contains(value),
                    MyMatcher::Suffix => attr.ends_with(value),
                })
                .unwrap_or(false)
        }
        MyComponent::Root => tree.parent(node).is_none(),
        MyComponent::PseudoClass(name) => html.pseudo_classes.contains(name.as_str(css)),
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
