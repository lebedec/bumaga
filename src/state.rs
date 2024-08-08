use crate::animation::{Animator, Transition};
use crate::css::PropertyKey;
use crate::models::ElementId;
use crate::styles::Scrolling;
use crate::Element;
use std::collections::{HashMap, HashSet};
use taffy::NodeId;

pub struct State {
    pub scroll: Option<NodeId>,
    pub focus: Option<NodeId>,
    pub hover: Option<NodeId>,
    pub pseudo_classes: HashMap<ElementId, HashSet<String>>,
    pub animators: HashMap<ElementId, Animator>,
    pub scrolling: HashMap<ElementId, Scrolling>,
    pub transitions: HashMap<ElementId, HashMap<PropertyKey, Transition>>,
}

impl State {
    pub fn new() -> Self {
        State {
            scroll: None,
            focus: None,
            hover: None,
            pseudo_classes: HashMap::new(),
            animators: HashMap::new(),
            scrolling: HashMap::new(),
            transitions: HashMap::new(),
        }
    }

    /// Removes all unused state.
    pub fn prune(&mut self) {
        self.pseudo_classes = HashMap::new();
        self.animators = HashMap::new();
        self.scrolling = HashMap::new();
        self.transitions = HashMap::new();
    }

    pub fn restore(&mut self, element: &mut Element) {
        if let Some(animator) = self.animators.remove(&element.id) {
            element.animator = animator.clone();
        }
        if let Some(classes) = self.pseudo_classes.remove(&element.id) {
            element.html.pseudo_classes = classes.clone();
        }
        if let Some(scrolling) = self.scrolling.remove(&element.id) {
            element.scrolling = Some(scrolling);
        }
        if let Some(transitions) = self.transitions.remove(&element.id) {
            element.transitions = transitions;
        }
    }

    pub fn save(&mut self, element: &Element) {
        if element.animator.is_declared() {
            self.animators.insert(element.id, element.animator.clone());
        }
        if !element.html.pseudo_classes.is_empty() {
            self.pseudo_classes
                .insert(element.id, element.html.pseudo_classes.clone());
        }
        if let Some(scrolling) = element.scrolling.clone() {
            self.scrolling.insert(element.id, scrolling);
        }
        if !element.transitions.is_empty() {
            self.transitions
                .insert(element.id, element.transitions.clone());
        }
    }
}
