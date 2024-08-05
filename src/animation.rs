use std::collections::HashMap;
use std::ops::Add;
use std::rc::Rc;
use std::time::Duration;

use crate::css::MyProperty;
use crate::models::ElementId;
use crate::{Component, Element};

pub struct Animator {
    pub easing: u32,
    pub duration: f32,
    pub play_state: u32,
    pub iterations: AnimationIterationCount,
    pub animation: Rc<Animation>,
    pub time: f32,
}

pub struct Animation {
    pub name: String,
    pub tracks: Vec<Track>,
}

pub struct Keyframe {
    pub time: f32,
    pub property: MyProperty,
}

#[derive(Default)]
pub struct Track {
    pub keyframes: Vec<Keyframe>,
}

pub fn apply_animation_rules(
    declarations: &Vec<MyProperty>,
    element: &mut Element,
    active_animators: &mut HashMap<ElementId, Vec<Animator>>,
    animators: &mut HashMap<ElementId, Vec<Animator>>,
    animations: &HashMap<String, Rc<Animation>>,
) {
    //let mut empty = vec![];
    for property in declarations {
        // match property {
        //     Property::Animation(declarations, _) => {
        //         for declaration in declarations {
        //             let active = active_animators.get_mut(&element.id).unwrap_or(&mut empty);
        //             let current = animators.entry(element.id).or_default();
        //             let name = match &declaration.name {
        //                 AnimationName::Ident(name) => name.to_string(),
        //                 name => {
        //                     error!("animation {name:?} not supported");
        //                     continue;
        //                 }
        //             };
        //             let found = active.iter().position(|animator| animator.id() == name);
        //             match found {
        //                 None => {
        //                     let animation = match animations.get(&name) {
        //                         None => {
        //                             error!("animation {name} @keyframes not specified");
        //                             continue;
        //                         }
        //                         Some(animation) => animation.clone(),
        //                     };
        //                     let duration = match declaration.duration {
        //                         Time::Seconds(value) => value,
        //                         Time::Milliseconds(value) => value / 1000.0,
        //                     };
        //                     let easing = declaration.timing_function.clone();
        //                     let iterations = declaration.iteration_count.clone();
        //                     let animator = Animator {
        //                         easing,
        //                         duration,
        //                         play_state: Default::default(),
        //                         iterations,
        //                         animation,
        //                         time: 0.0,
        //                     };
        //                     current.push(animator);
        //                 }
        //                 Some(index) => {
        //                     let animator = active.remove(index);
        //                     current.push(animator);
        //                 }
        //             }
        //         }
        //     }
        //     _ => {}
        // }
    }
}

enum AnimationIterationCount {
    Number(f32),
    Infinite,
}

impl Animator {
    pub fn id(&self) -> &str {
        self.animation.name.as_str()
    }

    pub fn update(&mut self, time: f32) -> Vec<MyProperty> {
        let time = if self.duration <= 0.0 {
            0.0
        } else {
            self.time += time;
            match self.iterations {
                AnimationIterationCount::Number(count) => {
                    if self.time >= self.duration * count {
                        self.time = self.duration * count;
                        1.0
                    } else {
                        (self.time / self.duration).fract()
                    }
                }
                AnimationIterationCount::Infinite => {
                    if self.time >= self.duration {
                        self.time = self.time - self.duration;
                    }
                    self.time / self.duration
                }
            }
        };
        let mut declarations = vec![];
        for track in &self.animation.tracks {
            let mut a = 0;
            let mut b = 0;
            for i in 0..track.keyframes.len() {
                let keyframe = &track.keyframes[i];
                if keyframe.time <= time {
                    a = i;
                } else {
                    b = i;
                    break;
                }
            }
            if a + 1 == b {
                // between a and b
                // let property = self.interpolate_property(
                //     &track.keyframes[a].property,
                //     &track.keyframes[b].property,
                //     time,
                // );
                // declarations.push(property);
            } else {
                // return a frame (last)
                // let property = track.keyframes[a].property.clone();
                // declarations.push(property);
            }
        }
        declarations
    }

    // fn interpolate_property(&self, a: &Property, b: &Property, t: f32) -> Property<'static> {
    //     match (a, b) {
    //         (Property::Height(a), Property::Height(b)) => Property::Height(self.size(a, b, t)),
    //         (Property::MinHeight(a), Property::MinHeight(b)) => {
    //             Property::Height(self.size(a, b, t))
    //         }
    //         (Property::Width(a), Property::Width(b)) => Property::Width(self.size(a, b, t)),
    //         (Property::MinWidth(a), Property::MinWidth(b)) => {
    //             Property::MinWidth(self.size(a, b, t))
    //         }
    //         (Property::Color(a), Property::Color(b)) => Property::Color(self.color(a, b, t)),
    //         (BackgroundColor(a), BackgroundColor(b)) => BackgroundColor(self.color(a, b, t)),
    //         (a, b) => {
    //             error!("interpolation from {a:?} to {b:?} not supported");
    //             a.clone().into_owned()
    //         }
    //     }
    // }
    //
    // fn color(&self, a: &CssColor, b: &CssColor, t: f32) -> CssColor {
    //     let fallback = a.clone();
    //     match (a, b) {
    //         (CssColor::RGBA(x), CssColor::RGBA(y)) => {
    //             let rgba = RGBA {
    //                 red: ((x.red_f32() + (y.red_f32() - x.red_f32()) * t) * 255.0) as u8,
    //                 green: ((x.green_f32() + (y.green_f32() - x.green_f32()) * t) * 255.0) as u8,
    //                 blue: ((x.blue_f32() + (y.blue_f32() - x.blue_f32()) * t) * 255.0) as u8,
    //                 alpha: ((x.alpha_f32() + (y.alpha_f32() - x.alpha_f32()) * t) * 255.0) as u8,
    //             };
    //             CssColor::RGBA(rgba)
    //         }
    //         _ => {
    //             error!("interpolation from {a:?} to {b:?} not supported");
    //             fallback
    //         }
    //     }
    // }
}
