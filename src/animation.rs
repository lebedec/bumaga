use crate::css::{Animation, Property, PropertyKey, Str, Value, Values};

#[derive(Clone)]
pub struct Transition {
    animator: Animator,
    current: Value,
    next: Value,
    property: PropertyKey,
}

impl Transition {
    pub fn play(&mut self, time: f32) -> Property {
        unimplemented!()
    }

    pub fn set(&mut self, next: &Value) {
        self.next = next.clone();
    }
}

#[derive(Clone)]
pub struct Animator {
    pub(crate) name: Str,
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
            name: Str::empty(),
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
    pub fn is_declared(&self) -> bool {
        !self.name.is_empty()
    }

    pub fn update(&mut self, time: f32) -> Option<f32> {
        if self.running && self.duration > 0.0 {
            self.time += time;
        }
        let t = self.time - self.delay;
        if t < 0.0 {
            return None;
        }
        let t = match self.iterations {
            AnimationIterations::Number(iterations) => t.min(iterations * self.duration),
            AnimationIterations::Infinite => t,
        };
        let x = (t % self.duration) / self.duration;
        Some(x)
    }

    pub fn play(&mut self, animation: &Animation, time: f32) -> Vec<Property> {
        let mut result = vec![];
        if let Some(t) = self.update(time) {
            let t = (t * 100.0) as u32;
            for keyframe in &animation.keyframes {
                let mut a = 0;
                let mut b = 0;
                for step in keyframe.frames.keys() {
                    if t >= *step {
                        a = *step;
                    }
                    if t <= *step {
                        b = *step;
                    }
                }
                if b > a {
                    let t = (t - a) as f32 / (b - a) as f32;
                    let a = &keyframe.frames[&a];
                    let b = &keyframe.frames[&b];
                    result.push(Property {
                        key: keyframe.key,
                        values: self.interpolate(keyframe.key, a, b, t),
                    });
                } else {
                    // last or exact frame (no interpolation)
                    result.push(Property {
                        key: keyframe.key,
                        values: keyframe.frames[&a].clone(),
                    });
                }
            }
        }
        result
    }

    fn interpolate(&self, key: PropertyKey, a: &Values, b: &Values, t: f32) -> Values {
        a.clone()
        // match key {
        //     CssProperty::Width => {}
        //     CssProperty::Height => {}
        //     CssProperty::Left => {}
        //     CssProperty::Right => {}
        //     CssProperty::Color => {}
        //     CssProperty::BackgroundColor => {}
        //     CssProperty::Transform => {}
        //     // Discrete
        //     // The values are not additive, and interpolation swaps from the start value to the end.
        //     _ => {
        //         if t < 0.5 {
        //             a
        //         } else {
        //             b
        //         }
        //     }
        // }
    }

    // fn interpolate2(&self, a: &MyProperty, b: &MyProperty, t: f32) -> Option<MyProperty> {
    //     let property = match (a.key, a.as_value(), b.key, b.as_value()) {
    //         (CssProperty::Height, CssValue::Dim(a), CssProperty::Height, CssValue::Dim(b)) => {
    //             MyProperty {
    //                 key: CssProperty::Height,
    //                 values: CssValues::One(N1(CssValue::Dim(CssDimension {
    //                     value: a.value + (b.value - a.value) * t,
    //                     unit: a.unit,
    //                 }))),
    //             }
    //         }
    //         (CssProperty::Width, CssValue::Dim(a), CssProperty::Width, CssValue::Dim(b)) => {
    //             MyProperty {
    //                 key: CssProperty::Width,
    //                 values: CssValues::One(N1(CssValue::Dim(CssDimension {
    //                     value: a.value + (b.value - a.value) * t,
    //                     unit: a.unit,
    //                 }))),
    //             }
    //         }
    //         (
    //             CssProperty::BackgroundColor,
    //             CssValue::Color(x),
    //             CssProperty::BackgroundColor,
    //             CssValue::Color(y),
    //         ) => {
    //             let r = (x[0] as f32 + (y[0] as f32 - x[0] as f32) * t).max(0.0) as u8;
    //             let g = (x[1] as f32 + (y[1] as f32 - x[1] as f32) * t).max(0.0) as u8;
    //             let b = (x[2] as f32 + (y[2] as f32 - x[2] as f32) * t).max(0.0) as u8;
    //             let a = (x[3] as f32 + (y[3] as f32 - x[3] as f32) * t).max(0.0) as u8;
    //             MyProperty {
    //                 key: CssProperty::BackgroundColor,
    //                 values: CssValues::One(N1(CssValue::Color([r, g, b, a]))),
    //             }
    //         }
    //         (CssProperty::Color, CssValue::Color(x), CssProperty::Color, CssValue::Color(y)) => {
    //             let r = (x[0] as f32 + (y[0] as f32 - x[0] as f32) * t).max(0.0) as u8;
    //             let g = (x[1] as f32 + (y[1] as f32 - x[1] as f32) * t).max(0.0) as u8;
    //             let b = (x[2] as f32 + (y[2] as f32 - x[2] as f32) * t).max(0.0) as u8;
    //             let a = (x[3] as f32 + (y[3] as f32 - x[3] as f32) * t).max(0.0) as u8;
    //             MyProperty {
    //                 key: CssProperty::Color,
    //                 values: CssValues::One(N1(CssValue::Color([r, g, b, a]))),
    //             }
    //         }
    //         _ => return None,
    //     };
    //     Some(property)
    // }
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
