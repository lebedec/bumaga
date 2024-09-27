mod apply;
mod compute_animation_tracks;
mod compute_style;
mod default;
mod inherit;
mod initial;
mod scrolling;

pub use default::*;
pub use inherit::*;
pub use scrolling::*;

use log::error;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env::var;

use taffy::{
    Dimension, Layout, LengthPercentage, LengthPercentageAuto, NodeId, Overflow, Point, Rect,
    TaffyTree,
};

use crate::animation::{
    AnimationDirection, AnimationFillMode, AnimationIterations, Animator, TimingFunction,
    Transition,
};
use crate::css::{
    match_style, Animation, AnimationTrack, ComputedStyle, ComputedValue, Css, Declaration,
    Definition, Dim, Property, PropertyKey, PseudoClassMatcher, Shorthand, Style, Units, Var,
    Variable,
};

use crate::css::ComputedValue::{Keyword, Number, Time};
use crate::styles::initial::initial;
use crate::{
    Background, Borders, Element, FontFace, Input, Length, ObjectFit, PointerEvents, TextAlign,
    TransformFunction,
};

/// The cascade is an algorithm that defines how to combine CSS (Cascading Style Sheets)
/// property values originating from different sources.
pub struct Cascade<'c> {
    css: &'c Css,
    pub variables: Variables,
    sizes: Sizes,
    resources: &'c str,
}

pub type Variables = HashMap<String, Shorthand>;

#[derive(Debug)]
pub enum CascadeError {
    PropertyNotSupported,
    DimensionUnitsNotSupported,
    ValueNotSupported,
    TransformFunctionNotSupported,
    VariableNotFound,
    InvalidKeyword(String),
}

impl CascadeError {
    pub fn invalid_keyword<T>(keyword: &str) -> Result<T, Self> {
        Err(CascadeError::InvalidKeyword(keyword.to_string()))
    }
}

impl<'c> Cascade<'c> {
    pub fn new(css: &'c Css, sizes: Sizes, variables: Variables, resources: &'c str) -> Self {
        Self {
            css,
            variables,
            sizes,
            resources,
        }
    }

    pub fn apply_styles(
        &mut self,
        input: &Input,
        node: NodeId,
        tree: &TaffyTree<Element>,
        parent: &Element,
        layout: &mut taffy::Style,
        element: &mut Element,
        matcher: &impl PseudoClassMatcher,
    ) {
        // -1: initial
        reset_element_style(element);
        // 0: inheritance
        inherit(parent, element);
        // 1: css rules
        let mut computed_style = HashMap::new();
        for style in &self.css.styles {
            if match_style(&style, node, tree, matcher) {
                self.compute_declaration_block(&style.declaration, &mut computed_style);
            }
        }
        // 2: inline css
        if !element.style.is_empty() {
            self.compute_declaration_block(&element.style, &mut computed_style);
        }
        // 3: animations
        let time = input.time.as_secs_f32();
        for animator in element.animators.iter_mut() {
            // TODO: animation blending
            let animation = match self.css.animations.get(&animator.name) {
                Some(animation) => animation,
                None => {
                    error!("unable to play animation {}, not found", animator.name);
                    continue;
                }
            };
            // TODO: cache animation computation ?
            let tracks = self.compute_animation_tracks(animation, &computed_style);
            animator.play(time, &tracks, &mut computed_style);
        }
        // TODO: !important
        // 4: transitions
        for transition in element.transitions.iter_mut() {
            transition.play(time, &mut computed_style);
        }
        for (property, value) in computed_style {
            if let Err(error) = self.apply(property.key, property.index, &value, layout, element) {
                error!("unable to apply {property:?}:{value:?} because of {error:?}");
            }
        }
    }

    pub fn take_variables(self) -> HashMap<String, Shorthand> {
        self.variables
    }

    fn compute_declaration_block(&mut self, block: &[Declaration], style: &mut ComputedStyle) {
        for declaration in block {
            match declaration {
                Declaration::Variable(variable) => self.set_variable(variable),
                Declaration::Property(property) => {
                    for index in 0..property.values.len() {
                        self.compute_style(property.key, index, &property.values[index], style);
                    }
                }
            }
        }
    }

    fn compute_shorthand(
        &self,
        definition: &[Definition],
        shorthand: &mut Vec<ComputedValue>,
    ) -> bool {
        for value in definition {
            match value {
                Definition::Var(name) => {
                    let definition = match self.get_variable(name) {
                        Some(shorthand) => shorthand,
                        None => {
                            error!("unable to compute variable {name}, not found");
                            return false;
                        }
                    };
                    self.compute_shorthand(definition, shorthand);
                }
                Definition::Function(function) => match function.name.as_str() {
                    "rgb" | "rgba" => {
                        let mut arguments = vec![];
                        self.compute_shorthand(&function.arguments, &mut arguments);
                        match arguments.as_slice() {
                            [Number(r), Number(g), Number(b), Number(a)] => {
                                shorthand.push(ComputedValue::Color([
                                    *r as u8,
                                    *g as u8,
                                    *b as u8,
                                    (a * 255.0) as u8,
                                ]));
                            }
                            [Number(r), Number(g), Number(b)] => {
                                shorthand.push(ComputedValue::Color([
                                    *r as u8, *g as u8, *b as u8, 255,
                                ]));
                            }
                            _ => {
                                error!(
                                    "unable to compute function {}, arguments {:?} not supported",
                                    function.name, arguments
                                );
                                return false;
                            }
                        }
                    }
                    _ => {
                        error!(
                            "unable to compute function {}, not supported",
                            function.name
                        );
                        return false;
                    }
                },
                Definition::Explicit(value) => shorthand.push(value.clone()),
            }
        }
        true
    }

    fn set_variable(&mut self, variable: &Variable) {
        self.variables
            .insert(variable.key.clone(), variable.shorthand.clone());
    }

    fn get_variable(&self, name: &str) -> Option<&Shorthand> {
        self.variables.get(name)
    }
}

#[derive(Clone, Copy)]
pub struct Sizes {
    pub root_font_size: f32,
    pub parent_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}