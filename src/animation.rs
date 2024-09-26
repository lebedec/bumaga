use std::collections::HashMap;
use std::mem::take;

use crate::css::ComputedValue::{Color, Dimension, Number};
use crate::css::{
    Animation, AnimationTrack, ComputedStyle, ComputedValue, Css, Dim, Keyframe, PropertyKey,
};
use crate::Rgba;

#[derive(Clone)]
pub struct Transition {
    animator: Animator,
    keyframe: Keyframe,
    target_id: usize,
    current: Option<Vec<ComputedValue>>,
    // Debounced setter of target value (only last value declaration take effect).
    setter: Option<(usize, Vec<ComputedValue>)>,
}

impl Transition {
    // pub fn play(&mut self, css: &Css, time: f32) -> Vec<AnimationResult> {
    //     if let Some((target_id, target)) = take(&mut self.setter) {
    //         self.update_keyframe(target_id, target);
    //     }
    //     let mut result = vec![];
    //     if self.animator.running {
    //         if let Some(t) = self.animator.update(time) {
    //             let t = (t * 100.0) as u32;
    //             let r = self.animator.play_keyframe(css, &self.keyframe, t);
    //             self.current = Some(r.shorthand.clone());
    //             result.push(r);
    //         }
    //     }
    //     result
    // }
    //
    // pub fn set(&mut self, target_id: usize, target: Vec<ComputedValue>) {
    //     self.setter = Some((target_id, target));
    // }
    //
    // fn update_keyframe(&mut self, target_id: usize, target: Vec<ComputedValue>) {
    //     if let Some(current) = self.current.as_ref() {
    //         if target_id != self.target_id {
    //             self.target_id = target_id;
    //             self.animator.time = 0.0;
    //             self.animator.running = true;
    //             self.keyframe.frames.insert(0, current.clone());
    //             self.keyframe.frames.insert(100, target);
    //         }
    //     } else {
    //         self.target_id = target_id;
    //         self.current = Some(target.clone());
    //         self.animator.running = true;
    //         self.keyframe.frames.insert(0, target.clone());
    //         self.keyframe.frames.insert(100, target.clone());
    //     }
    // }
    //
    // pub fn set_timing(&mut self, timing: TimingFunction) {
    //     self.animator.timing = timing;
    // }
    //
    // pub fn set_duration(&mut self, duration: f32) {
    //     self.animator.duration = duration;
    // }
    //
    // pub fn set_delay(&mut self, delay: f32) {
    //     self.animator.delay = delay;
    // }
}

impl Transition {
    // pub fn new(key: PropertyKey) -> Self {
    //     let mut transition = Self {
    //         animator: Animator::default(),
    //         keyframe: Keyframe {
    //             key,
    //             frames: Default::default(),
    //         },
    //         target_id: 0,
    //         current: None,
    //         setter: None,
    //     };
    //     transition.animator.running = false;
    //     transition
    // }
}

#[derive(Clone)]
pub struct Animator {
    pub(crate) name: String,
    ///  Specifies the amount of time in seconds to wait from applying the animation
    /// to an element before beginning to perform the animation
    pub(crate) delay: f32,
    pub(crate) direction: AnimationDirection,
    /// The length of time in seconds that an animation takes to complete one cycle.
    pub(crate) duration: f32,
    pub(crate) fill_mode: AnimationFillMode,
    pub(crate) timing: TimingFunction,
    pub(crate) iterations: AnimationIterations,
    pub(crate) running: bool,
    pub(crate) time: f32,
}

impl Default for Animator {
    fn default() -> Self {
        Self {
            name: String::new(),
            delay: 0.0,
            direction: AnimationDirection::Normal,
            duration: 0.0,
            fill_mode: AnimationFillMode::None,
            timing: TimingFunction::Linear,
            iterations: AnimationIterations::Number(1.0),
            running: true,
            time: 0.0,
        }
    }
}

impl Animator {
    pub fn play(&mut self, time: f32, tracks: &Vec<AnimationTrack>, style: &mut ComputedStyle) {
        if let Some(time) = self.update(time) {
            let step = (time * 100.0) as u32;
            for track in tracks {
                self.play_track(track, step, style);
            }
        }
    }

    pub fn play_track(&self, track: &AnimationTrack, t: u32, style: &mut ComputedStyle) {
        let mut a = 0;
        let mut b = 0;
        for step in track.frames.keys() {
            if t >= *step {
                a = *step;
            }
            if t <= *step {
                b = *step;
                break;
            }
        }
        if b > a {
            let t = (t - a) as f32 / (b - a) as f32;
            let a = &track.frames[&a];
            let b = &track.frames[&b];
            let result = animate(track.descriptor.key, a, b, t);
            style.insert(track.descriptor, result);
        } else {
            // last or exact frame (no interpolation)
            let result = track.frames[&a].clone();
            style.insert(track.descriptor, result);
        }
    }

    pub fn update(&mut self, time: f32) -> Option<f32> {
        if self.running && self.duration > 0.0 {
            self.time += time;
        }
        let t = self.time - self.delay;
        if t < 0.0 {
            return None;
        }
        let mut t = match self.iterations {
            AnimationIterations::Number(iterations) => t.min(iterations * self.duration),
            AnimationIterations::Infinite => t,
        };
        // use this loop instead of % to stop at last frame 1.0 (100%)
        while t > self.duration {
            t -= self.duration;
        }
        let x = t / self.duration;
        Some(x)
    }
}

pub fn animate(key: PropertyKey, a: &ComputedValue, b: &ComputedValue, t: f32) -> ComputedValue {
    match (a, b) {
        (Number(a), Number(b)) => number(a, b, t),
        (Dimension(a), Dimension(b)) => dimension(a, b, t),
        (Color(a), Color(b)) => color(a, b, t),
        (a, b) => {
            // discrete
            (if t < 0.5 { a } else { b }).clone()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Clone, Copy, Debug)]
pub enum AnimationFillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Clone, Copy, Debug)]
pub enum TimingFunction {
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    Linear,
    StepStart,
    StepEnd,
    Steps(u8, Jump),
    CubicBezier(f32, f32, f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub enum Jump {
    None,
    Start,
    End,
    Both,
}

#[derive(Clone, Copy, Debug)]
pub enum AnimationIterations {
    Number(f32),
    Infinite,
}

fn color(x: &Rgba, y: &Rgba, t: f32) -> ComputedValue {
    let r = (x[0] as f32 + (y[0] as f32 - x[0] as f32) * t).max(0.0) as u8;
    let g = (x[1] as f32 + (y[1] as f32 - x[1] as f32) * t).max(0.0) as u8;
    let b = (x[2] as f32 + (y[2] as f32 - x[2] as f32) * t).max(0.0) as u8;
    let a = (x[3] as f32 + (y[3] as f32 - x[3] as f32) * t).max(0.0) as u8;
    Color([r, g, b, a])
}

fn dimension(a: &Dim, b: &Dim, t: f32) -> ComputedValue {
    // TODO: convertable units
    if a.unit != b.unit {
        Dimension(if t < 0.5 { *a } else { *b })
    } else {
        Dimension(Dim {
            value: a.value + (b.value - a.value) * t,
            unit: a.unit,
        })
    }
}

fn number(a: &f32, b: &f32, t: f32) -> ComputedValue {
    Number(a + (b - a) * t)
}

fn transform(_a: &[ComputedValue], _b: &[ComputedValue], _t: f32) -> Vec<ComputedValue> {
    unimplemented!()
}
