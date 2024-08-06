use std::collections::{HashMap, HashSet};

use crate::animation::Animator;
use crate::models::ElementId;
use crate::Element;

pub struct State {
    pub focus: Option<ElementId>,
    pub pseudo_classes: HashMap<ElementId, HashSet<String>>,
    pub animators: HashMap<ElementId, Animator>,
}

impl State {
    pub fn new() -> Self {
        State {
            focus: None,
            pseudo_classes: HashMap::new(),
            animators: HashMap::new(),
        }
    }

    pub fn reset_focus(&mut self) {
        self.focus = None;
    }

    pub fn set_focus(&mut self, element_id: ElementId) {
        self.focus = Some(element_id)
    }

    /// Removes all unused state.
    pub fn prune(&mut self) {
        self.pseudo_classes = HashMap::new();
        self.animators = HashMap::new();
    }

    pub fn restore(&mut self, element: &mut Element) {
        if let Some(animator) = self.animators.remove(&element.id) {
            element.animator = animator.clone();
        }
        if let Some(classes) = self.pseudo_classes.remove(&element.id) {
            element.html.pseudo_classes = classes.clone();
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
    }
}
