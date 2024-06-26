use std::collections::HashMap;

use crate::models::ElementId;

pub struct State {
    pub element_n: usize,
    pub pseudo_classes: HashMap<ElementId, Vec<String>>,
    pub focus: Option<ElementId>,
}

static NO_PSEUDO_CLASSES: Vec<String> = vec![];

impl State {
    pub fn new() -> Self {
        State {
            element_n: 0,
            pseudo_classes: HashMap::new(),
            focus: None,
        }
    }

    pub fn reset_focus(&mut self) {
        self.focus = None;
    }

    pub fn set_focus(&mut self, element_id: ElementId) {
        self.focus = Some(element_id)
    }

    pub fn get_pseudo_classes(&self, element_id: ElementId) -> &Vec<String> {
        self.pseudo_classes
            .get(&element_id)
            .unwrap_or(&NO_PSEUDO_CLASSES)
    }

    pub fn set_pseudo_classes(&mut self, element_id: ElementId, classes: Vec<String>) {
        self.pseudo_classes.insert(element_id, classes);
    }

    pub fn has_pseudo_class(&self, element_id: ElementId, target: &str) -> bool {
        match self.pseudo_classes.get(&element_id) {
            None => false,
            Some(classes) => classes
                .iter()
                .find(|class| class.as_str() == target)
                .is_some(),
        }
    }
}
