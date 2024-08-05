use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::animation::{Animation, Keyframe, Track};
use crate::css::CssShorthand::{N1, N2, N3, N4};
use crate::css::CssValue::{Color, Keyword, Number};
use crate::css::{
    CssDimension, CssProperty, CssShorthand, CssValue, CssValues, MyProperty, MyStyle,
};
use crate::html::Dom;
use crate::models::{ElementId, Object, SizeContext};
use crate::{Background, Borders, Element, MyBorder, ObjectFit, Rgba, TextStyle};
use log::{debug, error, warn};
use taffy::prelude::FromLength;
use taffy::prelude::TaffyAuto;
use taffy::prelude::{FromFlex, FromPercent, TaffyFitContent, TaffyMaxContent, TaffyMinContent};
use taffy::style_helpers::TaffyZero;
use taffy::{
    Dimension, GridPlacement, GridTrackRepetition, LengthPercentageAuto, Line, Overflow, Point,
    Rect, Style, TrackSizingFunction,
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
            top: None,
            bottom: None,
            right: None,
            left: None,
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
        transform: None,
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

pub fn inherit(parent: &Element, view: &mut Element) {
    // border-collapse
    // border-spacing
    // caption-side
    // color
    view.color = parent.color;
    // cursor
    // direction
    // empty-cells
    // font-family
    view.text_style.font_family = parent.text_style.font_family.clone();
    // font-size
    view.text_style.font_size = parent.text_style.font_size;
    // font-style
    //view.text_style.font_style = parent.text_style.font_style.clone();
    // font-variant
    // font-weight
    view.text_style.font_weight = parent.text_style.font_weight;
    // font-size-adjust
    // font-stretch
    //view.text_style.font_stretch = parent.text_style.font_stretch.clone();
    // font
    // letter-spacing
    // line-height
    view.text_style.line_height = parent.text_style.line_height;
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

pub fn apply_element_property(
    css: &str,
    property: &MyProperty,
    parent: &Element,
    element: &mut Element,
    context: SizeContext,
    resources: &str,
) {
    let matching = (property.name, property.values.as_shorthand());
    match matching {
        (CssProperty::Background, N1(Color(color))) => {
            element.background.color = *color;
        }
        (CssProperty::BackgroundColor, N1(Color(color))) => {
            element.background.color = *color;
        }
        (CssProperty::Color, N1(Color(color))) => {
            element.color = *color;
        }
        name => {}
    }
}

pub fn apply_element_rules2(
    css: &str,
    style: &MyStyle,
    parent: &Element,
    element: &mut Element,
    context: SizeContext,
    resources: &str,
) {
    inherit(parent, element);
    for property in &style.declaration {
        apply_element_property(css, property, parent, element, context, resources)
    }
}

#[inline(always)]
pub fn apply_layout_rules2(css: &str, style: &MyStyle, layout: &mut Style, context: SizeContext) {
    for property in &style.declaration {
        apply_layout_property(css, property, layout, context)
    }
}

pub fn apply_layout_property(
    css: &str,
    property: &MyProperty,
    layout: &mut Style,
    ctx: SizeContext,
) {
    let matching = (property.name, property.values.as_shorthand());
    match matching {
        (CssProperty::Display, N1(Keyword(keyword))) => match keyword.as_str(css) {
            "flow" => layout.display = taffy::Display::Block,
            "block" => layout.display = taffy::Display::Block,
            "flex" => layout.display = taffy::Display::Flex,
            "grid" => layout.display = taffy::Display::Grid,
            keyword => {
                error!("display {keyword} not supported")
            }
        },
        (CssProperty::Overflow, N2(Keyword(x), Keyword(y))) => {
            layout.overflow.x = map_overflow2(x.as_str(css));
            layout.overflow.y = map_overflow2(y.as_str(css));
        }
        (CssProperty::OverflowX, N1(Keyword(x))) => {
            layout.overflow.x = map_overflow2(x.as_str(css))
        }
        (CssProperty::OverflowY, N1(Keyword(y))) => {
            layout.overflow.y = map_overflow2(y.as_str(css))
        }
        (CssProperty::Position, N1(Keyword(keyword))) => match keyword.as_str(css) {
            "relative" => layout.position = taffy::Position::Relative,
            "absolute" => layout.position = taffy::Position::Absolute,
            keyword => {
                error!("position {keyword} not supported")
            }
        },
        (CssProperty::Inset, N4(top, right, bottom, left)) => {
            layout.inset.top = resolve_lpa(css, top, ctx);
            layout.inset.right = resolve_lpa(css, right, ctx);
            layout.inset.bottom = resolve_lpa(css, bottom, ctx);
            layout.inset.left = resolve_lpa(css, left, ctx);
        }
        (CssProperty::Left, N1(value)) => layout.inset.left = resolve_lpa(css, value, ctx),
        (CssProperty::Right, N1(value)) => layout.inset.right = resolve_lpa(css, value, ctx),
        (CssProperty::Top, N1(value)) => layout.inset.top = resolve_lpa(css, value, ctx),
        (CssProperty::Bottom, N1(value)) => layout.inset.bottom = resolve_lpa(css, value, ctx),
        (CssProperty::Width, N1(value)) => layout.size.width = dimension(css, value, ctx),
        (CssProperty::Height, N1(value)) => layout.size.height = dimension(css, value, ctx),
        (CssProperty::MinWidth, N1(value)) => layout.min_size.width = dimension(css, value, ctx),
        (CssProperty::MinHeight, N1(value)) => layout.min_size.height = dimension(css, value, ctx),
        (CssProperty::MaxWidth, N1(value)) => layout.max_size.width = dimension(css, value, ctx),
        (CssProperty::MaxHeight, N1(value)) => layout.max_size.height = dimension(css, value, ctx),
        (CssProperty::AspectRatio, _) => {
            // TODO:
            // layout.aspect_ratio = None;
            error!("aspect-ratio not supported")
        }
        (CssProperty::Margin, N4(top, right, bottom, left)) => {
            layout.margin.top = resolve_lpa(css, top, ctx);
            layout.margin.right = resolve_lpa(css, right, ctx);
            layout.margin.bottom = resolve_lpa(css, bottom, ctx);
            layout.margin.left = resolve_lpa(css, left, ctx);
        }
        (CssProperty::Margin, N3(top, horizontal, bottom)) => {
            layout.margin.top = top.resolve(css, ctx);
            layout.margin.right = horizontal.resolve(css, ctx);
            layout.margin.bottom = bottom.resolve(css, ctx);
            layout.margin.left = horizontal.resolve(css, ctx);
        }
        (CssProperty::Margin, N2(vertical, horizontal)) => {
            layout.margin.top = resolve_lpa(css, vertical, ctx);
            layout.margin.right = resolve_lpa(css, horizontal, ctx);
            layout.margin.bottom = resolve_lpa(css, vertical, ctx);
            layout.margin.left = resolve_lpa(css, horizontal, ctx);
        }
        (CssProperty::Margin, N1(value)) => {
            layout.margin.top = resolve_lpa(css, value, ctx);
            layout.margin.right = resolve_lpa(css, value, ctx);
            layout.margin.bottom = resolve_lpa(css, value, ctx);
            layout.margin.left = resolve_lpa(css, value, ctx);
        }
        (CssProperty::MarginTop, N1(value)) => {
            layout.margin.top = resolve_lpa(css, value, ctx);
        }
        (CssProperty::MarginRight, N1(value)) => {
            layout.margin.right = resolve_lpa(css, value, ctx);
        }
        (CssProperty::MarginBottom, N1(value)) => {
            layout.margin.bottom = resolve_lpa(css, value, ctx);
        }
        (CssProperty::MarginLeft, N1(value)) => {
            layout.margin.left = resolve_lpa(css, value, ctx);
        }

        (CssProperty::Padding, N4(top, right, bottom, left)) => {
            layout.padding.top = resolve_lp(css, top, ctx);
            layout.padding.right = resolve_lp(css, right, ctx);
            layout.padding.bottom = resolve_lp(css, bottom, ctx);
            layout.padding.left = resolve_lp(css, left, ctx);
        }
        (CssProperty::Padding, N3(top, horizontal, bottom)) => {
            layout.padding.top = resolve_lp(css, top, ctx);
            layout.padding.right = resolve_lp(css, horizontal, ctx);
            layout.padding.bottom = resolve_lp(css, bottom, ctx);
            layout.padding.left = resolve_lp(css, horizontal, ctx);
        }
        (CssProperty::Padding, N2(vertical, horizontal)) => {
            layout.padding.top = resolve_lp(css, vertical, ctx);
            layout.padding.right = resolve_lp(css, horizontal, ctx);
            layout.padding.bottom = resolve_lp(css, vertical, ctx);
            layout.padding.left = resolve_lp(css, horizontal, ctx);
        }
        (CssProperty::Padding, N1(value)) => {
            layout.padding.top = resolve_lp(css, value, ctx);
            layout.padding.right = resolve_lp(css, value, ctx);
            layout.padding.bottom = resolve_lp(css, value, ctx);
            layout.padding.left = resolve_lp(css, value, ctx);
        }
        (CssProperty::PaddingTop, N1(value)) => {
            layout.padding.top = resolve_lp(css, value, ctx);
        }
        (CssProperty::PaddingRight, N1(value)) => {
            layout.padding.right = resolve_lp(css, value, ctx);
        }
        (CssProperty::PaddingBottom, N1(value)) => {
            layout.padding.bottom = resolve_lp(css, value, ctx);
        }
        (CssProperty::PaddingLeft, N1(value)) => {
            layout.padding.left = resolve_lp(css, value, ctx);
        }

        (CssProperty::Border, N4(top, right, bottom, left)) => {
            layout.border.top = resolve_lp(css, top, ctx);
            layout.border.right = resolve_lp(css, right, ctx);
            layout.border.bottom = resolve_lp(css, bottom, ctx);
            layout.border.left = resolve_lp(css, left, ctx);
        }
        (CssProperty::Border, N3(top, horizontal, bottom)) => {
            layout.border.top = resolve_lp(css, top, ctx);
            layout.border.right = resolve_lp(css, horizontal, ctx);
            layout.border.bottom = resolve_lp(css, bottom, ctx);
            layout.border.left = resolve_lp(css, horizontal, ctx);
        }
        (CssProperty::Border, N2(vertical, horizontal)) => {
            layout.border.top = resolve_lp(css, vertical, ctx);
            layout.border.right = resolve_lp(css, horizontal, ctx);
            layout.border.bottom = resolve_lp(css, vertical, ctx);
            layout.border.left = resolve_lp(css, horizontal, ctx);
        }
        (CssProperty::Border, N1(value)) => {
            layout.border.top = resolve_lp(css, value, ctx);
            layout.border.right = resolve_lp(css, value, ctx);
            layout.border.bottom = resolve_lp(css, value, ctx);
            layout.border.left = resolve_lp(css, value, ctx);
        }
        (CssProperty::BorderTopWidth, N1(value)) => {
            layout.border.top = resolve_lp(css, value, ctx);
        }
        (CssProperty::BorderRightWidth, N1(value)) => {
            layout.border.right = resolve_lp(css, value, ctx);
        }
        (CssProperty::BorderLeftWidth, N1(value)) => {
            layout.border.bottom = resolve_lp(css, value, ctx);
        }
        (CssProperty::BorderBottomWidth, N1(value)) => {
            layout.border.left = resolve_lp(css, value, ctx);
        }
        (CssProperty::AlignContent, N1(Keyword(keyword))) => {
            layout.align_content = map_align_content(keyword.as_str(css))
        }
        (CssProperty::AlignItems, N1(Keyword(keyword))) => {
            layout.align_items = map_align_items(keyword.as_str(css))
        }
        (CssProperty::AlignSelf, N1(Keyword(keyword))) => {
            layout.align_self = map_align_items(keyword.as_str(css))
        }
        (CssProperty::JustifyContent, N1(Keyword(keyword))) => {
            layout.justify_content = map_align_content(keyword.as_str(css))
        }
        (CssProperty::JustifyItems, N1(Keyword(keyword))) => {
            layout.justify_items = map_align_items(keyword.as_str(css))
        }
        (CssProperty::JustifySelf, N1(Keyword(keyword))) => {
            layout.justify_self = map_align_items(keyword.as_str(css))
        }
        (CssProperty::Gap, N2(column, row)) => {
            layout.gap.width = column.resolve(css, ctx);
            layout.gap.height = row.resolve(css, ctx);
        }
        (CssProperty::ColumnGap, N1(column)) => {
            layout.gap.width = column.resolve(css, ctx);
        }
        (CssProperty::RowGap, N1(row)) => {
            layout.gap.height = row.resolve(css, ctx);
        }
        (CssProperty::FlexDirection, N1(Keyword(keyword))) => {
            layout.flex_direction = match keyword.as_str(css) {
                "row" => taffy::FlexDirection::Row,
                "row-reverse" => taffy::FlexDirection::RowReverse,
                "column" => taffy::FlexDirection::Column,
                "column-reverse" => taffy::FlexDirection::ColumnReverse,
                keyword => {
                    error!("flex-direction: {keyword}; not supported");
                    return;
                }
            }
        }
        (CssProperty::FlexWrap, N1(Keyword(keyword))) => {
            layout.flex_wrap = match keyword.as_str(css) {
                "row" => taffy::FlexWrap::Wrap,
                "nowrap" => taffy::FlexWrap::NoWrap,
                "wrap-reverse" => taffy::FlexWrap::WrapReverse,
                keyword => {
                    error!("flex-wrap: {keyword}; not supported");
                    return;
                }
            }
        }
        (CssProperty::FlexBasis, N1(value)) => layout.flex_basis = value.resolve(css, ctx),
        (CssProperty::FlexGrow, N1(Number(value))) => layout.flex_grow = *value,
        (CssProperty::FlexShrink, N1(Number(value))) => layout.flex_shrink = *value,
        // TODO: grid
        _ => {
            debug!("property2 {property:?} not supported")
        }
    }
}

trait MyResolver<T> {
    fn resolve(&self, css: &str, context: SizeContext) -> T;
}

impl MyResolver<Dimension> for &CssValue {
    fn resolve(&self, css: &str, context: SizeContext) -> Dimension {
        dimension(css, self, context)
    }
}

impl MyResolver<LengthPercentageAuto> for &CssValue {
    fn resolve(&self, css: &str, context: SizeContext) -> LengthPercentageAuto {
        resolve_lpa(css, self, context)
    }
}

impl MyResolver<taffy::LengthPercentage> for &CssValue {
    fn resolve(&self, css: &str, context: SizeContext) -> taffy::LengthPercentage {
        resolve_lp(css, self, context)
    }
}

fn dimension(css: &str, value: &CssValue, context: SizeContext) -> Dimension {
    type Result = Dimension;
    match value {
        CssValue::Dimension(dimension) => Result::Length(resolve_length2(css, *dimension, context)),
        CssValue::Percentage(value) => Result::Percent(*value),
        Keyword(keyword) => match keyword.as_str(css) {
            "auto" => Result::Auto,
            keyword => {
                error!("keyword {keyword} not supported");
                Result::Auto
            }
        },
        _ => {
            error!("value {value:?} not supported");
            Result::Auto
        }
    }
}

fn resolve_lp(css: &str, value: &CssValue, context: SizeContext) -> taffy::LengthPercentage {
    type Result = taffy::LengthPercentage;
    match value {
        CssValue::Dimension(dimension) => Result::Length(resolve_length2(css, *dimension, context)),
        CssValue::Percentage(value) => Result::Percent(*value),
        _ => {
            error!("value {value:?} not supported");
            Result::ZERO
        }
    }
}

fn resolve_lpa(css: &str, value: &CssValue, context: SizeContext) -> LengthPercentageAuto {
    type Result = LengthPercentageAuto;
    match value {
        CssValue::Dimension(dimension) => Result::Length(resolve_length2(css, *dimension, context)),
        CssValue::Percentage(value) => Result::Percent(*value),
        Keyword(keyword) => match keyword.as_str(css) {
            "auto" => Result::Auto,
            keyword => {
                error!("keyword {keyword} not supported");
                Result::Auto
            }
        },
        _ => {
            error!("value {value:?} not supported");
            Result::Auto
        }
    }
}

fn resolve_length2(css: &str, dimension: CssDimension, context: SizeContext) -> f32 {
    let value = dimension.value;
    match dimension.unit.as_str(css) {
        "px" => value,
        "em" => context.parent_font_size * value,
        "rem" => context.root_font_size * value,
        "vw" => context.viewport_width * value / 100.0,
        "vh" => context.viewport_height * value / 100.0,
        value => {
            error!("length value {value:?} not supported");
            context.parent_font_size
        }
    }
}

fn map_overflow2(keyword: &str) -> Overflow {
    match keyword {
        "visible" => Overflow::Visible,
        "hidden" => Overflow::Hidden,
        "clip" => Overflow::Clip,
        "scroll" => Overflow::Scroll,
        "auto" => Overflow::Scroll,
        _ => {
            error!("overflow keyword {keyword} not supported");
            Overflow::Visible
        }
    }
}

fn map_align_items(keyword: &str) -> Option<taffy::AlignItems> {
    let align = match keyword {
        "normal" => return None,
        "start" => taffy::AlignItems::Start,
        "end" => taffy::AlignItems::End,
        "flex-start" => taffy::AlignItems::FlexStart,
        "flex-end" => taffy::AlignItems::FlexEnd,
        "center" => taffy::AlignItems::Center,
        "baseline" => taffy::AlignItems::Baseline,
        "stretch" => taffy::AlignItems::Stretch,
        _ => {
            error!("align-items keyword {keyword} not supported");
            return None;
        }
    };
    Some(align)
}

fn map_align_content(keyword: &str) -> Option<taffy::AlignContent> {
    let align = match keyword {
        "normal" => return None,
        "start" => taffy::AlignContent::Start,
        "end" => taffy::AlignContent::End,
        "flex-start" => taffy::AlignContent::FlexStart,
        "flex-end" => taffy::AlignContent::FlexEnd,
        "center" => taffy::AlignContent::Center,
        "stretch" => taffy::AlignContent::Stretch,
        "space-between" => taffy::AlignContent::SpaceBetween,
        "space-evenly" => taffy::AlignContent::SpaceEvenly,
        "space-around" => taffy::AlignContent::SpaceAround,
        _ => {
            error!("align-items keyword {keyword} not supported");
            return None;
        }
    };
    Some(align)
}
