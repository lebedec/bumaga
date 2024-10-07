use crate::{Background, Borders, Element, FontFace, Length, ObjectFit, TextAlign};
use std::collections::HashMap;
use taffy::{Dimension, NodeId, Overflow, Point, Rect};

impl FontFace {
    pub const DEFAULT_FONT_FAMILY: &'static str = "system-ui";
    pub const DEFAULT_FONT_WEIGHT: u16 = 400;
    // pub const DEFAULT_FONT_STRETCH: FontStretchKeyword = FontStretchKeyword::Normal;
}

pub(crate) fn reset_element_style(element: &mut Element) {
    element.backgrounds = vec![];
    element.borders = Borders {
        top: Default::default(),
        bottom: Default::default(),
        right: Default::default(),
        left: Default::default(),
        radius: [Length::zero(); 4],
    };
    element.color = [0, 0, 0, 255];
    element.font = FontFace {
        family: FontFace::DEFAULT_FONT_FAMILY.to_string(),
        size: 16.0,
        style: "normal".to_string(),
        weight: FontFace::DEFAULT_FONT_WEIGHT,
        // font_stretch: TextStyle::DEFAULT_FONT_STRETCH,
        line_height: 1.0,
        // wrap: OverflowWrap::Normal,
        align: TextAlign::Start,
    };
    element.opacity = 1.0;
}

pub fn create_element(node: NodeId) -> Element {
    Element {
        node,
        children: vec![],
        tag: "".to_string(),
        text: None,
        attrs: Default::default(),
        attrs_bindings: Default::default(),
        position: [0.0; 2],
        size: [0.0; 2],
        content_size: [0.0; 2],
        object_fit: ObjectFit::Fill,
        backgrounds: vec![],
        borders: Borders {
            top: Default::default(),
            bottom: Default::default(),
            right: Default::default(),
            left: Default::default(),
            radius: [Length::zero(); 4],
        },
        color: [0, 0, 0, 255],
        font: FontFace {
            family: FontFace::DEFAULT_FONT_FAMILY.to_string(),
            size: 16.0,
            style: "normal".to_string(),
            weight: FontFace::DEFAULT_FONT_WEIGHT,
            // font_stretch: TextStyle::DEFAULT_FONT_STRETCH,
            line_height: 1.0,
            // wrap: OverflowWrap::Normal,
            align: TextAlign::Start,
        },
        listeners: Default::default(),
        opacity: 1.0,
        transforms: vec![],
        animators: vec![],
        scrolling: None,
        clipping: None,
        transitions: vec![],
        state: Default::default(),
        pointer_events: Default::default(),
        style_hints: Default::default(),
        styles: vec![],
        style: vec![],
    }
}

pub fn default_layout() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Block,
        overflow: Point {
            x: Overflow::Visible,
            y: Overflow::Visible,
        },
        scrollbar_width: 0.0,
        position: taffy::Position::Relative,
        inset: Rect::auto(),
        margin: Rect::zero(),
        padding: Rect::zero(),
        border: Rect::zero(),
        size: taffy::Size::auto(),
        min_size: taffy::Size::auto(),
        max_size: taffy::Size::auto(),
        aspect_ratio: None,
        gap: taffy::Size::zero(),
        align_items: None,
        align_self: None,
        justify_items: None,
        justify_self: None,
        align_content: None,
        justify_content: None,
        flex_direction: taffy::FlexDirection::Row,
        flex_wrap: taffy::FlexWrap::NoWrap,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: Dimension::Auto,
        ..Default::default()
    }
}
