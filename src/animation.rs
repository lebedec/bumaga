use crate::css::CssSpan;

#[derive(Clone)]
pub struct Animator {
    pub(crate) name: CssSpan,
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
            name: CssSpan::empty(),
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
