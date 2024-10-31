use crate::css::ComputedValue::{Color, Dimension, Number, Percentage, Zero};
use crate::css::{
    AnimationTrack, ComputedStyle, ComputedValue, Dim, PropertyDescriptor, PropertyKey,
};
use crate::Rgba;

#[derive(Clone)]
pub struct Transition {
    pub key: Option<PropertyKey>,
    pub animator: Animator,
    pub range: Option<(ComputedValue, ComputedValue)>,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            key: None,
            animator: Default::default(),
            range: None,
        }
    }
}

impl Transition {
    pub fn init_after_style_applied(&mut self, style: &mut ComputedStyle) {
        if self.range.is_some() {
            return;
        }
        let key = match self.key {
            Some(key) => key,
            None => return,
        };
        let mut index = 0;
        loop {
            let descriptor = PropertyDescriptor::new(key, index);
            let value = match style.get(&descriptor) {
                Some(value) => value,
                None => return,
            };
            self.range = Some((value.clone(), value.clone()));
            index += 1;
        }
    }

    pub fn play(&mut self, time: f32, style: &mut ComputedStyle) {
        let key = match self.key {
            Some(key) => key,
            None => return,
        };
        let time = match self.animator.update(time) {
            Some(time) => time,
            None => return,
        };
        let mut index = 0;
        loop {
            let descriptor = PropertyDescriptor::new(key, index);
            let value = match style.get(&descriptor) {
                Some(value) => value,
                None => return,
            };
            let (from, to) = match self.range.as_ref() {
                Some(range) => range,
                None => {
                    continue;
                }
            };
            let current = animate(key, &from, &to, time);
            if to != value {
                self.range = Some((current.clone(), value.clone()));
                self.animator.restart();
            }
            if &current != value {
                style.insert(descriptor, current);
            }
            index += 1;
        }
    }
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
    pub fn restart(&mut self) {
        self.time = 0.0;
    }

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

pub fn animate(_key: PropertyKey, a: &ComputedValue, b: &ComputedValue, t: f32) -> ComputedValue {
    if t == 0.0 {
        return a.clone();
    }
    if t == 1.0 {
        return b.clone();
    }
    match (a, b) {
        (Number(a), Number(b)) => number(a, b, t),
        (Percentage(a), Percentage(b)) => percentage(a, b, t),
        (Percentage(a), Zero) => {
            let percentage = percentage(a, &0.0, t);
            if Percentage(0.0) == percentage {
                Zero
            } else {
                percentage
            }
        }
        (Zero, Percentage(b)) => percentage(&0.0, b, t),
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
    _Steps(u8, Jump),
    _CubicBezier(f32, f32, f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub enum Jump {
    _None,
    _Start,
    _End,
    _Both,
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

fn percentage(a: &f32, b: &f32, t: f32) -> ComputedValue {
    Percentage(a + (b - a) * t)
}

fn number(a: &f32, b: &f32, t: f32) -> ComputedValue {
    Number(a + (b - a) * t)
}

fn _transform(_a: &[ComputedValue], _b: &[ComputedValue], _t: f32) -> Vec<ComputedValue> {
    unimplemented!()
}
