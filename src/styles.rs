use std::collections::HashMap;

use log::error;

use taffy::{
    Dimension, Layout, LengthPercentage, LengthPercentageAuto, NodeId, Overflow, Point, Rect,
    TaffyTree,
};

use crate::animation::{
    AnimationDirection, AnimationFillMode, AnimationIterations, AnimationResult, Animator,
    TimingFunction, Transition,
};
use crate::css::{
    match_style, ComputedValue, Css, Declaration, Definition, Dim, PropertyKey, PseudoClassMatcher,
    Shorthand, Style, Units, Var, Variable,
};

use crate::css::ComputedValue::{Keyword, Time};
use crate::{
    Background, Borders, Element, FontFace, Input, Length, ObjectFit, PointerEvents, TextAlign,
    TransformFunction,
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
        initial(element);
        // 0: inheritance
        inherit(parent, element);
        // 1: css rules
        for style in &self.css.styles {
            if match_style(&style, node, tree, matcher) {
                self.apply_style(style, parent, layout, element);
            }
        }
        // 2: transitions
        // let time = input.time.as_secs_f32();
        // let transitions: Vec<AnimationResult> = element
        //     .transitions
        //     .values_mut()
        //     .flat_map(|transition| transition.play(self.css, time))
        //     .collect();
        // for play in transitions {
        //     let result =
        //         self.apply_shorthand___to_delete(play.key, &play.shorthand, layout, element);
        //     if let Err(error) = result {
        //         error!(
        //             "unable to apply transition result of {:?}, {:?}, {:?}",
        //             play.key, play.shorthand, error
        //         );
        //     }
        // }
        // 3: animations
        // if !element.animator.name.is_empty() {
        //     if let Some(animation) = self.css.animations.get(&element.animator.name) {
        //         for play in element.animator.play(self.css, animation, time) {
        //             let result = self.apply_shorthand___to_delete(
        //                 play.key,
        //                 &play.shorthand,
        //                 layout,
        //                 element,
        //             );
        //             if let Err(error) = result {
        //                 error!(
        //                     "unable to apply animation result of {:?}, {:?}, {:?}",
        //                     play.key, play.shorthand, error
        //                 );
        //             }
        //         }
        //     }
        // }
    }

    fn apply_style(
        &mut self,
        style: &'c Style,
        _parent: &Element,
        layout: &mut taffy::Style,
        element: &mut Element,
    ) {
        // for property in &style.declaration {
        //     // if let PropertyKey::Variable(name) = property.key {
        //     //     self.push_variable(name, &property.values);
        //     //     continue;
        //     // }
        //     if PropertyKey::Transition == property.key {
        //         for shorthand in property.values.to_vec() {
        //             let key = match &shorthand[0] {
        //                 Keyword(name) => {
        //                     let key = match PropertyKey::parse(name) {
        //                         Some(key) => key,
        //                         None => {
        //                             error!("unable to make transition of {name}, not supported");
        //                             continue;
        //                         }
        //                     };
        //                     key
        //                 }
        //                 _ => {
        //                     error!("invalid transition property value");
        //                     continue;
        //                 }
        //             };
        //             let transition = element
        //                 .transitions
        //                 .entry(key)
        //                 .or_insert_with(|| Transition::new(key));
        //             match &shorthand[1..] {
        //                 [Time(duration)] => {
        //                     transition.set_duration(*duration);
        //                 }
        //                 [Time(duration), timing] => {
        //                     transition.set_duration(*duration);
        //                     transition.set_timing(resolve_timing(&timing, self).unwrap());
        //                 }
        //                 [Time(duration), timing, Time(delay)] => {
        //                     transition.set_duration(*duration);
        //                     transition.set_timing(resolve_timing(&timing, self).unwrap());
        //                     transition.set_delay(*delay);
        //                 }
        //                 shorthand => {
        //                     error!("transition value not supported {shorthand:?}");
        //                     continue;
        //                 }
        //             }
        //         }
        //         continue;
        //     }
        //     if let Some(transition) = element.transitions.get_mut(&property.key) {
        //         let shorthand = property.get_first_shorthand();
        //         // ID ?
        //         transition.set(property.id, shorthand);
        //         continue;
        //     }
        //     if let Err(error) =
        //         self.apply_shorthand(property.key, &property.values[0], layout, element)
        //     {
        //         error!("unable to apply property {property:?}, {error:?}")
        //     }
        // }
        for declaration in &style.declaration {
            match declaration {
                Declaration::Variable(variable) => self.set_variable(variable),
                Declaration::Property(property) => {
                    // TODO: properties with multiple values
                    let mut shorthand = vec![];
                    self.compute_shorthand(&property.values[0], &mut shorthand);
                    let result = self.apply_shorthand(property.key, &shorthand, layout, element);
                    if let Err(error) = result {
                        error!(
                            "unable to apply property {:?} shorthand, {:?}",
                            property.key, error
                        );
                    }
                }
            }
        }
    }

    fn compute_shorthand(
        &self,
        definition: &[Definition],
        computed_values: &mut Vec<ComputedValue>,
    ) {
        for value in definition {
            match value {
                Definition::Var(name) => {
                    self.compute_variable(name, computed_values);
                }
                Definition::Function(_) => {}
                Definition::Explicit(value) => {
                    // TODO: remove clone ? remove string from value
                    computed_values.push(value.clone())
                }
            }
        }
    }

    fn compute_variable(&self, name: &str, computed_values: &mut Vec<ComputedValue>) {
        let shorthand = match self.get_variable(name) {
            Some(shorthand) => shorthand,
            None => {
                error!("unable to compute variable {name}, not found");
                return;
            }
        };
        self.compute_shorthand(shorthand, computed_values);
    }

    fn set_variable(&self, variable: &Variable) {
        unimplemented!()
    }

    fn get_variable(&self, name: &str) -> Option<&Shorthand> {
        unimplemented!()
    }

    fn apply_shorthand(
        &mut self,
        key: PropertyKey,
        shorthand: &[ComputedValue],
        layout: &mut taffy::Style,
        element: &mut Element,
    ) -> Result<(), CascadeError> {
        match (key, shorthand) {
            //
            // Element
            //
            (PropertyKey::Background, [color]) => {
                self.apply(PropertyKey::BackgroundColor, color, layout, element)?;
            }
            //
            // Element + Layout
            //
            (PropertyKey::Border, [width, _style, color]) => {
                self.apply(PropertyKey::BorderTopWidth, width, layout, element)?;
                self.apply(PropertyKey::BorderTopColor, color, layout, element)?;
                self.apply(PropertyKey::BorderRightWidth, width, layout, element)?;
                self.apply(PropertyKey::BorderRightColor, color, layout, element)?;
                self.apply(PropertyKey::BorderBottomWidth, width, layout, element)?;
                self.apply(PropertyKey::BorderBottomColor, color, layout, element)?;
                self.apply(PropertyKey::BorderLeftWidth, width, layout, element)?;
                self.apply(PropertyKey::BorderLeftColor, color, layout, element)?;
            }
            (PropertyKey::BorderTop, [width, _style, color]) => {
                self.apply(PropertyKey::BorderTopWidth, width, layout, element)?;
                self.apply(PropertyKey::BorderTopColor, color, layout, element)?;
            }
            (PropertyKey::BorderRight, [width, _style, color]) => {
                self.apply(PropertyKey::BorderRightWidth, width, layout, element)?;
                self.apply(PropertyKey::BorderRightColor, color, layout, element)?;
            }
            (PropertyKey::BorderBottom, [width, _style, color]) => {
                self.apply(PropertyKey::BorderBottomWidth, width, layout, element)?;
                self.apply(PropertyKey::BorderBottomColor, color, layout, element)?;
            }
            (PropertyKey::BorderLeft, [width, _style, color]) => {
                self.apply(PropertyKey::BorderLeftWidth, width, layout, element)?;
                self.apply(PropertyKey::BorderLeftColor, color, layout, element)?;
            }
            (PropertyKey::BorderWidth, [top, right, bottom, left]) => {
                self.apply(PropertyKey::BorderTopWidth, top, layout, element)?;
                self.apply(PropertyKey::BorderRightWidth, right, layout, element)?;
                self.apply(PropertyKey::BorderBottomWidth, bottom, layout, element)?;
                self.apply(PropertyKey::BorderLeftWidth, left, layout, element)?;
            }
            (PropertyKey::BorderWidth, [top, h, bottom]) => {
                self.apply(PropertyKey::BorderTopWidth, top, layout, element)?;
                self.apply(PropertyKey::BorderRightWidth, h, layout, element)?;
                self.apply(PropertyKey::BorderBottomWidth, bottom, layout, element)?;
                self.apply(PropertyKey::BorderLeftWidth, h, layout, element)?;
            }
            (PropertyKey::BorderWidth, [v, h]) => {
                self.apply(PropertyKey::BorderTopWidth, v, layout, element)?;
                self.apply(PropertyKey::BorderRightWidth, h, layout, element)?;
                self.apply(PropertyKey::BorderBottomWidth, v, layout, element)?;
                self.apply(PropertyKey::BorderLeftWidth, h, layout, element)?;
            }
            (PropertyKey::BorderWidth, [value]) => {
                self.apply(PropertyKey::BorderTopWidth, value, layout, element)?;
                self.apply(PropertyKey::BorderRightWidth, value, layout, element)?;
                self.apply(PropertyKey::BorderBottomWidth, value, layout, element)?;
                self.apply(PropertyKey::BorderLeftWidth, value, layout, element)?;
            }
            (PropertyKey::BorderColor, [top, right, bottom, left]) => {
                self.apply(PropertyKey::BorderTopColor, top, layout, element)?;
                self.apply(PropertyKey::BorderRightColor, right, layout, element)?;
                self.apply(PropertyKey::BorderBottomColor, bottom, layout, element)?;
                self.apply(PropertyKey::BorderLeftColor, left, layout, element)?;
            }
            (PropertyKey::BorderColor, [top, h, bottom]) => {
                self.apply(PropertyKey::BorderTopColor, top, layout, element)?;
                self.apply(PropertyKey::BorderRightColor, h, layout, element)?;
                self.apply(PropertyKey::BorderBottomColor, bottom, layout, element)?;
                self.apply(PropertyKey::BorderLeftColor, h, layout, element)?;
            }
            (PropertyKey::BorderColor, [v, h]) => {
                self.apply(PropertyKey::BorderTopColor, v, layout, element)?;
                self.apply(PropertyKey::BorderRightColor, h, layout, element)?;
                self.apply(PropertyKey::BorderBottomColor, v, layout, element)?;
                self.apply(PropertyKey::BorderLeftColor, h, layout, element)?;
            }
            (PropertyKey::BorderColor, [value]) => {
                self.apply(PropertyKey::BorderTopColor, value, layout, element)?;
                self.apply(PropertyKey::BorderRightColor, value, layout, element)?;
                self.apply(PropertyKey::BorderBottomColor, value, layout, element)?;
                self.apply(PropertyKey::BorderLeftColor, value, layout, element)?;
            }
            (PropertyKey::BorderRadius, [a, b, c, d]) => {
                self.apply(PropertyKey::BorderTopLeftRadius, a, layout, element)?;
                self.apply(PropertyKey::BorderTopRightRadius, b, layout, element)?;
                self.apply(PropertyKey::BorderBottomRightRadius, c, layout, element)?;
                self.apply(PropertyKey::BorderBottomLeftRadius, d, layout, element)?;
            }
            (PropertyKey::BorderRadius, [a, b, c]) => {
                self.apply(PropertyKey::BorderTopLeftRadius, a, layout, element)?;
                self.apply(PropertyKey::BorderTopRightRadius, b, layout, element)?;
                self.apply(PropertyKey::BorderBottomRightRadius, c, layout, element)?;
                self.apply(PropertyKey::BorderBottomLeftRadius, b, layout, element)?;
            }
            (PropertyKey::BorderRadius, [a, b]) => {
                self.apply(PropertyKey::BorderTopLeftRadius, a, layout, element)?;
                self.apply(PropertyKey::BorderTopRightRadius, b, layout, element)?;
                self.apply(PropertyKey::BorderBottomRightRadius, a, layout, element)?;
                self.apply(PropertyKey::BorderBottomLeftRadius, b, layout, element)?;
            }
            (PropertyKey::BorderRadius, [value]) => {
                self.apply(PropertyKey::BorderTopLeftRadius, value, layout, element)?;
                self.apply(PropertyKey::BorderTopRightRadius, value, layout, element)?;
                self.apply(PropertyKey::BorderBottomRightRadius, value, layout, element)?;
                self.apply(PropertyKey::BorderBottomLeftRadius, value, layout, element)?;
            }
            //
            // Layout
            //
            (PropertyKey::Overflow, [value]) => {
                self.apply(PropertyKey::OverflowX, value, layout, element)?;
                self.apply(PropertyKey::OverflowY, value, layout, element)?;
            }
            (PropertyKey::Overflow, [x, y]) => {
                self.apply(PropertyKey::OverflowX, x, layout, element)?;
                self.apply(PropertyKey::OverflowY, y, layout, element)?;
            }
            (PropertyKey::Inset, [top, right, bottom, left]) => {
                self.apply(PropertyKey::Top, top, layout, element)?;
                self.apply(PropertyKey::Right, right, layout, element)?;
                self.apply(PropertyKey::Bottom, bottom, layout, element)?;
                self.apply(PropertyKey::Left, left, layout, element)?;
            }
            (PropertyKey::Inset, [value]) => {
                self.apply(PropertyKey::Top, value, layout, element)?;
                self.apply(PropertyKey::Right, value, layout, element)?;
                self.apply(PropertyKey::Bottom, value, layout, element)?;
                self.apply(PropertyKey::Left, value, layout, element)?;
            }
            (PropertyKey::Gap, [column, row]) => {
                self.apply(PropertyKey::RowGap, row, layout, element)?;
                self.apply(PropertyKey::ColumnGap, column, layout, element)?;
            }
            (PropertyKey::Gap, [value]) => {
                self.apply(PropertyKey::RowGap, value, layout, element)?;
                self.apply(PropertyKey::ColumnGap, value, layout, element)?;
            }
            (PropertyKey::Padding, [top, right, bottom, left]) => {
                self.apply(PropertyKey::PaddingTop, top, layout, element)?;
                self.apply(PropertyKey::PaddingRight, right, layout, element)?;
                self.apply(PropertyKey::PaddingBottom, bottom, layout, element)?;
                self.apply(PropertyKey::PaddingLeft, left, layout, element)?;
            }
            (PropertyKey::Padding, [top, h, bottom]) => {
                self.apply(PropertyKey::PaddingTop, top, layout, element)?;
                self.apply(PropertyKey::PaddingRight, h, layout, element)?;
                self.apply(PropertyKey::PaddingBottom, bottom, layout, element)?;
                self.apply(PropertyKey::PaddingLeft, h, layout, element)?;
            }
            (PropertyKey::Padding, [v, h]) => {
                self.apply(PropertyKey::PaddingTop, v, layout, element)?;
                self.apply(PropertyKey::PaddingRight, h, layout, element)?;
                self.apply(PropertyKey::PaddingBottom, v, layout, element)?;
                self.apply(PropertyKey::PaddingLeft, h, layout, element)?;
            }
            (PropertyKey::Padding, [value]) => {
                self.apply(PropertyKey::PaddingTop, value, layout, element)?;
                self.apply(PropertyKey::PaddingRight, value, layout, element)?;
                self.apply(PropertyKey::PaddingBottom, value, layout, element)?;
                self.apply(PropertyKey::PaddingLeft, value, layout, element)?;
            }
            (PropertyKey::Margin, [top, right, bottom, left]) => {
                self.apply(PropertyKey::MarginTop, top, layout, element)?;
                self.apply(PropertyKey::MarginRight, right, layout, element)?;
                self.apply(PropertyKey::MarginBottom, bottom, layout, element)?;
                self.apply(PropertyKey::MarginLeft, left, layout, element)?;
            }
            (PropertyKey::Margin, [top, h, bottom]) => {
                self.apply(PropertyKey::MarginTop, top, layout, element)?;
                self.apply(PropertyKey::MarginRight, h, layout, element)?;
                self.apply(PropertyKey::MarginBottom, bottom, layout, element)?;
                self.apply(PropertyKey::MarginLeft, h, layout, element)?;
            }
            (PropertyKey::Margin, [v, h]) => {
                self.apply(PropertyKey::MarginTop, v, layout, element)?;
                self.apply(PropertyKey::MarginRight, h, layout, element)?;
                self.apply(PropertyKey::MarginBottom, v, layout, element)?;
                self.apply(PropertyKey::MarginLeft, h, layout, element)?;
            }
            (PropertyKey::Margin, [value]) => {
                self.apply(PropertyKey::MarginTop, value, layout, element)?;
                self.apply(PropertyKey::MarginRight, value, layout, element)?;
                self.apply(PropertyKey::MarginBottom, value, layout, element)?;
                self.apply(PropertyKey::MarginLeft, value, layout, element)?;
            }
            //
            // Transform
            //
            (PropertyKey::Transform, shorthand) => {
                element.transforms = resolve_transforms(shorthand, self)?;
            }
            //
            // Animation
            //
            // there is no static shorthand pattern, we should set values by it type and order
            // TODO: special animation shorthand parser
            (PropertyKey::Animation, [Time(duration), timing, Time(delay), Keyword(name)]) => {
                element.animator.name = name.to_string();
                element.animator.delay = *delay;
                element.animator.duration = *duration;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (PropertyKey::Animation, [Time(duration), timing, iterations, Keyword(name)]) => {
                element.animator.name = name.to_string();
                element.animator.duration = *duration;
                element.animator.iterations = resolve_iterations(iterations, self)?;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (PropertyKey::Animation, [Time(duration), timing, Keyword(name)]) => {
                element.animator.name = name.to_string();
                element.animator.duration = *duration;
                element.animator.timing = resolve_timing(timing, self)?;
            }
            (PropertyKey::Animation, [Time(duration), Keyword(name)]) => {
                element.animator.name = name.to_string();
                element.animator.duration = *duration;
            }
            (key, [value]) => return self.apply(key, value, layout, element),
            _ => return Err(CascadeError::PropertyNotSupported),
        }
        Ok(())
    }

    fn apply(
        &mut self,
        key: PropertyKey,
        value: &ComputedValue,
        layout: &mut taffy::Style,
        element: &mut Element,
    ) -> Result<(), CascadeError> {
        match (key, value) {
            //
            // Element
            //
            (PropertyKey::BackgroundColor, value) => {
                element.background.color = resolve_color(value, self)?
            }
            (PropertyKey::Color, value) => element.color = resolve_color(value, self)?,
            (PropertyKey::FontSize, value) => {
                element.font.size = resolve_length(value, self, self.sizes.parent_font_size)?;
            }
            (PropertyKey::FontWeight, value) => {
                element.font.weight = resolve_font_weight(value, self)?
            }
            (PropertyKey::FontFamily, value) => element.font.family = resolve_string(value, self)?,
            (PropertyKey::FontStyle, ComputedValue::Keyword(keyword)) => {
                element.font.style = match keyword.as_str() {
                    "normal" => "normal".to_string(),
                    "italic" => "italic".to_string(),
                    "oblique" => "oblique".to_string(),
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::TextAlign, ComputedValue::Keyword(keyword)) => {
                element.font.align = match keyword.as_str() {
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
            (PropertyKey::PointerEvents, ComputedValue::Keyword(keyword)) => {
                element.pointer_events = match keyword.as_str() {
                    "auto" => PointerEvents::Auto,
                    "none" => PointerEvents::None,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            //
            // Element + Layout
            //
            (PropertyKey::BorderTopWidth, value) => {
                element.borders.top.width = dimension_length(value, self)?;
                layout.border.top = LengthPercentage::Length(element.borders.top.width);
            }
            (PropertyKey::BorderRightWidth, value) => {
                element.borders.right.width = dimension_length(value, self)?;
                layout.border.right = LengthPercentage::Length(element.borders.right.width);
            }
            (PropertyKey::BorderBottomWidth, value) => {
                element.borders.bottom.width = dimension_length(value, self)?;
                layout.border.bottom = LengthPercentage::Length(element.borders.bottom.width);
            }
            (PropertyKey::BorderLeftWidth, value) => {
                element.borders.left.width = dimension_length(value, self)?;
                layout.border.left = LengthPercentage::Length(element.borders.left.width);
            }
            (PropertyKey::BorderTopColor, value) => {
                element.borders.top.color = resolve_color(value, self)?;
            }
            (PropertyKey::BorderRightColor, value) => {
                element.borders.right.color = resolve_color(value, self)?;
            }
            (PropertyKey::BorderBottomColor, value) => {
                element.borders.bottom.color = resolve_color(value, self)?;
            }
            (PropertyKey::BorderLeftColor, value) => {
                element.borders.left.color = resolve_color(value, self)?;
            }
            (PropertyKey::BorderTopLeftRadius, value) => {
                element.borders.radius[0] = length(value, self)?;
            }
            (PropertyKey::BorderTopRightRadius, value) => {
                element.borders.radius[1] = length(value, self)?;
            }
            (PropertyKey::BorderBottomRightRadius, value) => {
                element.borders.radius[2] = length(value, self)?;
            }
            (PropertyKey::BorderBottomLeftRadius, value) => {
                element.borders.radius[3] = length(value, self)?;
            }
            //
            // Layout
            //
            (PropertyKey::MarginTop, value) => layout.margin.top = lengthp_auto(value, self)?,
            (PropertyKey::MarginRight, value) => layout.margin.right = lengthp_auto(value, self)?,
            (PropertyKey::MarginBottom, value) => layout.margin.bottom = lengthp_auto(value, self)?,
            (PropertyKey::MarginLeft, value) => layout.margin.left = lengthp_auto(value, self)?,
            (PropertyKey::PaddingTop, value) => layout.padding.top = lengthp(value, self)?,
            (PropertyKey::PaddingRight, value) => layout.padding.right = lengthp(value, self)?,
            (PropertyKey::PaddingBottom, value) => layout.padding.bottom = lengthp(value, self)?,
            (PropertyKey::PaddingLeft, value) => layout.padding.left = lengthp(value, self)?,
            (PropertyKey::Display, Keyword(keyword)) => match keyword.as_str() {
                "flow" => layout.display = taffy::Display::Block,
                "block" => layout.display = taffy::Display::Block,
                "flex" => layout.display = taffy::Display::Flex,
                "grid" => layout.display = taffy::Display::Grid,
                keyword => return CascadeError::invalid_keyword(keyword),
            },
            (PropertyKey::OverflowX, Keyword(x)) => {
                layout.overflow.x = resolve_overflow(x.as_str())?
            }
            (PropertyKey::OverflowY, Keyword(y)) => {
                layout.overflow.y = resolve_overflow(y.as_str())?
            }
            (PropertyKey::Position, Keyword(keyword)) => match keyword.as_str() {
                "relative" => layout.position = taffy::Position::Relative,
                "absolute" => layout.position = taffy::Position::Absolute,
                keyword => return CascadeError::invalid_keyword(keyword),
            },
            (PropertyKey::Left, value) => layout.inset.left = lengthp_auto(value, self)?,
            (PropertyKey::Right, value) => layout.inset.right = lengthp_auto(value, self)?,
            (PropertyKey::Top, value) => layout.inset.top = lengthp_auto(value, self)?,
            (PropertyKey::Bottom, value) => layout.inset.bottom = lengthp_auto(value, self)?,
            (PropertyKey::Width, value) => layout.size.width = dimension(value, self)?,
            (PropertyKey::Height, value) => layout.size.height = dimension(value, self)?,
            (PropertyKey::MinWidth, value) => layout.min_size.width = dimension(value, self)?,
            (PropertyKey::MinHeight, value) => layout.min_size.height = dimension(value, self)?,
            (PropertyKey::MaxWidth, value) => layout.max_size.width = dimension(value, self)?,
            (PropertyKey::MaxHeight, value) => layout.max_size.height = dimension(value, self)?,
            (PropertyKey::AlignContent, Keyword(keyword)) => {
                layout.align_content = map_align_content(keyword.as_str())?
            }
            (PropertyKey::AlignItems, Keyword(keyword)) => {
                layout.align_items = map_align_items(keyword.as_str())?
            }
            (PropertyKey::AlignSelf, Keyword(keyword)) => {
                layout.align_self = map_align_items(keyword.as_str())?
            }
            (PropertyKey::JustifyContent, Keyword(keyword)) => {
                layout.justify_content = map_align_content(keyword.as_str())?
            }
            (PropertyKey::JustifyItems, Keyword(keyword)) => {
                layout.justify_items = map_align_items(keyword.as_str())?
            }
            (PropertyKey::JustifySelf, Keyword(keyword)) => {
                layout.justify_self = map_align_items(keyword.as_str())?
            }
            (PropertyKey::FlexDirection, Keyword(keyword)) => {
                layout.flex_direction = match keyword.as_str() {
                    "row" => taffy::FlexDirection::Row,
                    "row-reverse" => taffy::FlexDirection::RowReverse,
                    "column" => taffy::FlexDirection::Column,
                    "column-reverse" => taffy::FlexDirection::ColumnReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::FlexWrap, Keyword(keyword)) => {
                layout.flex_wrap = match keyword.as_str() {
                    "wrap" => taffy::FlexWrap::Wrap,
                    "nowrap" => taffy::FlexWrap::NoWrap,
                    "wrap-reverse" => taffy::FlexWrap::WrapReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::FlexBasis, value) => layout.flex_basis = dimension(value, self)?,
            (PropertyKey::FlexGrow, ComputedValue::Number(value)) => layout.flex_grow = *value,
            (PropertyKey::FlexShrink, ComputedValue::Number(value)) => layout.flex_shrink = *value,
            (PropertyKey::ColumnGap, column) => {
                layout.gap.width = lengthp(column, self)?;
            }
            (PropertyKey::RowGap, row) => {
                layout.gap.height = lengthp(row, self)?;
            }
            //
            // Animation
            //
            (PropertyKey::AnimationName, Keyword(name)) => {
                element.animator.name = name.to_string();
            }
            (PropertyKey::AnimationDelay, Time(delay)) => {
                element.animator.delay = *delay;
            }
            (PropertyKey::AnimationDirection, Keyword(keyword)) => {
                element.animator.direction = match keyword.as_str() {
                    "normal" => AnimationDirection::Normal,
                    "reverse" => AnimationDirection::Reverse,
                    "alternate" => AnimationDirection::Alternate,
                    "alternate-reverse" => AnimationDirection::AlternateReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationDuration, Time(duration)) => {
                element.animator.duration = *duration;
            }
            (PropertyKey::AnimationFillMode, Keyword(keyword)) => {
                element.animator.fill_mode = match keyword.as_str() {
                    "none" => AnimationFillMode::None,
                    "forwards" => AnimationFillMode::Forwards,
                    "backwards" => AnimationFillMode::Backwards,
                    "both" => AnimationFillMode::Both,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationIterationCount, iterations) => {
                element.animator.iterations = resolve_iterations(iterations, self)?;
            }
            (PropertyKey::AnimationPlayState, Keyword(keyword)) => {
                element.animator.running = match keyword.as_str() {
                    "running" => true,
                    "paused" => false,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationTimingFunction, timing) => {
                element.animator.timing = resolve_timing(timing, self)?
            }
            _ => return Err(CascadeError::PropertyNotSupported),
        }
        Ok(())
    }
}

fn resolve_font_weight(value: &ComputedValue, cascade: &Cascade) -> Result<u16, CascadeError> {
    let value = match value {
        ComputedValue::Number(value) if *value >= 1.0 && *value <= 1000.0 => *value as u16,
        ComputedValue::Keyword(keyword) => match keyword.as_str() {
            "normal" => 400,
            "bold" => 700,
            keyword => return Err(CascadeError::InvalidKeyword(keyword.to_string())),
        },
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_color(value: &ComputedValue, _cascade: &Cascade) -> Result<[u8; 4], CascadeError> {
    let value = match value {
        ComputedValue::Color(color) => *color,
        ComputedValue::Keyword(keyword) => match keyword.as_str() {
            "black" => [0, 0, 0, 255],
            "white" => [255, 255, 255, 255],
            "red" => [255, 0, 0, 255],
            "blue" => [0, 0, 255, 255],
            "green" => [0, 255, 0, 255],
            "transparent" => [0, 0, 0, 0],
            keyword => return Err(CascadeError::InvalidKeyword(keyword.to_string())),
        },
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_timing(
    value: &ComputedValue,
    cascade: &Cascade,
) -> Result<TimingFunction, CascadeError> {
    let value = match value {
        ComputedValue::Keyword(keyword) => match keyword.as_str() {
            "ease" => TimingFunction::Ease,
            "ease-in" => TimingFunction::EaseIn,
            "ease-out" => TimingFunction::EaseOut,
            "ease-in-out" => TimingFunction::EaseInOut,
            "linear" => TimingFunction::Linear,
            "step-start" => TimingFunction::StepStart,
            "step-end" => TimingFunction::StepEnd,
            _ => return Err(CascadeError::ValueNotSupported),
        },
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_transforms(
    values: &[ComputedValue],
    cascade: &Cascade,
) -> Result<Vec<TransformFunction>, CascadeError> {
    unimplemented!()
    // let mut transforms = vec![];
    // for value in values.iter() {
    //     match value {
    //         ComputedValue::Function(function) => match function.describe() {
    //             ("translate", [x]) => {
    //                 let x = length(x, cascade)?;
    //                 let y = Length::zero();
    //                 let z = 0.0;
    //                 transforms.push(TransformFunction::translate(x, y, z))
    //             }
    //             ("translate", [x, y]) => {
    //                 let x = length(x, cascade)?;
    //                 let y = length(y, cascade)?;
    //                 let _z = 0.0;
    //                 transforms.push(TransformFunction::translate(x, y, 0.0))
    //             }
    //             ("translate3d", [x, y, z]) => {
    //                 let x = length(x, cascade)?;
    //                 let y = length(y, cascade)?;
    //                 let z = dimension_length(z, cascade)?;
    //                 transforms.push(TransformFunction::translate(x, y, z))
    //             }
    //             ("translateX", [x]) => {
    //                 let x = length(x, cascade)?;
    //                 let y = Length::zero();
    //                 let z = 0.0;
    //                 transforms.push(TransformFunction::translate(x, y, z))
    //             }
    //             ("translateY", [y]) => {
    //                 let x = Length::zero();
    //                 let y = length(y, cascade)?;
    //                 let z = 0.0;
    //                 transforms.push(TransformFunction::translate(x, y, z))
    //             }
    //             ("translateZ", [z]) => {
    //                 let x = Length::zero();
    //                 let y = Length::zero();
    //                 let z = dimension_length(z, cascade)?;
    //                 transforms.push(TransformFunction::translate(x, y, z))
    //             }
    //             _ => return Err(CascadeError::TransformFunctionNotSupported),
    //         },
    //         _ => return Err(CascadeError::ValueNotSupported),
    //     }
    // }
    // Ok(transforms)
}

fn resolve_iterations(
    value: &ComputedValue,
    cascade: &Cascade,
) -> Result<AnimationIterations, CascadeError> {
    let value = match value {
        ComputedValue::Keyword(keyword) => match keyword.as_str() {
            "infinite" => AnimationIterations::Infinite,
            _ => return Err(CascadeError::ValueNotSupported),
        },
        ComputedValue::Number(number) => AnimationIterations::Number(*number),
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_string(value: &ComputedValue, cascade: &Cascade) -> Result<String, CascadeError> {
    let value = match value {
        ComputedValue::String(value) => value.clone(),
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn resolve_length(
    value: &ComputedValue,
    cascade: &Cascade,
    base: f32,
) -> Result<f32, CascadeError> {
    let value = match value {
        ComputedValue::Zero => 0.0,
        ComputedValue::Dimension(dimension) => parse_dimension_length(dimension, cascade)?,
        ComputedValue::Percentage(percent) => percent * base,
        ComputedValue::Number(value) => *value,
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn dimension_length(value: &ComputedValue, cascade: &Cascade) -> Result<f32, CascadeError> {
    let value = match value {
        ComputedValue::Zero => 0.0,
        ComputedValue::Dimension(dimension) => parse_dimension_length(dimension, cascade)?,
        ComputedValue::Number(value) => *value,
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn parse_dimension_length(dimension: &Dim, cascade: &Cascade) -> Result<f32, CascadeError> {
    let value = dimension.value;
    let sizes = cascade.sizes;
    let value = match dimension.unit {
        Units::Px => value,
        Units::Em => sizes.parent_font_size * value,
        Units::Rem => sizes.root_font_size * value,
        Units::Vw => sizes.viewport_width * value / 100.0,
        Units::Vh => sizes.viewport_height * value / 100.0,
        Units::Vmax => sizes.viewport_width.max(sizes.viewport_height) * value / 100.0,
        Units::Vmin => sizes.viewport_width.min(sizes.viewport_height) * value / 100.0,
    };
    Ok(value)
}

fn dimension(value: &ComputedValue, cascade: &Cascade) -> Result<Dimension, CascadeError> {
    let value = match value {
        ComputedValue::Dimension(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            Dimension::Length(length)
        }
        ComputedValue::Percentage(value) => Dimension::Percent(*value),
        ComputedValue::Keyword(keyword) if keyword.as_str() == "auto" => Dimension::Auto,
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn length(value: &ComputedValue, cascade: &Cascade) -> Result<Length, CascadeError> {
    let value = match value {
        ComputedValue::Dimension(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            Length::Number(length)
        }
        ComputedValue::Percentage(value) => Length::Percent(*value),
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn lengthp(value: &ComputedValue, cascade: &Cascade) -> Result<LengthPercentage, CascadeError> {
    let value = match value {
        ComputedValue::Dimension(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            LengthPercentage::Length(length)
        }
        ComputedValue::Percentage(value) => LengthPercentage::Percent(*value),
        _ => return Err(CascadeError::ValueNotSupported),
    };
    Ok(value)
}

fn lengthp_auto(
    value: &ComputedValue,
    cascade: &Cascade,
) -> Result<LengthPercentageAuto, CascadeError> {
    let value = match value {
        ComputedValue::Zero => LengthPercentageAuto::Length(0.0),
        ComputedValue::Dimension(dimension) => {
            let length = parse_dimension_length(dimension, cascade)?;
            LengthPercentageAuto::Length(length)
        }
        ComputedValue::Percentage(value) => LengthPercentageAuto::Percent(*value),
        ComputedValue::Keyword(keyword) if keyword.as_str() == "auto" => LengthPercentageAuto::Auto,
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
