use std::collections::HashMap;
use std::ops::Add;
use std::rc::Rc;
use std::time::Duration;

use lightningcss::properties::animation::{
    AnimationIterationCount, AnimationName, AnimationPlayState,
};
use lightningcss::properties::Property;
use lightningcss::properties::size::Size;
use lightningcss::values::easing::EasingFunction;
use lightningcss::values::length::{LengthPercentage, LengthValue};
use lightningcss::values::number::CSSNumber;
use lightningcss::values::percentage::Percentage;
use lightningcss::values::time::Time;
use log::error;
use static_self::IntoOwned;

use crate::{Component, Element};
use crate::models::ElementId;

pub struct Animator {
    pub easing: EasingFunction,
    pub duration: f32,
    pub play_state: AnimationPlayState,
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
    pub property: Property<'static>,
}

#[derive(Default)]
pub struct Track {
    pub keyframes: Vec<Keyframe>,
}

pub fn apply_animation_rules<'i>(
    declarations: &Vec<Property<'i>>,
    element: &mut Element,
    active_animators: &mut HashMap<ElementId, Vec<Animator>>,
    animators: &mut HashMap<ElementId, Vec<Animator>>,
    animations: &HashMap<String, Rc<Animation>>,
) {
    let mut empty = vec![];
    for property in declarations {
        match property {
            Property::Animation(declarations, _) => {
                for declaration in declarations {
                    let active = active_animators.get_mut(&element.id).unwrap_or(&mut empty);
                    let current = animators.entry(element.id).or_default();
                    let name = match &declaration.name {
                        AnimationName::Ident(name) => name.to_string(),
                        name => {
                            error!("animation {name:?} not supported");
                            continue;
                        }
                    };
                    let found = active.iter().position(|animator| animator.id() == name);
                    match found {
                        None => {
                            let animation = match animations.get(&name) {
                                None => {
                                    error!("animation {name} @keyframes not specified");
                                    continue;
                                }
                                Some(animation) => animation.clone(),
                            };
                            let duration = match declaration.duration {
                                Time::Seconds(value) => value,
                                Time::Milliseconds(value) => value / 1000.0,
                            };
                            let easing = declaration.timing_function.clone();
                            let iterations = declaration.iteration_count.clone();
                            let animator = Animator {
                                easing,
                                duration,
                                play_state: Default::default(),
                                iterations,
                                animation,
                                time: 0.0,
                            };
                            current.push(animator);
                        }
                        Some(index) => {
                            let animator = active.remove(index);
                            current.push(animator);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl Animator {
    pub fn id(&self) -> &str {
        self.animation.name.as_str()
    }

    pub fn update(&mut self, time: f32) -> Vec<Property<'static>> {
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
                let property = self.interpolate_property(
                    &track.keyframes[a].property,
                    &track.keyframes[b].property,
                    time,
                );
                declarations.push(property);
            } else {
                // return a frame (last)
                let property = track.keyframes[a].property.clone();
                declarations.push(property);
            }
        }
        declarations
    }

    fn interpolate_property(&self, a: &Property, b: &Property, t: f32) -> Property<'static> {
        match (a, b) {
            (Property::Height(a), Property::Height(b)) => Property::Height(self.size(a, b, t)),
            (Property::Width(a), Property::Width(b)) => Property::Width(self.size(a, b, t)),
            (a, b) => {
                error!("interpolation from {a:?} to {b:?} not supported");
                a.clone().into_owned()
            }
        }
    }

    fn size(&self, a: &Size, b: &Size, t: f32) -> Size {
        let fallback = a.clone();
        match (a, b) {
            (Size::LengthPercentage(a), Size::LengthPercentage(b)) => match (a, b) {
                (LengthPercentage::Dimension(a), LengthPercentage::Dimension(b)) => {
                    let value = self.interpolate_length(a, b, t);
                    Size::LengthPercentage(LengthPercentage::Dimension(value))
                }
                (LengthPercentage::Percentage(a), LengthPercentage::Percentage(b)) => {
                    let value = a.0 + t * (b.0 - a.0);
                    Size::LengthPercentage(LengthPercentage::Percentage(Percentage(value)))
                }
                _ => {
                    error!("interpolation from {a:?} to {b:?} not supported");
                    fallback
                }
            },
            _ => {
                error!("interpolation from {a:?} to {b:?} not supported");
                fallback
            }
        }
    }

    fn interpolate_length(&self, a: &LengthValue, b: &LengthValue, t: f32) -> LengthValue {
        match (a, b) {
            (LengthValue::Px(a), LengthValue::Px(b)) => LengthValue::Px(a + t * (b - a)),
            (LengthValue::In(a), LengthValue::In(b)) => LengthValue::In(a + t * (b - a)),
            (LengthValue::Cm(a), LengthValue::Cm(b)) => LengthValue::Cm(a + t * (b - a)),
            (LengthValue::Mm(a), LengthValue::Mm(b)) => LengthValue::Mm(a + t * (b - a)),
            (LengthValue::Q(a), LengthValue::Q(b)) => LengthValue::Q(a + t * (b - a)),
            (LengthValue::Pt(a), LengthValue::Pt(b)) => LengthValue::Pt(a + t * (b - a)),
            (LengthValue::Pc(a), LengthValue::Pc(b)) => LengthValue::Pc(a + t * (b - a)),
            (LengthValue::Em(a), LengthValue::Em(b)) => LengthValue::Em(a + t * (b - a)),
            (LengthValue::Rem(a), LengthValue::Rem(b)) => LengthValue::Rem(a + t * (b - a)),
            (LengthValue::Ex(a), LengthValue::Ex(b)) => LengthValue::Ex(a + t * (b - a)),
            (LengthValue::Rex(a), LengthValue::Rex(b)) => LengthValue::Rex(a + t * (b - a)),
            (LengthValue::Ch(a), LengthValue::Ch(b)) => LengthValue::Ch(a + t * (b - a)),
            (LengthValue::Rch(a), LengthValue::Rch(b)) => LengthValue::Rch(a + t * (b - a)),
            (LengthValue::Cap(a), LengthValue::Cap(b)) => LengthValue::Cap(a + t * (b - a)),
            (LengthValue::Rcap(a), LengthValue::Rcap(b)) => LengthValue::Rcap(a + t * (b - a)),
            (LengthValue::Ic(a), LengthValue::Ic(b)) => LengthValue::Ic(a + t * (b - a)),
            (LengthValue::Ric(a), LengthValue::Ric(b)) => LengthValue::Ric(a + t * (b - a)),
            (LengthValue::Lh(a), LengthValue::Lh(b)) => LengthValue::Lh(a + t * (b - a)),
            (LengthValue::Rlh(a), LengthValue::Rlh(b)) => LengthValue::Rlh(a + t * (b - a)),
            (LengthValue::Vw(a), LengthValue::Vw(b)) => LengthValue::Vw(a + t * (b - a)),
            (LengthValue::Lvw(a), LengthValue::Lvw(b)) => LengthValue::Lvw(a + t * (b - a)),
            (LengthValue::Svw(a), LengthValue::Svw(b)) => LengthValue::Svw(a + t * (b - a)),
            (LengthValue::Dvw(a), LengthValue::Dvw(b)) => LengthValue::Dvw(a + t * (b - a)),
            (LengthValue::Cqw(a), LengthValue::Cqw(b)) => LengthValue::Cqw(a + t * (b - a)),
            (LengthValue::Vh(a), LengthValue::Vh(b)) => LengthValue::Vh(a + t * (b - a)),
            (LengthValue::Lvh(a), LengthValue::Lvh(b)) => LengthValue::Lvh(a + t * (b - a)),
            (LengthValue::Svh(a), LengthValue::Svh(b)) => LengthValue::Svh(a + t * (b - a)),
            (LengthValue::Dvh(a), LengthValue::Dvh(b)) => LengthValue::Dvh(a + t * (b - a)),
            (LengthValue::Cqh(a), LengthValue::Cqh(b)) => LengthValue::Cqh(a + t * (b - a)),
            (LengthValue::Vi(a), LengthValue::Vi(b)) => LengthValue::Vi(a + t * (b - a)),
            (LengthValue::Svi(a), LengthValue::Svi(b)) => LengthValue::Svi(a + t * (b - a)),
            (LengthValue::Lvi(a), LengthValue::Lvi(b)) => LengthValue::Lvi(a + t * (b - a)),
            (LengthValue::Dvi(a), LengthValue::Dvi(b)) => LengthValue::Dvi(a + t * (b - a)),
            (LengthValue::Cqi(a), LengthValue::Cqi(b)) => LengthValue::Cqi(a + t * (b - a)),
            (LengthValue::Vb(a), LengthValue::Vb(b)) => LengthValue::Vb(a + t * (b - a)),
            (LengthValue::Svb(a), LengthValue::Svb(b)) => LengthValue::Svb(a + t * (b - a)),
            (LengthValue::Lvb(a), LengthValue::Lvb(b)) => LengthValue::Lvb(a + t * (b - a)),
            (LengthValue::Dvb(a), LengthValue::Dvb(b)) => LengthValue::Dvb(a + t * (b - a)),
            (LengthValue::Cqb(a), LengthValue::Cqb(b)) => LengthValue::Cqb(a + t * (b - a)),
            (LengthValue::Vmin(a), LengthValue::Vmin(b)) => LengthValue::Vmin(a + t * (b - a)),
            (LengthValue::Svmin(a), LengthValue::Svmin(b)) => LengthValue::Svmin(a + t * (b - a)),
            (LengthValue::Lvmin(a), LengthValue::Lvmin(b)) => LengthValue::Lvmin(a + t * (b - a)),
            (LengthValue::Dvmin(a), LengthValue::Dvmin(b)) => LengthValue::Dvmin(a + t * (b - a)),
            (LengthValue::Cqmin(a), LengthValue::Cqmin(b)) => LengthValue::Cqmin(a + t * (b - a)),
            (LengthValue::Vmax(a), LengthValue::Vmax(b)) => LengthValue::Vmax(a + t * (b - a)),
            (LengthValue::Svmax(a), LengthValue::Svmax(b)) => LengthValue::Svmax(a + t * (b - a)),
            (LengthValue::Lvmax(a), LengthValue::Lvmax(b)) => LengthValue::Lvmax(a + t * (b - a)),
            (LengthValue::Dvmax(a), LengthValue::Dvmax(b)) => LengthValue::Dvmax(a + t * (b - a)),
            (LengthValue::Cqmax(a), LengthValue::Cqmax(b)) => LengthValue::Cqmax(a + t * (b - a)),
            _ => {
                error!("interpolation from {a:?} to {b:?} not supported");
                a.clone()
            }
        }
    }
}
