use std::collections::{HashMap, HashSet};

use crate::animation::Animator;
use crate::models::ElementId;

pub struct State {
    pub pseudo_classes: HashMap<ElementId, HashSet<String>>,
    pub no_pseudo_classes: HashSet<String>,
    pub focus: Option<ElementId>,
    pub animators: HashMap<ElementId, Vec<Animator>>,
    pub no_animators: Vec<Animator>,
    pub active_animators: HashMap<ElementId, Vec<Animator>>,
}

impl State {
    pub fn new() -> Self {
        State {
            pseudo_classes: HashMap::new(),
            no_pseudo_classes: Default::default(),
            focus: None,
            animators: Default::default(),
            no_animators: vec![],
            active_animators: Default::default(),
        }
    }

    pub fn reset_focus(&mut self) {
        self.focus = None;
    }

    pub fn set_focus(&mut self, element_id: ElementId) {
        self.focus = Some(element_id)
    }

    pub fn load_animators_mut(&mut self, element_id: ElementId) -> &mut Vec<Animator> {
        self.animators
            .get_mut(&element_id)
            .unwrap_or(&mut self.no_animators)
    }

    pub fn load_pseudo_classes(&self, element_id: ElementId) -> &HashSet<String> {
        self.pseudo_classes
            .get(&element_id)
            .unwrap_or(&self.no_pseudo_classes)
    }

    pub fn save_pseudo_classes(&mut self, element_id: ElementId, classes: HashSet<String>) {
        self.pseudo_classes.insert(element_id, classes);
    }
}
