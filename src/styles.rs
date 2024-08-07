use crate::animation::{
    AnimationDirection, AnimationFillMode, AnimationIterations, Animator, TimingFunction,
};
use crate::css::CssShorthand::{N1, N2, N3, N4};
use crate::css::CssValue::{Color, Dim, Keyword, Number, Time};
use crate::css::{
    match_style, Css, CssDimension, CssProperty, CssSpan, CssValue, CssValues, CssVariable,
    MyProperty, MyStyle,
};
use crate::models::{ElementId, Object, Sizes};
use crate::{
    Background, Borders, Element, Input, Length, MyBorder, ObjectFit, TextStyle, TransformFunction,
};
use log::error;
use std::collections::HashMap;
use taffy::style_helpers::TaffyZero;
use taffy::{
    Dimension, Layout, LengthPercentage, LengthPercentageAuto, NodeId, Overflow, Point, Rect,
    Style, TaffyTree,
};

impl TextStyle {
    pub const DEFAULT_FONT_FAMILY: &'static str = "system-ui";
    pub const DEFAULT_FONT_WEIGHT: u16 = 400;
    // pub const DEFAULT_FONT_STRETCH: FontStretchKeyword = FontStretchKeyword::Normal;
}

pub fn create_element(id: ElementId, html: Object) -> Element {
    Element {
        layout: Default::default(),
        id,
        html,
        object_fit: ObjectFit::Fill,
        background: Background {
            image: None,
            color: Default::default(),
            // position: Default::default(),
            // repeat: Default::default(),
            // size: Default::default(),
            // attachment: Default::default(),
            // clip: Default::default(),
        },
        borders: Borders {
            top: Default::default(),
            bottom: Default::default(),
            right: Default::default(),
            left: Default::default(),
            radius: [Length::zero(); 4],
        },
        color: [255, 255, 255, 255],
        text_style: TextStyle {
            font_family: TextStyle::DEFAULT_FONT_FAMILY.to_string(),
            font_size: 16.0,
            // font_style: FontStyle::Normal,
            font_weight: TextStyle::DEFAULT_FONT_WEIGHT,
            // font_stretch: TextStyle::DEFAULT_FONT_STRETCH,
            line_height: 16.0,
            // wrap: OverflowWrap::Normal,
        },
        listeners: HashMap::new(),
        opacity: 1.0,
        transforms: vec![],
        animator: Animator::default(),
        scrolling: None,
        clip: None,
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

    pub fn offset(&mut self, x: f32, y: f32) {
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

pub fn default_layout_style() -> Style {
    Style {
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
    element.text_style.font_family = parent.text_style.font_family.clone();
    // font-size
    element.text_style.font_size = parent.text_style.font_size;
    // font-style
    //view.text_style.font_style = parent.text_style.font_style.clone();
    // font-variant
    // font-weight
    element.text_style.font_weight = parent.text_style.font_weight;
    // font-size-adjust
    // font-stretch
    //view.text_style.font_stretch = parent.text_style.font_stretch.clone();
    // font
    // letter-spacing
    // line-height
    element.text_style.line_height = parent.text_style.line_height;
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
}

/// The cascade is an algorithm that defines how to combine CSS (Cascading Style Sheets)
/// property values originating from different sources.
pub struct Cascade<'c> {
    css: &'c Css,
    variables: HashMap<&'c str, &'c CssValues>,
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

    pub fn push_variable(&mut self, name: CssSpan, values: &'c CssValues) {
        self.variables.insert(name.as_str(&self.css.source), values);
    }

    pub fn get_variable_value(&self, variable: &CssVariable) -> Result<&CssValue, CascadeError> {
        let name = variable.name.as_str(&self.css.source);
        self.variables
            .get(name)
            .map(|values| values.as_value())
            .ok_or(CascadeError::VariableNotFound)
    }

    pub fn apply_styles(
        &mut self,
        input: &Input,
        node: NodeId,
        tree: &TaffyTree<Element>,
        parent: &Element,
        layout: &mut Style,
        element: &mut Element,
    ) {
        let css = &self.css.source;
        for style in &self.css.styles {
            if match_style(css, &style, node, tree) {
                self.apply_style(style, parent, layout, element);
            }
        }
        if !element.animator.name.is_empty() {
            let key = element.animator.name.as_str(css);
            if let Some(t) = element.animator.update(input.time.as_secs_f32()) {
                if let Some(animation) = self.css.animations.get(key) {
                    let mut a = 0;
                    let mut b = 0;
                    for i in 0..animation.keyframes.len() {
                        let keyframe = &animation.keyframes[i];
                        if keyframe.step <= t {
                            a = i;
                        } else {
                            b = i;
                            break;
                        }
                    }
                    if a + 1 == b {
                        // between a and b
                        // TODO: property tracks
                        for k in 0..animation.keyframes[a].declaration.len() {
                            let sa = animation.keyframes[a].step;
                            let sb = animation.keyframes[b].step;

                            let pa = &animation.keyframes[a].declaration[k];
                            let pb = &animation.keyframes[b].declaration[k];

                            let p = self.interpolate(pa, pb, (t - sa) / (sb - sa)).unwrap();
                            self.apply_property(&p, layout, element).unwrap();
                        }
                    } else {
                        // return a frame (last)
                        let pa = &animation.keyframes[a].declaration[0];
                        self.apply_property(pa, layout, element).unwrap();
                    }
                }
            }
        }
    }

    fn apply_style(
        &mut self,
        style: &'c MyStyle,
        parent: &Element,
        layout: &mut Style,
        element: &mut Element,
    ) {
        inherit(parent, element);
        for property in &style.declaration {
            if let CssProperty::Variable(name) = property.name {
                self.push_variable(name, &property.values);
                continue;
            }
            if let Err(error) = self.apply_property(property, layout, element) {
                error!("unable to apply property {property:?}, {error:?}")
            }
        }
    }

    fn interpolate(&self, a: &MyProperty, b: &MyProperty, t: f32) -> Option<MyProperty> {
        let property = match (a.name, a.as_value(), b.name, b.as_value()) {
            (CssProperty::Height, CssValue::Dim(a), CssProperty::Height, CssValue::Dim(b)) => {
                MyProperty {
                    name: CssProperty::Height,
                    values: CssValues::One(N1(CssValue::Dim(CssDimension {
                        value: a.value + (b.value - a.value) * t,
                        unit: a.unit,
                    }))),
                }
            }
            (CssProperty::Width, CssValue::Dim(a), CssProperty::Width, CssValue::Dim(b)) => {
                MyProperty {
                    name: CssProperty::Width,
                    values: CssValues::One(N1(CssValue::Dim(CssDimension {
                        value: a.value + (b.value - a.value) * t,
                        unit: a.unit,
                    }))),
                }
            }
            (
                CssProperty::BackgroundColor,
                CssValue::Color(x),
                CssProperty::BackgroundColor,
                CssValue::Color(y),
            ) => {
                let r = (x[0] as f32 + (y[0] as f32 - x[0] as f32) * t).max(0.0) as u8;
                let g = (x[1] as f32 + (y[1] as f32 - x[1] as f32) * t).max(0.0) as u8;
                let b = (x[2] as f32 + (y[2] as f32 - x[2] as f32) * t).max(0.0) as u8;
                let a = (x[3] as f32 + (y[3] as f32 - x[3] as f32) * t).max(0.0) as u8;
                MyProperty {
                    name: CssProperty::BackgroundColor,
                    values: CssValues::One(N1(CssValue::Color([r, g, b, a]))),
                }
            }
            (CssProperty::Color, CssValue::Color(x), CssProperty::Color, CssValue::Color(y)) => {
                let r = (x[0] as f32 + (y[0] as f32 - x[0] as f32) * t).max(0.0) as u8;
                let g = (x[1] as f32 + (y[1] as f32 - x[1] as f32) * t).max(0.0) as u8;
                let b = (x[2] as f32 + (y[2] as f32 - x[2] as f32) * t).max(0.0) as u8;
                let a = (x[3] as f32 + (y[3] as f32 - x[3] as f32) * t).max(0.0) as u8;
                MyProperty {
                    name: CssProperty::Color,
                    values: CssValues::One(N1(CssValue::Color([r, g, b, a]))),
                }
            }
            _ => return None,
        };
        Some(property)
    }

    fn apply_property(
        &mut self,
        property: &MyProperty,
        layout: &mut Style,
        element: &mut Element,
    ) -> Result<(), CascadeError> {
        let css = &self.css.source;
        let ctx = self.sizes;
        // TODO: multiple values
        match (property.name, property.values.as_shorthand()) {
            //
            // Element
            //
            (CssProperty::Background, N1(color)) => {
                element.background.color = resolve_color(color, self)?
            }
            (CssProperty::BackgroundColor, N1(color)) => {
                element.background.color = resolve_color(color, self)?
            }
            (CssProperty::Color, N1(color)) => element.color = resolve_color(color, self)?,
            (CssProperty::FontSize, N1(size)) => {
                element.text_style.font_size =
                    resolve_length(size, self, self.sizes.parent_font_size)?;
            }
            (CssProperty::FontFamily, N1(value)) => {
                element.text_style.font_family = resolve_string(value, self)?;
            }
            (CssProperty::Border, N3(width, _style, color)) => {
                element.borders.top.width = dimension_length(width, self)?;
                element.borders.top.color = resolve_color(color, self)?;
                element.borders.right = element.borders.top;
                element.borders.bottom = element.borders.top;
                element.borders.left = element.borders.top;
            }
            (CssProperty::BorderTop, N3(width, _style, color)) => {
                element.borders.top.width = dimension_length(width, self)?;
                element.borders.top.color = resolve_color(color, self)?;
            }
            (CssProperty::BorderRight, N3(width, _style, color)) => {
                element.borders.right.width = dimension_length(width, self)?;
                element.borders.right.color = resolve_color(color, self)?;
            }
            (CssProperty::BorderBottom, N3(width, _style, color)) => {
                element.borders.bottom.width = dimension_length(width, self)?;
                element.borders.bottom.color = resolve_color(color, self)?;
            }
            (CssProperty::BorderLeft, N3(width, _style, color)) => {
                element.borders.left.width = dimension_length(width, self)?;
                element.borders.left.color = resolve_color(color, self)?;
            }
            (CssProperty::BorderWidth, N4(top, right, bottom, left)) => {
                element.borders.top.width = dimension_length(top, self)?;
                element.borders.right.width = dimension_length(right, self)?;
                element.borders.bottom.width = dimension_length(bottom, self)?;
                element.borders.left.width = dimension_length(left, self)?;
            }
            (CssProperty::BorderWidth, N3(top, h, bottom)) => {
                element.borders.top.width = dimension_length(top, self)?;
                element.borders.right.width = dimension_length(h, self)?;
                element.borders.bottom.width = dimension_length(bottom, self)?;
                element.borders.left.width = dimension_length(h, self)?;
            }
            (CssProperty::BorderWidth, N2(v, h)) => {
                element.borders.top.width = dimension_length(v, self)?;
                element.borders.right.width = dimension_length(h, self)?;
                element.borders.bottom.width = dimension_length(v, self)?;
                element.borders.left.width = dimension_length(h, self)?;
            }
            (CssProperty::BorderWidth, N1(value)) => {
                element.borders.top.width = dimension_length(value, self)?;
                element.borders.right.width = element.borders.top.width;
                element.borders.bottom.width = element.borders.top.width;
                element.borders.left.width = element.borders.top.width;
            }
            (CssProperty::BorderTopWidth, N1(value)) => {
                element.borders.top.width = dimension_length(value, self)?;
            }
            (CssProperty::BorderRightWidth, N1(value)) => {
                element.borders.right.width = dimension_length(value, self)?;
            }
            (CssProperty::BorderBottomWidth, N1(value)) => {
                element.borders.bottom.width = dimension_length(value, self)?;
            }
            (CssProperty::BorderLeftWidth, N1(value)) => {
                element.borders.left.width = dimension_length(value, self)?;
            }
            (CssProperty::BorderColor, N4(top, right, bottom, left)) => {
                element.borders.top.color = resolve_color(top, self)?;
                element.borders.right.color = resolve_color(right, self)?;
                element.borders.bottom.color = resolve_color(bottom, self)?;
                element.borders.left.color = resolve_color(left, self)?;
            }
            (CssProperty::BorderColor, N3(top, h, bottom)) => {
                element.borders.top.color = resolve_color(top, self)?;
                element.borders.right.color = resolve_color(h, self)?;
                element.borders.bottom.color = resolve_color(bottom, self)?;
                element.borders.left.color = resolve_color(h, self)?;
            }
            (CssProperty::BorderColor, N2(v, h)) => {
                element.borders.top.color = resolve_color(v, self)?;
                element.borders.right.color = resolve_color(h, self)?;
                element.borders.bottom.color = resolve_color(v, self)?;
                element.borders.left.color = resolve_color(h, self)?;
            }
            (CssProperty::BorderColor, N1(value)) => {
                element.borders.top.color = resolve_color(value, self)?;
                element.borders.right.color = element.borders.top.color;
                element.borders.bottom.color = element.borders.top.color;
                element.borders.left.color = element.borders.top.color;
            }
            (CssProperty::BorderTopColor, N1(value)) => {
                element.borders.top.color = resolve_color(value, self)?;
            }
            (CssProperty::BorderRightColor, N1(value)) => {
                element.borders.right.color = resolve_color(value, self)?;
            }
            (CssProperty::BorderBottomColor, N1(value)) => {
                element.borders.bottom.color = resolve_color(value, self)?;
            }
            (CssProperty::BorderLeftColor, N1(value)) => {
                element.borders.left.color = resolve_color(value, self)?;
            }
            (CssProperty::BorderRadius, N4(a, b, c, d)) => {
                element.borders.radius[0] = length(a, self)?;
                element.borders.radius[1] = length(b, self)?;
                element.borders.radius[2] = length(c, self)?;
                element.borders.radius[3] = length(d, self)?;
            }
            (CssProperty::BorderRadius, N3(a, b, c)) => {
                element.borders.radius[0] = length(a, self)?;
                element.borders.radius[1] = length(b, self)?;
                element.borders.radius[2] = length(c, self)?;
                element.borders.radius[3] = length(b, self)?;
            }
            (CssProperty::BorderRadius, N2(a, b)) => {
                element.borders.radius[0] = length(a, self)?;
                element.borders.radius[1] = length(b, self)?;
                element.borders.radius[2] = length(a, self)?;
                element.borders.radius[3] = length(b, self)?;
            }
            (CssProperty::BorderRadius, N1(value)) => {
                element.borders.radius[0] = length(value, self)?;
                element.borders.radius[1] = length(value, self)?;
                element.borders.radius[2] = length(value, self)?;
                element.borders.radius[3] = length(value, self)?;
            }
            (CssProperty::BorderTopLeftRadius, N1(value)) => {
                element.borders.radius[0] = length(value, self)?;
            }
            (CssProperty::BorderTopRightRadius, N1(value)) => {
                element.borders.radius[1] = length(value, self)?;
            }
            (CssProperty::BorderBottomRightRadius, N1(value)) => {
                element.borders.radius[2] = length(value, self)?;
            }
            (CssProperty::BorderBottomLeftRadius, N1(value)) => {
                element.borders.radius[3] = length(value, self)?;
            }
            //
            // Transform
            //
            (CssProperty::Transform, shorthand) => {
                element.transforms = resolve_transforms(shorthand.values(), self)?;
            }
            // Animation
            //
            // there is no static shorthand pattern, we should set values by it type and order
            // TODO: special animation shorthand parser
            (CssProperty::Animation, N4(Time(duration), timing, Time(delay), Keyword(name))) => {
                element.animator.name = *name;
                element.animator.delay = *delay;
                element.animator.duration = *duration;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (CssProperty::Animation, N4(Time(duration), timing, iterations, Keyword(name))) => {
                element.animator.name = *name;
                element.animator.duration = *duration;
                element.animator.iterations = resolve_iterations(iterations, self)?;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (CssProperty::Animation, N3(Time(duration), timing, Keyword(name))) => {
                element.animator.name = *name;
                element.animator.duration = *duration;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (CssProperty::Animation, N2(Time(duration), Keyword(name))) => {
                element.animator.name = *name;
                element.animator.duration = *duration;
            }
            (CssProperty::AnimationName, N1(Keyword(name))) => {
                element.animator.name = *name;
            }
            (CssProperty::AnimationDelay, N1(Time(delay))) => {
                element.animator.delay = *delay;
            }
            (CssProperty::AnimationDirection, N1(Keyword(keyword))) => {
                element.animator.direction = match keyword.as_str(css) {
                    "normal" => AnimationDirection::Normal,
                    "reverse" => AnimationDirection::Reverse,
                    "alternate" => AnimationDirection::Alternate,
                    "alternate-reverse" => AnimationDirection::AlternateReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (CssProperty::AnimationDuration, N1(Time(duration))) => {
                element.animator.duration = *duration;
            }
            (CssProperty::AnimationFillMode, N1(Keyword(keyword))) => {
                element.animator.fill_mode = match keyword.as_str(css) {
                    "none" => AnimationFillMode::None,
                    "forwards" => AnimationFillMode::Forwards,
                    "backwards" => AnimationFillMode::Backwards,
                    "both" => AnimationFillMode::Both,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (CssProperty::AnimationIterationCount, N1(iterations)) => {
                element.animator.iterations = resolve_iterations(iterations, self)?;
            }
            (CssProperty::AnimationPlayState, N1(Keyword(keyword))) => {
                element.animator.running = match keyword.as_str(css) {
                    "running" => true,
                    "paused" => false,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (CssProperty::AnimationTimingFunction, N1(timing)) => {
                element.animator.timing = resolve_timing(timing, self)?
            }
            //
            // Layout
            //
            (CssProperty::Display, N1(Keyword(keyword))) => match keyword.as_str(css) {
                "flow" => layout.display = taffy::Display::Block,
                "block" => layout.display = taffy::Display::Block,
                "flex" => layout.display = taffy::Display::Flex,
                "grid" => layout.display = taffy::Display::Grid,
                keyword => return CascadeError::invalid_keyword(keyword),
            },
            (CssProperty::Overflow, N1(Keyword(value))) => {
                layout.overflow.x = resolve_overflow(value.as_str(css))?;
                layout.overflow.y = layout.overflow.x;
            }
            (CssProperty::Overflow, N2(Keyword(x), Keyword(y))) => {
                layout.overflow.x = resolve_overflow(x.as_str(css))?;
                layout.overflow.y = resolve_overflow(y.as_str(css))?;
            }
            (CssProperty::OverflowX, N1(Keyword(x))) => {
                layout.overflow.x = resolve_overflow(x.as_str(css))?
            }
            (CssProperty::OverflowY, N1(Keyword(y))) => {
                layout.overflow.y = resolve_overflow(y.as_str(css))?
            }
            (CssProperty::Position, N1(Keyword(keyword))) => match keyword.as_str(css) {
                "relative" => layout.position = taffy::Position::Relative,
                "absolute" => layout.position = taffy::Position::Absolute,
                keyword => return CascadeError::invalid_keyword(keyword),
            },
            (CssProperty::Inset, N4(top, right, bottom, left)) => {
                layout.inset.top = lengthp_auto(top, self)?;
                layout.inset.right = lengthp_auto(right, self)?;
                layout.inset.bottom = lengthp_auto(bottom, self)?;
                layout.inset.left = lengthp_auto(left, self)?;
            }
            (CssProperty::Left, N1(value)) => layout.inset.left = lengthp_auto(value, self)?,
            (CssProperty::Right, N1(value)) => layout.inset.right = lengthp_auto(value, self)?,
            (CssProperty::Top, N1(value)) => layout.inset.top = lengthp_auto(value, self)?,
            (CssProperty::Bottom, N1(value)) => layout.inset.bottom = lengthp_auto(value, self)?,
            (CssProperty::Width, N1(value)) => layout.size.width = dimension(value, self)?,
            (CssProperty::Height, N1(value)) => layout.size.height = dimension(value, self)?,
            (CssProperty::MinWidth, N1(value)) => layout.min_size.width = dimension(value, self)?,
            (CssProperty::MinHeight, N1(value)) => layout.min_size.height = dimension(value, self)?,
            (CssProperty::MaxWidth, N1(value)) => layout.max_size.width = dimension(value, self)?,
            (CssProperty::MaxHeight, N1(value)) => layout.max_size.height = dimension(value, self)?,
            (CssProperty::AspectRatio, _) => {
                // TODO:
                // layout.aspect_ratio = None;
                return Err(CascadeError::PropertyNotSupported);
            }
            (CssProperty::Margin, N4(top, right, bottom, left)) => {
                layout.margin.top = lengthp_auto(top, self)?;
                layout.margin.right = lengthp_auto(right, self)?;
                layout.margin.bottom = lengthp_auto(bottom, self)?;
                layout.margin.left = lengthp_auto(left, self)?;
            }
            (CssProperty::Margin, N3(top, horizontal, bottom)) => {
                layout.margin.top = lengthp_auto(top, self)?;
                layout.margin.right = lengthp_auto(horizontal, self)?;
                layout.margin.bottom = lengthp_auto(bottom, self)?;
                layout.margin.left = lengthp_auto(horizontal, self)?;
            }
            (CssProperty::Margin, N2(vertical, horizontal)) => {
                layout.margin.top = lengthp_auto(vertical, self)?;
                layout.margin.right = lengthp_auto(horizontal, self)?;
                layout.margin.bottom = lengthp_auto(vertical, self)?;
                layout.margin.left = lengthp_auto(horizontal, self)?;
            }
            (CssProperty::Margin, N1(value)) => {
                layout.margin.top = lengthp_auto(value, self)?;
                layout.margin.right = lengthp_auto(value, self)?;
                layout.margin.bottom = lengthp_auto(value, self)?;
                layout.margin.left = lengthp_auto(value, self)?;
            }
            (CssProperty::MarginTop, N1(value)) => {
                layout.margin.top = lengthp_auto(value, self)?;
            }
            (CssProperty::MarginRight, N1(value)) => {
                layout.margin.right = lengthp_auto(value, self)?;
            }
            (CssProperty::MarginBottom, N1(value)) => {
                layout.margin.bottom = lengthp_auto(value, self)?;
            }
            (CssProperty::MarginLeft, N1(value)) => {
                layout.margin.left = lengthp_auto(value, self)?;
            }

            (CssProperty::Padding, N4(top, right, bottom, left)) => {
                layout.padding.top = lengthp(top, self)?;
                layout.padding.right = lengthp(right, self)?;
                layout.padding.bottom = lengthp(bottom, self)?;
                layout.padding.left = lengthp(left, self)?;
            }
            (CssProperty::Padding, N3(top, horizontal, bottom)) => {
                layout.padding.top = lengthp(top, self)?;
                layout.padding.right = lengthp(horizontal, self)?;
                layout.padding.bottom = lengthp(bottom, self)?;
                layout.padding.left = lengthp(horizontal, self)?;
            }
            (CssProperty::Padding, N2(vertical, horizontal)) => {
                layout.padding.top = lengthp(vertical, self)?;
                layout.padding.right = lengthp(horizontal, self)?;
                layout.padding.bottom = lengthp(vertical, self)?;
                layout.padding.left = lengthp(horizontal, self)?;
            }
            (CssProperty::Padding, N1(value)) => {
                layout.padding.top = lengthp(value, self)?;
                layout.padding.right = lengthp(value, self)?;
                layout.padding.bottom = lengthp(value, self)?;
                layout.padding.left = lengthp(value, self)?;
            }
            (CssProperty::PaddingTop, N1(value)) => {
                layout.padding.top = lengthp(value, self)?;
            }
            (CssProperty::PaddingRight, N1(value)) => {
                layout.padding.right = lengthp(value, self)?;
            }
            (CssProperty::PaddingBottom, N1(value)) => {
                layout.padding.bottom = lengthp(value, self)?;
            }
            (CssProperty::PaddingLeft, N1(value)) => {
                layout.padding.left = lengthp(value, self)?;
            }

            (CssProperty::Border, N4(top, right, bottom, left)) => {
                layout.border.top = lengthp(top, self)?;
                layout.border.right = lengthp(right, self)?;
                layout.border.bottom = lengthp(bottom, self)?;
                layout.border.left = lengthp(left, self)?;
            }
            (CssProperty::Border, N3(top, horizontal, bottom)) => {
                layout.border.top = lengthp(top, self)?;
                layout.border.right = lengthp(horizontal, self)?;
                layout.border.bottom = lengthp(bottom, self)?;
                layout.border.left = lengthp(horizontal, self)?;
            }
            (CssProperty::Border, N2(vertical, horizontal)) => {
                layout.border.top = lengthp(vertical, self)?;
                layout.border.right = lengthp(horizontal, self)?;
                layout.border.bottom = lengthp(vertical, self)?;
                layout.border.left = lengthp(horizontal, self)?;
            }
            (CssProperty::Border, N1(value)) => {
                layout.border.top = lengthp(value, self)?;
                layout.border.right = lengthp(value, self)?;
                layout.border.bottom = lengthp(value, self)?;
                layout.border.left = lengthp(value, self)?;
            }
            (CssProperty::BorderTopWidth, N1(value)) => {
                layout.border.top = lengthp(value, self)?;
            }
            (CssProperty::BorderRightWidth, N1(value)) => {
                layout.border.right = lengthp(value, self)?;
            }
            (CssProperty::BorderLeftWidth, N1(value)) => {
                layout.border.bottom = lengthp(value, self)?;
            }
            (CssProperty::BorderBottomWidth, N1(value)) => {
                layout.border.left = lengthp(value, self)?;
            }
            (CssProperty::AlignContent, N1(Keyword(keyword))) => {
                layout.align_content = map_align_content(keyword.as_str(css))?
            }
            (CssProperty::AlignItems, N1(Keyword(keyword))) => {
                layout.align_items = map_align_items(keyword.as_str(css))?
            }
            (CssProperty::AlignSelf, N1(Keyword(keyword))) => {
                layout.align_self = map_align_items(keyword.as_str(css))?
            }
            (CssProperty::JustifyContent, N1(Keyword(keyword))) => {
                layout.justify_content = map_align_content(keyword.as_str(css))?
            }
            (CssProperty::JustifyItems, N1(Keyword(keyword))) => {
                layout.justify_items = map_align_items(keyword.as_str(css))?
            }
            (CssProperty::JustifySelf, N1(Keyword(keyword))) => {
                layout.justify_self = map_align_items(keyword.as_str(css))?
            }
            (CssProperty::Gap, N2(column, row)) => {
                layout.gap.width = lengthp(column, self)?;
                layout.gap.height = lengthp(row, self)?;
            }
            (CssProperty::Gap, N1(gap)) => {
                layout.gap.width = lengthp(gap, self)?;
                layout.gap.height = lengthp(gap, self)?;
            }
            (CssProperty::ColumnGap, N1(column)) => {
                layout.gap.width = lengthp(column, self)?;
            }
            (CssProperty::RowGap, N1(row)) => {
                layout.gap.height = lengthp(row, self)?;
            }
            (CssProperty::FlexDirection, N1(Keyword(keyword))) => {
                layout.flex_direction = match keyword.as_str(css) {
                    "row" => taffy::FlexDirection::Row,
                    "row-reverse" => taffy::FlexDirection::RowReverse,
                    "column" => taffy::FlexDirection::Column,
                    "column-reverse" => taffy::FlexDirection::ColumnReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (CssProperty::FlexWrap, N1(Keyword(keyword))) => {
                layout.flex_wrap = match keyword.as_str(css) {
                    "row" => taffy::FlexWrap::Wrap,
                    "nowrap" => taffy::FlexWrap::NoWrap,
                    "wrap-reverse" => taffy::FlexWrap::WrapReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (CssProperty::FlexBasis, N1(value)) => layout.flex_basis = dimension(value, self)?,
            (CssProperty::FlexGrow, N1(Number(value))) => layout.flex_grow = *value,
            (CssProperty::FlexShrink, N1(Number(value))) => layout.flex_shrink = *value,
            _ => return Err(CascadeError::PropertyNotSupported),
        }
        Ok(())
    }
}

fn resolve_color(value: &CssValue, cascade: &Cascade) -> Result<[u8; 4], CascadeError> {
    let value = match value {
        CssValue::Color(color) => *color,
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_color(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_timing(value: &CssValue, cascade: &Cascade) -> Result<TimingFunction, CascadeError> {
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
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_timing(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_transforms(
    values: Vec<&CssValue>,
    cascade: &Cascade,
) -> Result<Vec<TransformFunction>, CascadeError> {
    let mut transforms = vec![];
    for value in values.into_iter() {
        match value {
            CssValue::Function(function) => match cascade.css.as_function(function) {
                ("translate", [x]) => {
                    let x = length(x, cascade)?;
                    let y = Length::zero();
                    let z = 0.0;
                    transforms.push(TransformFunction::translate(x, y, z))
                }
                ("translate", [x, y]) => {
                    let x = length(x, cascade)?;
                    let y = length(y, cascade)?;
                    let z = 0.0;
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
    value: &CssValue,
    cascade: &Cascade,
) -> Result<AnimationIterations, CascadeError> {
    let value = match value {
        Keyword(keyword) => match cascade.css.as_str(*keyword) {
            "infinite" => AnimationIterations::Infinite,
            _ => return Err(CascadeError::ValueNotSupported),
        },
        Number(number) => AnimationIterations::Number(*number),
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_iterations(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_string(value: &CssValue, cascade: &Cascade) -> Result<String, CascadeError> {
    let value = match value {
        CssValue::String(value) => value.as_str(&cascade.css.source).to_string(),
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_string(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_length(value: &CssValue, cascade: &Cascade, base: f32) -> Result<f32, CascadeError> {
    let value = match value {
        CssValue::Zero => 0.0,
        CssValue::Dim(dimension) => parse_dimension_length(dimension, cascade)?,
        CssValue::Percentage(percent) => percent * base,
        Number(value) => *value,
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return resolve_length(value, cascade, base);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn dimension_length(value: &CssValue, cascade: &Cascade) -> Result<f32, CascadeError> {
    let value = match value {
        CssValue::Zero => 0.0,
        CssValue::Dim(dimension) => parse_dimension_length(dimension, cascade)?,
        Number(value) => *value,
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return dimension_length(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn parse_dimension_length(
    dimension: &CssDimension,
    cascade: &Cascade,
) -> Result<f32, CascadeError> {
    let value = dimension.value;
    let sizes = cascade.sizes;
    let value = match dimension.unit.as_str(&cascade.css.source) {
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

fn dimension(value: &CssValue, cascade: &Cascade) -> Result<Dimension, CascadeError> {
    let value = match value {
        CssValue::Dim(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            Dimension::Length(length)
        }
        CssValue::Percentage(value) => Dimension::Percent(*value),
        Keyword(keyword) if keyword.as_str(&cascade.css.source) == "auto" => Dimension::Auto,
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return dimension(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn length(value: &CssValue, cascade: &Cascade) -> Result<Length, CascadeError> {
    let value = match value {
        CssValue::Dim(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            Length::Number(length)
        }
        CssValue::Percentage(value) => Length::Percent(*value),
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return length(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn lengthp(value: &CssValue, cascade: &Cascade) -> Result<LengthPercentage, CascadeError> {
    let value = match value {
        CssValue::Dim(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            LengthPercentage::Length(length)
        }
        CssValue::Percentage(value) => LengthPercentage::Percent(*value),
        CssValue::Var(variable) => {
            let value = cascade.get_variable_value(variable)?;
            return lengthp(value, cascade);
        }
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn lengthp_auto(value: &CssValue, cascade: &Cascade) -> Result<LengthPercentageAuto, CascadeError> {
    let value = match value {
        CssValue::Dim(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            LengthPercentageAuto::Length(length)
        }
        CssValue::Percentage(value) => LengthPercentageAuto::Percent(*value),
        Keyword(keyword) if keyword.as_str(&cascade.css.source) == "auto" => {
            LengthPercentageAuto::Auto
        }
        CssValue::Var(variable) => {
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
