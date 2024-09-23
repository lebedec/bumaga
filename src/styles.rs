use std::collections::HashMap;


use log::error;

use taffy::{
    Dimension, Layout, LengthPercentage, LengthPercentageAuto, NodeId, Overflow, Point, Rect, TaffyTree,
};

use crate::animation::{
    AnimationDirection, AnimationFillMode, AnimationIterations, AnimationResult, Animator,
    TimingFunction, Transition,
};
use crate::css::Value::{Keyword, Number, Time};
use crate::css::{
    match_style, Css, Dim, PropertyKey, PseudoClassMatcher, Str, Style, Value, Values,
    Var,
};

use crate::{
    Background, Borders, Element, FontFace, Input, Length, ObjectFit, PointerEvents,
    TextAlign, TransformFunction,
};

impl FontFace {
    pub const DEFAULT_FONT_FAMILY: &'static str = "system-ui";
    pub const DEFAULT_FONT_WEIGHT: u16 = 400;
    // pub const DEFAULT_FONT_STRETCH: FontStretchKeyword = FontStretchKeyword::Normal;
}

pub fn initial(element: &mut Element) {
    // TODO: extract element style to struct
    element.background = Background {
        image: None,
        color: [0; 4],
    };
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
        line_height: 16.0,
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
        position: [0.0; 2],
        size: [0.0; 2],
        content_size: [0.0; 2],
        object_fit: ObjectFit::Fill,
        background: Background {
            image: None,
            color: [0; 4],
        },
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
            line_height: 16.0,
            // wrap: OverflowWrap::Normal,
            align: TextAlign::Start,
        },
        listeners: Default::default(),
        opacity: 1.0,
        transforms: vec![],
        animator: Animator::default(),
        scrolling: None,
        clipping: None,
        transitions: HashMap::default(),
        state: Default::default(),
        pointer_events: Default::default(),
    }
}

