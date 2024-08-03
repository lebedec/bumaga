use crate::Element;
use lightningcss::rules::style::StyleRule;
use lightningcss::selector::{PseudoClass, PseudoElement};
use lightningcss::values::ident::Ident;
use lightningcss::values::string::{CSSString, CowArcStr};
use lightningcss::vendor_prefix::VendorPrefix;
use log::error;
use parcel_selectors::attr::AttrSelectorOperator;
use parcel_selectors::parser::{Combinator, Component, NthType, Selector};
use parcel_selectors::SelectorImpl;
use std::collections::{HashMap, HashSet};
use taffy::{NodeId, TaffyTree, TraversePartialTree};

pub fn match_rule(rule: &StyleRule, node: NodeId, tree: &TaffyTree<Element>) -> bool {
    rule.selectors
        .0
        .iter()
        .any(|selector| match_complex_selector(selector, node, tree))
}

fn match_complex_selector<'i, T>(
    selector: &Selector<'i, T>,
    node: NodeId,
    tree: &TaffyTree<Element>,
) -> bool
where
    T: SelectorImpl<
        'i,
        Identifier = Ident<'i>,
        NonTSPseudoClass = PseudoClass<'i>,
        AttrValue = CSSString<'i>,
        LocalName = Ident<'i>,
    >,
{
    let mut target = node;
    selector
        .iter_raw_match_order()
        .all(|component| match component.as_combinator() {
            None => match_simple_selector(component, target, tree),
            Some(combinator) => find_next_target(combinator, &mut target, tree),
        })
}

fn find_next_target(
    combinator: Combinator,
    target: &mut NodeId,
    tree: &TaffyTree<Element>,
) -> bool {
    let next = match combinator {
        Combinator::Child => tree.parent(*target),
        Combinator::NextSibling => tree
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

fn match_simple_selector<'i, T>(
    component: &Component<'i, T>,
    node: NodeId,
    tree: &TaffyTree<Element>,
) -> bool
where
    T: SelectorImpl<
        'i,
        Identifier = Ident<'i>,
        NonTSPseudoClass = PseudoClass<'i>,
        AttrValue = CSSString<'i>,
        LocalName = Ident<'i>,
    >,
{
    let html = match tree.get_node_context(node) {
        Some(element) => &element.html,
        None => {
            error!("unable to match selector for node {node:?}, html context not found");
            return false;
        }
    };
    match component {
        Component::LocalName(local) => html.tag.as_str() == local.name.as_ref(),
        Component::ID(ident) => html
            .attrs
            .get("id")
            .map(|id| id.as_str() == ident.as_ref())
            .unwrap_or(false),
        Component::Class(ident) => html
            .attrs
            .get("class")
            .map(|classes| match_class(classes, ident))
            .unwrap_or(false),
        Component::AttributeInNoNamespace {
            local_name,
            value,
            operator,
            ..
        } => {
            let value = value.as_ref();
            html.attrs
                .get(local_name.as_ref())
                .map(|attr| match operator {
                    AttrSelectorOperator::Equal => attr == value,
                    AttrSelectorOperator::Includes => attr.split(" ").any(|word| word == value),
                    AttrSelectorOperator::DashMatch => {
                        attr == value || attr == &format!("-{value}")
                    }
                    AttrSelectorOperator::Prefix => attr.starts_with(value),
                    AttrSelectorOperator::Substring => attr.contains(value),
                    AttrSelectorOperator::Suffix => attr.ends_with(value),
                })
                .unwrap_or(false)
        }
        Component::NonTSPseudoClass(class) => {
            let name = match class {
                PseudoClass::Hover => "hover",
                PseudoClass::Active => "active",
                PseudoClass::Focus => "focus",
                _ => {
                    error!("pseudo class {component:?} not supported");
                    return false;
                }
            };
            html.pseudo_classes.contains(name)
        }
        _ => {
            error!("selector {component:?} not supported");
            false
        }
    }
}

#[inline(always)]
fn match_class(classes: &str, ident: &Ident) -> bool {
    classes
        .split(" ")
        .any(|class| class.as_bytes() == ident.as_bytes())
}
