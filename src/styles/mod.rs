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

use crate::css::ComputedValue::{Keyword, Time};
use crate::styles::initial::initial;
use crate::{
    Background, Borders, Element, FontFace, Input, Length, ObjectFit, PointerEvents, TextAlign,
    TransformFunction,
};

/// The cascade is an algorithm that defines how to combine CSS (Cascading Style Sheets)
/// property values originating from different sources.
pub struct Cascade<'c> {
    css: &'c Css,
    variables: HashMap<&'c str, &'c Vec<Shorthand>>,
    sizes: Sizes,
    resources: &'c str,
}

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
    pub fn new(css: &'c Css, sizes: Sizes, resources: &'c str) -> Self {
        Self {
            css,
            variables: HashMap::with_capacity(8),
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
        // 4: transitions ?
        for (property, value) in computed_style {
            if let Err(error) = self.apply(property.key, property.index, &value, layout, element) {
                error!("unable to apply {property:?}:{value:?} because of {error:?}");
            }
        }
    }

    // fn apply_declarations(
    //     &mut self,
    //     declarations: &Vec<Declaration>,
    //     layout: &mut taffy::Style,
    //     element: &mut Element,
    // ) {
    //     // for property in &styles.declaration {
    //     //     // if let PropertyKey::Variable(name) = property.key {
    //     //     //     self.push_variable(name, &property.values);
    //     //     //     continue;
    //     //     // }
    //     //     if PropertyKey::Transition == property.key {
    //     //         for shorthand in property.values.to_vec() {
    //     //             let key = match &shorthand[0] {
    //     //                 Keyword(name) => {
    //     //                     let key = match PropertyKey::parse(name) {
    //     //                         Some(key) => key,
    //     //                         None => {
    //     //                             error!("unable to make transition of {name}, not supported");
    //     //                             continue;
    //     //                         }
    //     //                     };
    //     //                     key
    //     //                 }
    //     //                 _ => {
    //     //                     error!("invalid transition property value");
    //     //                     continue;
    //     //                 }
    //     //             };
    //     //             let transition = element
    //     //                 .transitions
    //     //                 .entry(key)
    //     //                 .or_insert_with(|| Transition::new(key));
    //     //             match &shorthand[1..] {
    //     //                 [Time(duration)] => {
    //     //                     transition.set_duration(*duration);
    //     //                 }
    //     //                 [Time(duration), timing] => {
    //     //                     transition.set_duration(*duration);
    //     //                     transition.set_timing(resolve_timing(&timing, self).unwrap());
    //     //                 }
    //     //                 [Time(duration), timing, Time(delay)] => {
    //     //                     transition.set_duration(*duration);
    //     //                     transition.set_timing(resolve_timing(&timing, self).unwrap());
    //     //                     transition.set_delay(*delay);
    //     //                 }
    //     //                 shorthand => {
    //     //                     error!("transition value not supported {shorthand:?}");
    //     //                     continue;
    //     //                 }
    //     //             }
    //     //         }
    //     //         continue;
    //     //     }
    //     //     if let Some(transition) = element.transitions.get_mut(&property.key) {
    //     //         let shorthand = property.get_first_shorthand();
    //     //         // ID ?
    //     //         transition.set(property.id, shorthand);
    //     //         continue;
    //     //     }
    //     //     if let Err(error) =
    //     //         self.apply_shorthand(property.key, &property.values[0], layout, element)
    //     //     {
    //     //         error!("unable to apply property {property:?}, {error:?}")
    //     //     }
    //     // }
    //     for declaration in declarations {
    //         match declaration {
    //             Declaration::Variable(variable) => self.set_variable(variable),
    //             Declaration::Property(property) => self.apply_property(property, layout, element),
    //         }
    //     }
    // }

    fn compute_declaration_block(&self, block: &[Declaration], style: &mut ComputedStyle) {
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

    fn set_variable(&self, variable: &Variable) {
        unimplemented!()
    }

    fn get_variable(&self, name: &str) -> Option<&Shorthand> {
        unimplemented!()
    }
}

#[derive(Clone, Copy)]
pub struct Sizes {
    pub root_font_size: f32,
    pub parent_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}