#[derive(Default, Clone, Debug)]
pub struct Scrolling {
    pub x: f32,
    pub y: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl Scrolling {
    pub fn ensure(layout: &Layout, current: &Option<Scrolling>) -> Option<Scrolling> {
        let content = layout.content_size;
        let size = layout.size;
        let [x, y] = current
            .as_ref()
            .map(|current| [current.x, current.y])
            .unwrap_or_default();
        if content.width > size.width || content.height > size.height {
            let scroll_x = content.width - size.width;
            let scroll_y = content.height - size.height;
            let scrolling = Scrolling {
                x: x.min(scroll_x),
                y: y.min(scroll_y),
                scroll_x,
                scroll_y,
            };
            Some(scrolling)
        } else {
            None
        }
    }

    pub fn offset(&mut self, wheel: [f32; 2]) {
        let [x, y] = wheel;
        if x != 0.0 {
            self.x += x.signum() * 50.0;
            self.x = self.x.min(self.scroll_x).max(0.0);
        }
        if y != 0.0 {
            self.y -= y.signum() * 50.0;
            self.y = self.y.min(self.scroll_y).max(0.0);
        }
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

pub fn inherit(parent: &Element, element: &mut Element) {
    // border-collapse
    // border-spacing
    // caption-side
    // color
    element.color = parent.color;

    // cursor
    // direction
    // empty-cells
    // font-family
    element.font.family = parent.font.family.clone();
    // font-size
    element.font.size = parent.font.size;
    // font-style
    element.font.style = parent.font.style.clone();
    // font-variant
    // font-weight
    element.font.weight = parent.font.weight;
    // font-size-adjust
    // font-stretch
    //view.text_style.font_stretch = parent.text_style.font_stretch.clone();
    // font
    // letter-spacing
    // line-height
    element.font.line_height = parent.font.line_height;
    // list-style-image
    // list-style-position
    // list-style-type
    // list-style
    // orphans
    // quotes
    // tab-size
    // text-align
    // text-align-last
    // text-decoration-color
    // text-indent
    // text-justify
    // text-shadow
    // text-transform
    // visibility
    // white-space
    // widows
    // word-break
    // word-spacing
    // word-wrap
    //view.text_style.wrap = parent.text_style.wrap;
    element.pointer_events = parent.pointer_events;
}

/// The cascade is an algorithm that defines how to combine CSS (Cascading Style Sheets)
/// property values originating from different sources.
pub struct Cascade<'c> {
    css: &'c Css,
    variables: HashMap<&'c str, &'c Values>,
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

    pub fn push_variable(&mut self, name: Str, values: &'c Values) {
        self.variables.insert(name.as_str(&self.css.source), values);
    }

    pub fn get_variable_value(&self, variable: &Var) -> Result<&Value, CascadeError> {
        let name = variable.name.as_str(&self.css.source);
        self.variables
            .get(name)
            .map(|values| self.css.as_value(values))
            .ok_or(CascadeError::VariableNotFound)
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
        initial(element);
        // 0: inheritance
        inherit(parent, element);
        let css = &self.css.source;
        // 1: css rules
        for style in &self.css.styles {
            if match_style(css, &style, node, tree, matcher) {
                self.apply_style(style, parent, layout, element);
            }
        }
        // 2: transitions
        let time = input.time.as_secs_f32();
        let transitions: Vec<AnimationResult> = element
            .transitions
            .values_mut()
            .flat_map(|transition| transition.play(self.css, time))
            .collect();
        for play in transitions {
            let result = self.apply_shorthand(play.key, &play.shorthand, layout, element);
            if let Err(error) = result {
                error!(
                    "unable to apply transition result of {:?}, {:?}, {:?}",
                    play.key, play.shorthand, error
                );
            }
        }
        // 3: animations
        if !element.animator.name.is_empty() {
            let key = element.animator.name.as_str(css);
            if let Some(animation) = self.css.animations.get(key) {
                for play in element.animator.play(self.css, animation, time) {
                    let result = self.apply_shorthand(play.key, &play.shorthand, layout, element);
                    if let Err(error) = result {
                        error!(
                            "unable to apply animation result of {:?}, {:?}, {:?}",
                            play.key, play.shorthand, error
                        );
                    }
                }
            }
        }
    }

    fn apply_style(
        &mut self,
        style: &'c Style,
        _parent: &Element,
        layout: &mut taffy::Style,
        element: &mut Element,
    ) {
        for property in &style.declaration {
            if let PropertyKey::Variable(name) = property.key {
                self.push_variable(name, &property.values);
                continue;
            }
            if PropertyKey::Transition == property.key {
                for shorthand in property.values.to_vec() {
                    let shorthand = self.css.get_shorthand(shorthand);
                    let key = match shorthand[0] {
                        Keyword(name) => {
                            let key = PropertyKey::parse(name, &self.css.source);
                            if key.is_css_property() {
                                key
                            } else {
                                error!("invalid transition property value");
                                continue;
                            }
                        }
                        _ => {
                            error!("invalid transition property value");
                            continue;
                        }
                    };
                    let transition = element
                        .transitions
                        .entry(key)
                        .or_insert_with(|| Transition::new(key));
                    match &shorthand[1..] {
                        [Time(duration)] => {
                            transition.set_duration(*duration);
                        }
                        [Time(duration), timing] => {
                            transition.set_duration(*duration);
                            transition.set_timing(resolve_timing(&timing, self).unwrap());
                        }
                        [Time(duration), timing, Time(delay)] => {
                            transition.set_duration(*duration);
                            transition.set_timing(resolve_timing(&timing, self).unwrap());
                            transition.set_delay(*delay);
                        }
                        shorthand => {
                            error!("transition value not supported {shorthand:?}");
                            continue;
                        }
                    }
                }
                continue;
            }
            if let Some(transition) = element.transitions.get_mut(&property.key) {
                let shorthand = self.css.as_shorthand(&property.values);
                transition.set(property.values.id(), shorthand);
                continue;
            }
            if let Err(error) = self.apply_shorthand(
                property.key,
                self.css.as_shorthand(&property.values),
                layout,
                element,
            ) {
                error!("unable to apply property {property:?}, {error:?}")
            }
        }
    }

    fn apply_shorthand(
        &mut self,
        key: PropertyKey,
        shorthand: &[Value],
        layout: &mut taffy::Style,
        element: &mut Element,
    ) -> Result<(), CascadeError> {
        let css = &self.css.source;
        let _ctx = self.sizes;
        match (key, shorthand) {
            //
            // Element
            //
            (PropertyKey::Background, [color]) => {
                element.background.color = resolve_color(color, self)?
            }
            (PropertyKey::BackgroundColor, [color]) => {
                element.background.color = resolve_color(color, self)?
            }
            (PropertyKey::Color, [color]) => {
                element.color = resolve_color(color, self)?;
            }
            (PropertyKey::FontSize, [size]) => {
                element.font.size = resolve_length(size, self, self.sizes.parent_font_size)?;
            }
            (PropertyKey::FontWeight, [value]) => {
                element.font.weight = resolve_font_weight(value, self)?
            }
            (PropertyKey::FontFamily, [value]) => {
                element.font.family = resolve_string(value, self)?;
            }
            (PropertyKey::FontStyle, [Keyword(keyword)]) => {
                element.font.style = match keyword.as_str(css) {
                    "normal" => "normal".to_string(),
                    "italic" => "italic".to_string(),
                    "oblique" => "oblique".to_string(),
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::TextAlign, [Keyword(keyword)]) => {
                element.font.align = match keyword.as_str(css) {
                    "start" => TextAlign::Start,
                    "end" => TextAlign::End,
                    "left" => TextAlign::Left,
                    "right" => TextAlign::Right,
                    "center" => TextAlign::Center,
                    "justify" => TextAlign::Justify,
                    "justify-all" => TextAlign::JustifyAll,
                    "match-parent" => TextAlign::MatchParent,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::Border, [width, _style, color]) => {
                element.borders.top.width = dimension_length(width, self)?;
                element.borders.top.color = resolve_color(color, self)?;
                element.borders.right = element.borders.top;
                element.borders.bottom = element.borders.top;
                element.borders.left = element.borders.top;
            }
            (PropertyKey::BorderTop, [width, _style, color]) => {
                element.borders.top.width = dimension_length(width, self)?;
                element.borders.top.color = resolve_color(color, self)?;
            }
            (PropertyKey::BorderRight, [width, _style, color]) => {
                element.borders.right.width = dimension_length(width, self)?;
                element.borders.right.color = resolve_color(color, self)?;
            }
            (PropertyKey::BorderBottom, [width, _style, color]) => {
                element.borders.bottom.width = dimension_length(width, self)?;
                element.borders.bottom.color = resolve_color(color, self)?;
            }
            (PropertyKey::BorderLeft, [width, _style, color]) => {
                element.borders.left.width = dimension_length(width, self)?;
                element.borders.left.color = resolve_color(color, self)?;
            }
            (PropertyKey::BorderWidth, [top, right, bottom, left]) => {
                element.borders.top.width = dimension_length(top, self)?;
                element.borders.right.width = dimension_length(right, self)?;
                element.borders.bottom.width = dimension_length(bottom, self)?;
                element.borders.left.width = dimension_length(left, self)?;
            }
            (PropertyKey::BorderWidth, [top, h, bottom]) => {
                element.borders.top.width = dimension_length(top, self)?;
                element.borders.right.width = dimension_length(h, self)?;
                element.borders.bottom.width = dimension_length(bottom, self)?;
                element.borders.left.width = dimension_length(h, self)?;
            }
            (PropertyKey::BorderWidth, [v, h]) => {
                element.borders.top.width = dimension_length(v, self)?;
                element.borders.right.width = dimension_length(h, self)?;
                element.borders.bottom.width = dimension_length(v, self)?;
                element.borders.left.width = dimension_length(h, self)?;
            }
            (PropertyKey::BorderWidth, [value]) => {
                element.borders.top.width = dimension_length(value, self)?;
                element.borders.right.width = element.borders.top.width;
                element.borders.bottom.width = element.borders.top.width;
                element.borders.left.width = element.borders.top.width;
            }
            (PropertyKey::BorderTopWidth, [value]) => {
                element.borders.top.width = dimension_length(value, self)?;
            }
            (PropertyKey::BorderRightWidth, [value]) => {
                element.borders.right.width = dimension_length(value, self)?;
            }
            (PropertyKey::BorderBottomWidth, [value]) => {
                element.borders.bottom.width = dimension_length(value, self)?;
            }
            (PropertyKey::BorderLeftWidth, [value]) => {
                element.borders.left.width = dimension_length(value, self)?;
            }
            (PropertyKey::BorderColor, [top, right, bottom, left]) => {
                element.borders.top.color = resolve_color(top, self)?;
                element.borders.right.color = resolve_color(right, self)?;
                element.borders.bottom.color = resolve_color(bottom, self)?;
                element.borders.left.color = resolve_color(left, self)?;
            }
            (PropertyKey::BorderColor, [top, h, bottom]) => {
                element.borders.top.color = resolve_color(top, self)?;
                element.borders.right.color = resolve_color(h, self)?;
                element.borders.bottom.color = resolve_color(bottom, self)?;
                element.borders.left.color = resolve_color(h, self)?;
            }
            (PropertyKey::BorderColor, [v, h]) => {
                element.borders.top.color = resolve_color(v, self)?;
                element.borders.right.color = resolve_color(h, self)?;
                element.borders.bottom.color = resolve_color(v, self)?;
                element.borders.left.color = resolve_color(h, self)?;
            }
            (PropertyKey::BorderColor, [value]) => {
                element.borders.top.color = resolve_color(value, self)?;
                element.borders.right.color = element.borders.top.color;
                element.borders.bottom.color = element.borders.top.color;
                element.borders.left.color = element.borders.top.color;
            }
            (PropertyKey::BorderTopColor, [value]) => {
                element.borders.top.color = resolve_color(value, self)?;
            }
            (PropertyKey::BorderRightColor, [value]) => {
                element.borders.right.color = resolve_color(value, self)?;
            }
            (PropertyKey::BorderBottomColor, [value]) => {
                element.borders.bottom.color = resolve_color(value, self)?;
            }
            (PropertyKey::BorderLeftColor, [value]) => {
                element.borders.left.color = resolve_color(value, self)?;
            }
            (PropertyKey::BorderRadius, [a, b, c, d]) => {
                element.borders.radius[0] = length(a, self)?;
                element.borders.radius[1] = length(b, self)?;
                element.borders.radius[2] = length(c, self)?;
                element.borders.radius[3] = length(d, self)?;
            }
            (PropertyKey::BorderRadius, [a, b, c]) => {
                element.borders.radius[0] = length(a, self)?;
                element.borders.radius[1] = length(b, self)?;
                element.borders.radius[2] = length(c, self)?;
                element.borders.radius[3] = length(b, self)?;
            }
            (PropertyKey::BorderRadius, [a, b]) => {
                element.borders.radius[0] = length(a, self)?;
                element.borders.radius[1] = length(b, self)?;
                element.borders.radius[2] = length(a, self)?;
                element.borders.radius[3] = length(b, self)?;
            }
            (PropertyKey::BorderRadius, [value]) => {
                element.borders.radius[0] = length(value, self)?;
                element.borders.radius[1] = length(value, self)?;
                element.borders.radius[2] = length(value, self)?;
                element.borders.radius[3] = length(value, self)?;
            }
            (PropertyKey::BorderTopLeftRadius, [value]) => {
                element.borders.radius[0] = length(value, self)?;
            }
            (PropertyKey::BorderTopRightRadius, [value]) => {
                element.borders.radius[1] = length(value, self)?;
            }
            (PropertyKey::BorderBottomRightRadius, [value]) => {
                element.borders.radius[2] = length(value, self)?;
            }
            (PropertyKey::BorderBottomLeftRadius, [value]) => {
                element.borders.radius[3] = length(value, self)?;
            }
            (PropertyKey::PointerEvents, [Keyword(keyword)]) => {
                element.pointer_events = match keyword.as_str(css) {
                    "auto" => PointerEvents::Auto,
                    "none" => PointerEvents::None,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            //
            // Transform
            //
            (PropertyKey::Transform, shorthand) => {
                element.transforms = resolve_transforms(shorthand, self)?;
            }
            // Animation
            //
            // there is no static shorthand pattern, we should set values by it type and order
            // TODO: special animation shorthand parser
            (PropertyKey::Animation, [Time(duration), timing, Time(delay), Keyword(name)]) => {
                element.animator.name = *name;
                element.animator.delay = *delay;
                element.animator.duration = *duration;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (PropertyKey::Animation, [Time(duration), timing, iterations, Keyword(name)]) => {
                element.animator.name = *name;
                element.animator.duration = *duration;
                element.animator.iterations = resolve_iterations(iterations, self)?;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (PropertyKey::Animation, [Time(duration), timing, Keyword(name)]) => {
                element.animator.name = *name;
                element.animator.duration = *duration;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (PropertyKey::Animation, [Time(duration), Keyword(name)]) => {
                element.animator.name = *name;
                element.animator.duration = *duration;
            }
            (PropertyKey::AnimationName, [Keyword(name)]) => {
                element.animator.name = *name;
            }
            (PropertyKey::AnimationDelay, [Time(delay)]) => {
                element.animator.delay = *delay;
            }
            (PropertyKey::AnimationDirection, [Keyword(keyword)]) => {
                element.animator.direction = match keyword.as_str(css) {
                    "normal" => AnimationDirection::Normal,
                    "reverse" => AnimationDirection::Reverse,
                    "alternate" => AnimationDirection::Alternate,
                    "alternate-reverse" => AnimationDirection::AlternateReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationDuration, [Time(duration)]) => {
                element.animator.duration = *duration;
            }
            (PropertyKey::AnimationFillMode, [Keyword(keyword)]) => {
                element.animator.fill_mode = match keyword.as_str(css) {
                    "none" => AnimationFillMode::None,
                    "forwards" => AnimationFillMode::Forwards,
                    "backwards" => AnimationFillMode::Backwards,
                    "both" => AnimationFillMode::Both,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationIterationCount, [iterations]) => {
                element.animator.iterations = resolve_iterations(iterations, self)?;
            }
            (PropertyKey::AnimationPlayState, [Keyword(keyword)]) => {
                element.animator.running = match keyword.as_str(css) {
                    "running" => true,
                    "paused" => false,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationTimingFunction, [timing]) => {
                element.animator.timing = resolve_timing(timing, self)?
            }
            //
            // Layout
            //
            (PropertyKey::Display, [Keyword(keyword)]) => match keyword.as_str(css) {
                "flow" => layout.display = taffy::Display::Block,
                "block" => layout.display = taffy::Display::Block,
                "flex" => layout.display = taffy::Display::Flex,
                "grid" => layout.display = taffy::Display::Grid,
                keyword => return CascadeError::invalid_keyword(keyword),
            },
            (PropertyKey::Overflow, [Keyword(value)]) => {
                layout.overflow.x = resolve_overflow(value.as_str(css))?;
                layout.overflow.y = layout.overflow.x;
            }
            (PropertyKey::Overflow, [Keyword(x), Keyword(y)]) => {
                layout.overflow.x = resolve_overflow(x.as_str(css))?;
                layout.overflow.y = resolve_overflow(y.as_str(css))?;
            }
            (PropertyKey::OverflowX, [Keyword(x)]) => {
                layout.overflow.x = resolve_overflow(x.as_str(css))?
            }
            (PropertyKey::OverflowY, [Keyword(y)]) => {
                layout.overflow.y = resolve_overflow(y.as_str(css))?
            }
            (PropertyKey::Position, [Keyword(keyword)]) => match keyword.as_str(css) {
                "relative" => layout.position = taffy::Position::Relative,
                "absolute" => layout.position = taffy::Position::Absolute,
                keyword => return CascadeError::invalid_keyword(keyword),
            },
            (PropertyKey::Inset, [top, right, bottom, left]) => {
                layout.inset.top = lengthp_auto(top, self)?;
                layout.inset.right = lengthp_auto(right, self)?;
                layout.inset.bottom = lengthp_auto(bottom, self)?;
                layout.inset.left = lengthp_auto(left, self)?;
            }
            (PropertyKey::Left, [value]) => layout.inset.left = lengthp_auto(value, self)?,
            (PropertyKey::Right, [value]) => layout.inset.right = lengthp_auto(value, self)?,
            (PropertyKey::Top, [value]) => layout.inset.top = lengthp_auto(value, self)?,
            (PropertyKey::Bottom, [value]) => layout.inset.bottom = lengthp_auto(value, self)?,
            (PropertyKey::Width, [value]) => layout.size.width = dimension(value, self)?,
            (PropertyKey::Height, [value]) => layout.size.height = dimension(value, self)?,
            (PropertyKey::MinWidth, [value]) => layout.min_size.width = dimension(value, self)?,
            (PropertyKey::MinHeight, [value]) => layout.min_size.height = dimension(value, self)?,
            (PropertyKey::MaxWidth, [value]) => layout.max_size.width = dimension(value, self)?,
            (PropertyKey::MaxHeight, [value]) => layout.max_size.height = dimension(value, self)?,
            (PropertyKey::AspectRatio, _) => {
                // TODO:
                // layout.aspect_ratio = None;
                return Err(CascadeError::PropertyNotSupported);
            }
            (PropertyKey::Margin, [top, right, bottom, left]) => {
                layout.margin.top = lengthp_auto(top, self)?;
                layout.margin.right = lengthp_auto(right, self)?;
                layout.margin.bottom = lengthp_auto(bottom, self)?;
                layout.margin.left = lengthp_auto(left, self)?;
            }
            (PropertyKey::Margin, [top, horizontal, bottom]) => {
                layout.margin.top = lengthp_auto(top, self)?;
                layout.margin.right = lengthp_auto(horizontal, self)?;
                layout.margin.bottom = lengthp_auto(bottom, self)?;
                layout.margin.left = lengthp_auto(horizontal, self)?;
            }
            (PropertyKey::Margin, [vertical, horizontal]) => {
                layout.margin.top = lengthp_auto(vertical, self)?;
                layout.margin.right = lengthp_auto(horizontal, self)?;
                layout.margin.bottom = lengthp_auto(vertical, self)?;
                layout.margin.left = lengthp_auto(horizontal, self)?;
            }
            (PropertyKey::Margin, [value]) => {
                layout.margin.top = lengthp_auto(value, self)?;
                layout.margin.right = lengthp_auto(value, self)?;
                layout.margin.bottom = lengthp_auto(value, self)?;
                layout.margin.left = lengthp_auto(value, self)?;
            }
            (PropertyKey::MarginTop, [value]) => {
                layout.margin.top = lengthp_auto(value, self)?;
            }
            (PropertyKey::MarginRight, [value]) => {
                layout.margin.right = lengthp_auto(value, self)?;
            }
            (PropertyKey::MarginBottom, [value]) => {
                layout.margin.bottom = lengthp_auto(value, self)?;
            }
            (PropertyKey::MarginLeft, [value]) => {
                layout.margin.left = lengthp_auto(value, self)?;
            }

            (PropertyKey::Padding, [top, right, bottom, left]) => {
                layout.padding.top = lengthp(top, self)?;
                layout.padding.right = lengthp(right, self)?;
                layout.padding.bottom = lengthp(bottom, self)?;
                layout.padding.left = lengthp(left, self)?;
            }
            (PropertyKey::Padding, [top, horizontal, bottom]) => {
                layout.padding.top = lengthp(top, self)?;
                layout.padding.right = lengthp(horizontal, self)?;
                layout.padding.bottom = lengthp(bottom, self)?;
                layout.padding.left = lengthp(horizontal, self)?;
            }
            (PropertyKey::Padding, [vertical, horizontal]) => {
                layout.padding.top = lengthp(vertical, self)?;
                layout.padding.right = lengthp(horizontal, self)?;
                layout.padding.bottom = lengthp(vertical, self)?;
                layout.padding.left = lengthp(horizontal, self)?;
            }
            (PropertyKey::Padding, [value]) => {
                layout.padding.top = lengthp(value, self)?;
                layout.padding.right = lengthp(value, self)?;
                layout.padding.bottom = lengthp(value, self)?;
                layout.padding.left = lengthp(value, self)?;
            }
            (PropertyKey::PaddingTop, [value]) => {
                layout.padding.top = lengthp(value, self)?;
            }
            (PropertyKey::PaddingRight, [value]) => {
                layout.padding.right = lengthp(value, self)?;
            }
            (PropertyKey::PaddingBottom, [value]) => {
                layout.padding.bottom = lengthp(value, self)?;
            }
            (PropertyKey::PaddingLeft, [value]) => {
                layout.padding.left = lengthp(value, self)?;
            }

            /*
            (CssProperty::Border, [top, right, bottom, left]) => {
                layout.border.top = lengthp(top, self)?;
                layout.border.right = lengthp(right, self)?;
                layout.border.bottom = lengthp(bottom, self)?;
                layout.border.left = lengthp(left, self)?;
            }
            (CssProperty::Border, [top, horizontal, bottom]) => {
                layout.border.top = lengthp(top, self)?;
                layout.border.right = lengthp(horizontal, self)?;
                layout.border.bottom = lengthp(bottom, self)?;
                layout.border.left = lengthp(horizontal, self)?;
            }
            (CssProperty::BorderTopWidth, [value]) => {
                layout.border.top = lengthp(value, self)?;
            }
            (CssProperty::BorderRightWidth, [value]) => {
                layout.border.right = lengthp(value, self)?;
            }
            (CssProperty::BorderLeftWidth, [value]) => {
                layout.border.bottom = lengthp(value, self)?;
            }
            (CssProperty::BorderBottomWidth, [value]) => {
                layout.border.left = lengthp(value, self)?;
            }
            (CssProperty::Border, [vertical, horizontal]) => {
                layout.border.top = lengthp(vertical, self)?;
                layout.border.right = lengthp(horizontal, self)?;
                layout.border.bottom = lengthp(vertical, self)?;
                layout.border.left = lengthp(horizontal, self)?;
            }
            (CssProperty::Border, [value]) => {
                layout.border.top = lengthp(value, self)?;
                layout.border.right = lengthp(value, self)?;
                layout.border.bottom = lengthp(value, self)?;
                layout.border.left = lengthp(value, self)?;
            }*/
            (PropertyKey::AlignContent, [Keyword(keyword)]) => {
                layout.align_content = map_align_content(keyword.as_str(css))?
            }
            (PropertyKey::AlignItems, [Keyword(keyword)]) => {
                layout.align_items = map_align_items(keyword.as_str(css))?
            }
            (PropertyKey::AlignSelf, [Keyword(keyword)]) => {
                layout.align_self = map_align_items(keyword.as_str(css))?
            }
            (PropertyKey::JustifyContent, [Keyword(keyword)]) => {
                layout.justify_content = map_align_content(keyword.as_str(css))?
            }
            (PropertyKey::JustifyItems, [Keyword(keyword)]) => {
                layout.justify_items = map_align_items(keyword.as_str(css))?
            }
            (PropertyKey::JustifySelf, [Keyword(keyword)]) => {
                layout.justify_self = map_align_items(keyword.as_str(css))?
            }
            (PropertyKey::Gap, [column, row]) => {
                layout.gap.width = lengthp(column, self)?;
                layout.gap.height = lengthp(row, self)?;
            }
            (PropertyKey::Gap, [gap]) => {
                layout.gap.width = lengthp(gap, self)?;
                layout.gap.height = lengthp(gap, self)?;
            }
            (PropertyKey::ColumnGap, [column]) => {
                layout.gap.width = lengthp(column, self)?;
            }
            (PropertyKey::RowGap, [row]) => {
                layout.gap.height = lengthp(row, self)?;
            }
            (PropertyKey::FlexDirection, [Keyword(keyword)]) => {
                layout.flex_direction = match keyword.as_str(css) {
                    "row" => taffy::FlexDirection::Row,
                    "row-reverse" => taffy::FlexDirection::RowReverse,
                    "column" => taffy::FlexDirection::Column,
                    "column-reverse" => taffy::FlexDirection::ColumnReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::FlexWrap, [Keyword(keyword)]) => {
                layout.flex_wrap = match keyword.as_str(css) {
                    "wrap" => taffy::FlexWrap::Wrap,
                    "nowrap" => taffy::FlexWrap::NoWrap,
                    "wrap-reverse" => taffy::FlexWrap::WrapReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::FlexBasis, [value]) => layout.flex_basis = dimension(value, self)?,
            (PropertyKey::FlexGrow, [Number(value)]) => layout.flex_grow = *value,
            (PropertyKey::FlexShrink, [Number(value)]) => layout.flex_shrink = *value,
            _ => return Err(CascadeError::PropertyNotSupported),
        }
        Ok(())
    }

    // fn apply_property(
    //     &mut self,
    //     property: &Property,
    //     layout: &mut LayoutStyle,
    //     element: &mut Element,
    // ) -> Result<(), CascadeError> {
    //     let css = &self.css.source;
    //     let ctx = self.sizes;
    //     self.apply_shorthand(
    //         property.key,
    //         self.css.as_shorthand(&property.values),
    //         layout,
    //         element,
    //     )
    // }
}

fn resolve_font_weight(value: &Value, cascade: &Cascade) -> Result<u16, CascadeError> {
    let value = match value {
        Value::Number(value) if *value >= 1.0 && *value <= 1000.0 => *value as u16,
        Value::Keyword(keyword) => match keyword.as_str(&cascade.css.source) {
            "normal" => 400,
            "bold" => 700,
            keyword => return Err(CascadeError::InvalidKeyword(keyword.to_string())),
        },
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_font_weight(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_color(value: &Value, cascade: &Cascade) -> Result<[u8; 4], CascadeError> {
    let value = match value {
        Value::Color(color) => *color,
        Value::Keyword(keyword) => match keyword.as_str(&cascade.css.source) {
            "black" => [0, 0, 0, 255],
            "white" => [255, 255, 255, 255],
            "red" => [255, 0, 0, 255],
            "blue" => [0, 0, 255, 255],
            "green" => [0, 255, 0, 255],
            "transparent" => [0, 0, 0, 0],
            keyword => return Err(CascadeError::InvalidKeyword(keyword.to_string())),
        },
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_color(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_timing(value: &Value, cascade: &Cascade) -> Result<TimingFunction, CascadeError> {
    let value = match value {
        Keyword(keyword) => match keyword.as_str(&cascade.css.source) {
            "ease" => TimingFunction::Ease,
            "ease-in" => TimingFunction::EaseIn,
            "ease-out" => TimingFunction::EaseOut,
            "ease-in-out" => TimingFunction::EaseInOut,
            "linear" => TimingFunction::Linear,
            "step-start" => TimingFunction::StepStart,
            "step-end" => TimingFunction::StepEnd,
            _ => return Err(CascadeError::ValueNotSupported),
        },
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_timing(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_transforms(
    values: &[Value],
    cascade: &Cascade,
) -> Result<Vec<TransformFunction>, CascadeError> {
    let mut transforms = vec![];
    for value in values.iter() {
        match value {
            Value::Function(function) => match cascade.css.as_function(function) {
                ("translate", [x]) => {
                    let x = length(x, cascade)?;
                    let y = Length::zero();
                    let z = 0.0;
                    transforms.push(TransformFunction::translate(x, y, z))
                }
                ("translate", [x, y]) => {
                    let x = length(x, cascade)?;
                    let y = length(y, cascade)?;
                    let _z = 0.0;
                    transforms.push(TransformFunction::translate(x, y, 0.0))
                }
                ("translate3d", [x, y, z]) => {
                    let x = length(x, cascade)?;
                    let y = length(y, cascade)?;
                    let z = dimension_length(z, cascade)?;
                    transforms.push(TransformFunction::translate(x, y, z))
                }
                ("translateX", [x]) => {
                    let x = length(x, cascade)?;
                    let y = Length::zero();
                    let z = 0.0;
                    transforms.push(TransformFunction::translate(x, y, z))
                }
                ("translateY", [y]) => {
                    let x = Length::zero();
                    let y = length(y, cascade)?;
                    let z = 0.0;
                    transforms.push(TransformFunction::translate(x, y, z))
                }
                ("translateZ", [z]) => {
                    let x = Length::zero();
                    let y = Length::zero();
                    let z = dimension_length(z, cascade)?;
                    transforms.push(TransformFunction::translate(x, y, z))
                }
                _ => return Err(CascadeError::TransformFunctionNotSupported),
            },
            _ => return Err(CascadeError::ValueNotSupported),
        }
    }
    Ok(transforms)
}

fn resolve_iterations(
    value: &Value,
    cascade: &Cascade,
) -> Result<AnimationIterations, CascadeError> {
    let value = match value {
        Keyword(keyword) => match cascade.css.as_str(*keyword) {
            "infinite" => AnimationIterations::Infinite,
            _ => return Err(CascadeError::ValueNotSupported),
        },
        Number(number) => AnimationIterations::Number(*number),
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_iterations(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_string(value: &Value, cascade: &Cascade) -> Result<String, CascadeError> {
    let value = match value {
        Value::String(value) => value.as_str(&cascade.css.source).to_string(),
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_string(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_length(value: &Value, cascade: &Cascade, base: f32) -> Result<f32, CascadeError> {
    let value = match value {
        Value::Zero => 0.0,
        Value::Dimension(dimension) => parse_dimension_length(dimension, cascade)?,
        Value::Percentage(percent) => percent * base,
        Number(value) => *value,
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_length(value, cascade, base);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn dimension_length(value: &Value, cascade: &Cascade) -> Result<f32, CascadeError> {
    let value = match value {
        Value::Zero => 0.0,
        Value::Dimension(dimension) => parse_dimension_length(dimension, cascade)?,
        Number(value) => *value,
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return dimension_length(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn parse_dimension_length(dimension: &Dim, cascade: &Cascade) -> Result<f32, CascadeError> {
    let value = dimension.value;
    let sizes = cascade.sizes;
    let value = match dimension.unit.as_str() {
        "px" => value,
        "em" => sizes.parent_font_size * value,
        "rem" => sizes.root_font_size * value,
        "vw" => sizes.viewport_width * value / 100.0,
        "vh" => sizes.viewport_height * value / 100.0,
        _ => {
            return Err(CascadeError::DimensionUnitsNotSupported);
        }
    };
    Ok(value)
}

fn dimension(value: &Value, cascade: &Cascade) -> Result<Dimension, CascadeError> {
    let value = match value {
        Value::Dimension(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            Dimension::Length(length)
        }
        Value::Percentage(value) => Dimension::Percent(*value),
        Keyword(keyword) if keyword.as_str(&cascade.css.source) == "auto" => Dimension::Auto,
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return dimension(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn length(value: &Value, cascade: &Cascade) -> Result<Length, CascadeError> {
    let value = match value {
        Value::Dimension(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            Length::Number(length)
        }
        Value::Percentage(value) => Length::Percent(*value),
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return length(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn lengthp(value: &Value, cascade: &Cascade) -> Result<LengthPercentage, CascadeError> {
    let value = match value {
        Value::Dimension(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            LengthPercentage::Length(length)
        }
        Value::Percentage(value) => LengthPercentage::Percent(*value),
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return lengthp(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn lengthp_auto(value: &Value, cascade: &Cascade) -> Result<LengthPercentageAuto, CascadeError> {
    let value = match value {
        Value::Zero => LengthPercentageAuto::Length(0.0),
        Value::Dimension(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            LengthPercentageAuto::Length(length)
        }
        Value::Percentage(value) => LengthPercentageAuto::Percent(*value),
        Keyword(keyword) if keyword.as_str(&cascade.css.source) == "auto" => {
            LengthPercentageAuto::Auto
        }
        Value::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return lengthp_auto(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_overflow(keyword: &str) -> Result<Overflow, CascadeError> {
    let overflow = match keyword {
        "visible" => Overflow::Visible,
        "hidden" => Overflow::Hidden,
        "clip" => Overflow::Clip,
        "scroll" => Overflow::Scroll,
        "auto" => Overflow::Scroll,
        keyword => return CascadeError::invalid_keyword(keyword),
    };
    Ok(overflow)
}

fn map_align_items(keyword: &str) -> Result<Option<taffy::AlignItems>, CascadeError> {
    let align = match keyword {
        "normal" => return Ok(None),
        "start" => taffy::AlignItems::Start,
        "end" => taffy::AlignItems::End,
        "flex-start" => taffy::AlignItems::FlexStart,
        "flex-end" => taffy::AlignItems::FlexEnd,
        "center" => taffy::AlignItems::Center,
        "baseline" => taffy::AlignItems::Baseline,
        "stretch" => taffy::AlignItems::Stretch,
        keyword => return CascadeError::invalid_keyword(keyword),
    };
    Ok(Some(align))
}

fn map_align_content(keyword: &str) -> Result<Option<taffy::AlignContent>, CascadeError> {
    let align = match keyword {
        "normal" => return Ok(None),
        "start" => taffy::AlignContent::Start,
        "end" => taffy::AlignContent::End,
        "flex-start" => taffy::AlignContent::FlexStart,
        "flex-end" => taffy::AlignContent::FlexEnd,
        "center" => taffy::AlignContent::Center,
        "stretch" => taffy::AlignContent::Stretch,
        "space-between" => taffy::AlignContent::SpaceBetween,
        "space-evenly" => taffy::AlignContent::SpaceEvenly,
        "space-around" => taffy::AlignContent::SpaceAround,
        keyword => return CascadeError::invalid_keyword(keyword),
    };
    Ok(Some(align))
}

#[derive(Clone, Copy)]
pub struct Sizes {
    pub root_font_size: f32,
    pub parent_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}
