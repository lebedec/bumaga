use crate::animation::{Animator, Transition};
use crate::css::PropertyKey;
use crate::styles::Scrolling;
use crate::Element;
use std::collections::{HashMap, HashSet};
use taffy::NodeId;

pub struct State {
    pub scroll: Option<NodeId>,
    pub focus: Option<NodeId>,
    pub hover: Option<NodeId>,
}

impl State {
    pub fn new() -> Self {
        State {
            scroll: None,
            focus: None,
            hover: None,
        }
    }
}
