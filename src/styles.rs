use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use lightningcss::properties::align::{
    AlignContent, AlignItems, AlignSelf, ContentDistribution, ContentPosition, GapValue,
    JustifyContent, JustifyItems, JustifySelf, SelfPosition,
};
use lightningcss::properties::background::BackgroundOrigin;
use lightningcss::properties::border::{Border, BorderSideWidth, LineStyle};
use lightningcss::properties::custom::{CustomPropertyName, Token, TokenOrValue};
use lightningcss::properties::display::{Display, DisplayInside, DisplayKeyword, DisplayOutside};
use lightningcss::properties::flex::{FlexDirection, FlexWrap};
use lightningcss::properties::font::{
    AbsoluteFontWeight, FontFamily, FontSize, FontStretch, FontStretchKeyword, FontStyle,
    FontWeight, LineHeight,
};
use lightningcss::properties::grid::{
    GridAutoFlow, GridColumn, GridLine, GridRow, RepeatCount, TrackBreadth, TrackListItem,
    TrackSize, TrackSizing,
};
use lightningcss::properties::overflow::OverflowKeyword;
use lightningcss::properties::position::Position;
use lightningcss::properties::size::{MaxSize, Size};
use lightningcss::properties::text::OverflowWrap;
use lightningcss::properties::transform::Matrix3d;
use lightningcss::properties::Property;
use lightningcss::rules::keyframes::{KeyframeSelector, KeyframesName};
use lightningcss::rules::CssRule;
use lightningcss::selector::{Combinator, Component, PseudoClass, PseudoElement};
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use lightningcss::values::color::{CssColor, RGBA};
use lightningcss::values::image::Image;
use lightningcss::values::length::{Length, LengthPercentage, LengthPercentageOrAuto, LengthValue};
use log::error;
use parcel_selectors::attr::AttrSelectorOperator;
use parcel_selectors::parser::NthType;
use static_self::IntoOwned;
use taffy::prelude::FromLength;
use taffy::prelude::TaffyAuto;
use taffy::prelude::{FromFlex, FromPercent, TaffyFitContent, TaffyMaxContent, TaffyMinContent};
use taffy::{
    Dimension, GridPlacement, GridTrackRepetition, LengthPercentageAuto, Line, Overflow, Point,
    Rect, Style, TrackSizingFunction,
};

use crate::animation::{Animation, Keyframe, Track};
use crate::html::Dom;
use crate::models::{ElementId, Object, Presentation, Ruleset, SizeContext};
use crate::{Background, Borders, Element, MyBorder, ObjectFit, Rgba, TextStyle};

impl TextStyle {
    pub const DEFAULT_FONT_FAMILY: &'static str = "system-ui";
    pub const DEFAULT_FONT_WEIGHT: u16 = 400;
    pub const DEFAULT_FONT_STRETCH: FontStretchKeyword = FontStretchKeyword::Normal;
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
            position: Default::default(),
            repeat: Default::default(),
            size: Default::default(),
            attachment: Default::default(),
            origin: BackgroundOrigin::PaddingBox,
            clip: Default::default(),
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
            font_style: FontStyle::Normal,
            font_weight: TextStyle::DEFAULT_FONT_WEIGHT,
            font_stretch: TextStyle::DEFAULT_FONT_STRETCH,
            line_height: 16.0,
            wrap: OverflowWrap::Normal,
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

pub fn inherit<'i>(parent: &Element, view: &mut Element) {
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
    view.text_style.font_style = parent.text_style.font_style.clone();
    // font-variant
    // font-weight
    view.text_style.font_weight = parent.text_style.font_weight;
    // font-size-adjust
    // font-stretch
    view.text_style.font_stretch = parent.text_style.font_stretch.clone();
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
    view.text_style.wrap = parent.text_style.wrap;
}

pub fn apply_element_rules<'i>(
    declarations: &Vec<Property<'i>>,
    parent: &Element,
    view: &mut Element,
    context: SizeContext,
    resources: &str,
) {
    inherit(parent, view);
    for property in declarations {
        match property {
            Property::AnimationTimingFunction(timing_functions, _) => {}
            Property::AnimationIterationCount(iteraction_count, _) => {}
            Property::Animation(animations, _) => {}
            Property::Background(background) => {
                if background.len() > 1 {
                    error!("multiple background not supported");
                }
                let background = &background[0];
                view.background.color = resolve_css_color(&background.color);
                view.background.image = match &background.image {
                    Image::None => None,
                    Image::Url(url) => Some(format!("{resources}{}", url.url)),
                    image => {
                        error!("background image {image:?} not supported");
                        None
                    }
                };
                view.background.position = background.position.clone();
                view.background.repeat = background.repeat.clone();
                view.background.size = background.size.clone();
                view.background.attachment = background.attachment.clone();
                view.background.clip = background.clip.clone();
                view.background.origin = background.origin.clone();
            }
            Property::BackgroundColor(color) => view.background.color = resolve_css_color(color),
            Property::BackgroundImage(image) => {
                view.background.image = match &image[0] {
                    Image::None => None,
                    Image::Url(url) => Some(format!("{resources}{}", url.url)),
                    image => {
                        error!("background image {image:?} not supported");
                        None
                    }
                }
            }
            Property::BackgroundPosition(position) => {
                view.background.position = position[0].clone()
            }
            Property::BackgroundPositionX(position) => {
                view.background.position.x = position[0].clone()
            }
            Property::BackgroundPositionY(position) => {
                view.background.position.y = position[0].clone()
            }
            Property::BackgroundRepeat(repeat) => view.background.repeat = repeat[0].clone(),
            Property::BackgroundSize(size) => view.background.size = size[0].clone(),
            Property::BackgroundAttachment(attach) => {
                view.background.attachment = attach[0].clone()
            }
            Property::BackgroundClip(clip, _) => view.background.clip = clip[0].clone(),
            Property::BackgroundOrigin(origin) => view.background.origin = origin[0].clone(),
            Property::BorderTop(border) => {
                view.borders.top = Some(resolve_border(
                    &border.width,
                    border.style,
                    &border.color,
                    context,
                ))
            }
            Property::BorderBottom(border) => {
                view.borders.bottom = Some(resolve_border(
                    &border.width,
                    border.style,
                    &border.color,
                    context,
                ))
            }
            Property::BorderLeft(border) => {
                view.borders.left = Some(resolve_border(
                    &border.width,
                    border.style,
                    &border.color,
                    context,
                ))
            }
            Property::BorderRight(border) => {
                view.borders.right = Some(resolve_border(
                    &border.width,
                    border.style,
                    &border.color,
                    context,
                ))
            }
            Property::Border(border) => {
                view.borders.top = Some(resolve_border(
                    &border.width,
                    border.style,
                    &border.color,
                    context,
                ));
                view.borders.bottom = Some(resolve_border(
                    &border.width,
                    border.style,
                    &border.color,
                    context,
                ));
                view.borders.left = Some(resolve_border(
                    &border.width,
                    border.style,
                    &border.color,
                    context,
                ));
                view.borders.right = Some(resolve_border(
                    &border.width,
                    border.style,
                    &border.color,
                    context,
                ));
            }
            Property::Color(color) => view.color = resolve_css_color(color),
            Property::FontFamily(family) => {
                view.text_style.font_family = resolve_font_family(family)
            }
            Property::FontSize(size) => {
                view.text_style.font_size = resolve_font_size(size, context)
            }
            Property::FontStyle(style) => view.text_style.font_style = style.clone(),
            Property::FontWeight(weight) => {
                view.text_style.font_weight = resolve_font_weight(weight)
            }
            Property::FontStretch(stretch) => {
                view.text_style.font_stretch = resolve_font_stretch(stretch)
            }
            Property::LineHeight(height) => {
                view.text_style.line_height =
                    resolve_line_height(height, context, view.text_style.font_size)
            }
            Property::Font(font) => {
                view.text_style.font_family = resolve_font_family(&font.family);
                view.text_style.font_size = resolve_font_size(&font.size, context);
                view.text_style.font_style = font.style.clone();
                view.text_style.font_weight = resolve_font_weight(&font.weight);
                view.text_style.font_stretch = resolve_font_stretch(&font.stretch);
                view.text_style.line_height =
                    resolve_line_height(&font.line_height, context, view.text_style.font_size);
            }
            Property::OverflowWrap(wrap) => view.text_style.wrap = wrap.clone(),
            Property::WordWrap(wrap) => view.text_style.wrap = wrap.clone(),
            Property::Transform(transforms, _) => match transforms.to_matrix() {
                None => {
                    error!("unable to handle transform matrix {property:?}");
                }
                Some(transform) => view.transform = Some(transform),
            },
            Property::Custom(custom) => match &custom.name {
                CustomPropertyName::Unknown(ident) => match ident.0.as_ref() {
                    "object-fit" => {
                        let value = custom.value.0.get(0).map(|token| match token {
                            TokenOrValue::Token(token) => match token {
                                Token::Ident(ident) => ident.as_ref(),
                                _ => "",
                            },
                            _ => "",
                        });
                        view.object_fit = match value {
                            Some("contain") => ObjectFit::Contain,
                            Some("cover") => ObjectFit::Cover,
                            Some("fill") => ObjectFit::Fill,
                            Some("none") => ObjectFit::None,
                            Some("scale-down") => ObjectFit::ScaleDown,
                            _ => {
                                error!("object-fit value {:?} not supported", value);
                                ObjectFit::Fill
                            }
                        }
                    }
                    _ => {
                        error!("css property {:?} not supported", ident)
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }
}

pub fn apply_layout_rules<'i>(
    declarations: &Vec<Property<'i>>,
    style: &mut Style,
    context: SizeContext,
) {
    for property in declarations {
        match property {
            Property::Display(value) => match value {
                Display::Keyword(keyword) => match keyword {
                    DisplayKeyword::None => style.display = taffy::Display::None,
                    keyword => {
                        error!("display keyword {keyword:?} not supported")
                    }
                },
                Display::Pair(pair) => match pair.outside {
                    DisplayOutside::Block => match &pair.inside {
                        DisplayInside::Flow => style.display = taffy::Display::Block,
                        DisplayInside::Flex(_) => style.display = taffy::Display::Flex,
                        DisplayInside::Grid => style.display = taffy::Display::Grid,
                        inside => {
                            error!("display block inside {inside:?} not supported")
                        }
                    },
                    outside => {
                        error!("display outside {outside:?} not supported")
                    }
                },
            },
            Property::Overflow(overflow) => {
                style.overflow.x = map_overflow(overflow.x);
                style.overflow.y = map_overflow(overflow.y);
            }
            Property::OverflowX(overflow) => {
                style.overflow.x = map_overflow(*overflow);
            }
            Property::OverflowY(overflow) => {
                style.overflow.y = map_overflow(*overflow);
            }
            Property::Position(position) => match position {
                Position::Relative => style.position = taffy::Position::Relative,
                Position::Absolute => style.position = taffy::Position::Absolute,
                position => {
                    error!("position {position:?} not supported")
                }
            },
            Property::Inset(inset) => {
                style.inset.left = inset.left.resolve(context);
                style.inset.right = inset.right.resolve(context);
                style.inset.top = inset.top.resolve(context);
                style.inset.bottom = inset.bottom.resolve(context);
            }
            Property::Left(left) => style.inset.left = left.resolve(context),
            Property::Right(right) => style.inset.right = right.resolve(context),
            Property::Top(top) => style.inset.top = top.resolve(context),
            Property::Bottom(bottom) => style.inset.bottom = bottom.resolve(context),
            Property::Width(size) => style.size.width = size.resolve(context),
            Property::Height(size) => style.size.height = size.resolve(context),
            Property::MinWidth(size) => style.min_size.width = size.resolve(context),
            Property::MinHeight(size) => style.min_size.height = size.resolve(context),
            Property::MaxWidth(size) => style.max_size.width = size.resolve(context),
            Property::MaxHeight(size) => style.max_size.height = size.resolve(context),
            Property::AspectRatio(ratio) => match &ratio.ratio {
                None => style.aspect_ratio = None,
                Some(ratio) => style.aspect_ratio = Some(ratio.0 / ratio.1),
            },
            Property::Margin(margin) => {
                style.margin.left = margin.left.resolve(context);
                style.margin.right = margin.right.resolve(context);
                style.margin.top = margin.top.resolve(context);
                style.margin.bottom = margin.bottom.resolve(context);
            }
            Property::MarginLeft(left) => style.margin.left = left.resolve(context),
            Property::MarginRight(right) => style.margin.right = right.resolve(context),
            Property::MarginTop(top) => style.margin.top = top.resolve(context),
            Property::MarginBottom(bottom) => style.margin.bottom = bottom.resolve(context),
            Property::Padding(padding) => {
                style.padding.left = padding.left.resolve(context);
                style.padding.right = padding.right.resolve(context);
                style.padding.top = padding.top.resolve(context);
                style.padding.bottom = padding.bottom.resolve(context);
            }
            Property::PaddingLeft(left) => style.padding.left = left.resolve(context),
            Property::PaddingRight(right) => style.padding.right = right.resolve(context),
            Property::PaddingTop(top) => style.padding.top = top.resolve(context),
            Property::PaddingBottom(bottom) => style.padding.bottom = bottom.resolve(context),
            Property::Border(border) => {
                style.border.left = border.width.resolve(context);
                style.border.right = border.width.resolve(context);
                style.border.top = border.width.resolve(context);
                style.border.bottom = border.width.resolve(context);
            }
            Property::BorderLeftWidth(left) => style.border.left = left.resolve(context),
            Property::BorderRightWidth(right) => style.border.right = right.resolve(context),
            Property::BorderTopWidth(top) => style.border.top = top.resolve(context),
            Property::BorderBottomWidth(bottom) => style.border.bottom = bottom.resolve(context),
            Property::AlignItems(align, _) => match align {
                AlignItems::Normal => style.align_items = None,
                AlignItems::Stretch => style.align_items = Some(taffy::AlignItems::Stretch),
                AlignItems::BaselinePosition(_) => {
                    style.align_items = Some(taffy::AlignItems::Baseline)
                }
                AlignItems::SelfPosition { value, .. } => {
                    style.align_items = map_self_position(value)
                } // align => error!("align {align:?} not supported")
            },
            Property::AlignSelf(align, _) => match align {
                AlignSelf::Auto => style.align_self = None,
                AlignSelf::Normal => style.align_self = None,
                AlignSelf::Stretch => style.align_self = Some(taffy::AlignSelf::Stretch),
                AlignSelf::BaselinePosition(_) => {
                    style.align_self = Some(taffy::AlignSelf::Baseline)
                }
                AlignSelf::SelfPosition { value, .. } => {
                    style.align_self = map_self_position(value)
                } // align => error!("align {align:?} not supported")
            },
            Property::JustifyItems(justify) => match justify {
                JustifyItems::Normal => style.justify_items = None,
                JustifyItems::Stretch => style.justify_items = Some(taffy::JustifyItems::Stretch),
                JustifyItems::BaselinePosition(_) => {
                    style.justify_items = Some(taffy::JustifyItems::Baseline)
                }
                JustifyItems::SelfPosition { value, .. } => {
                    style.justify_items = map_self_position(value)
                }
                justify => error!("justify {justify:?} not supported"),
            },
            Property::JustifySelf(justify) => match justify {
                JustifySelf::Auto => style.justify_self = None,
                JustifySelf::Normal => style.justify_self = None,
                JustifySelf::Stretch => style.justify_self = Some(taffy::JustifySelf::Stretch),
                JustifySelf::BaselinePosition(_) => {
                    style.justify_self = Some(taffy::JustifySelf::Baseline)
                }
                JustifySelf::SelfPosition { value, .. } => {
                    style.justify_self = map_self_position(value)
                }
                justify => error!("justify {justify:?} not supported"),
            },
            Property::AlignContent(align, _) => match align {
                AlignContent::ContentDistribution(distribution) => match distribution {
                    ContentDistribution::SpaceBetween => {
                        style.align_content = Some(taffy::AlignContent::SpaceBetween)
                    }
                    ContentDistribution::SpaceAround => {
                        style.align_content = Some(taffy::AlignContent::SpaceAround)
                    }
                    ContentDistribution::SpaceEvenly => {
                        style.align_content = Some(taffy::AlignContent::SpaceEvenly)
                    }
                    ContentDistribution::Stretch => {
                        style.align_content = Some(taffy::AlignContent::Stretch)
                    }
                },
                AlignContent::ContentPosition { value, .. } => {
                    style.align_content = match value {
                        ContentPosition::Center => Some(taffy::AlignContent::Center),
                        ContentPosition::Start => Some(taffy::AlignContent::Start),
                        ContentPosition::End => Some(taffy::AlignContent::End),
                        ContentPosition::FlexStart => Some(taffy::AlignContent::FlexStart),
                        ContentPosition::FlexEnd => Some(taffy::AlignContent::FlexEnd),
                    }
                }
                align => error!("align content {align:?} not supported"),
            },
            Property::JustifyContent(justify, _) => match justify {
                JustifyContent::ContentDistribution(distribution) => match distribution {
                    ContentDistribution::SpaceBetween => {
                        style.justify_content = Some(taffy::JustifyContent::SpaceBetween)
                    }
                    ContentDistribution::SpaceAround => {
                        style.justify_content = Some(taffy::JustifyContent::SpaceAround)
                    }
                    ContentDistribution::SpaceEvenly => {
                        style.justify_content = Some(taffy::JustifyContent::SpaceEvenly)
                    }
                    ContentDistribution::Stretch => {
                        style.justify_content = Some(taffy::JustifyContent::Stretch)
                    }
                },
                JustifyContent::ContentPosition { value, .. } => {
                    style.justify_content = match value {
                        ContentPosition::Center => Some(taffy::JustifyContent::Center),
                        ContentPosition::Start => Some(taffy::JustifyContent::Start),
                        ContentPosition::End => Some(taffy::JustifyContent::End),
                        ContentPosition::FlexStart => Some(taffy::JustifyContent::FlexStart),
                        ContentPosition::FlexEnd => Some(taffy::JustifyContent::FlexEnd),
                    }
                }
                justify => error!("justify content {justify:?} not supported"),
            },
            Property::Gap(gap) => {
                style.gap.width = gap.column.resolve(context);
                style.gap.height = gap.row.resolve(context);
            }
            Property::ColumnGap(value) => style.gap.width = value.resolve(context),
            Property::RowGap(value) => style.gap.height = value.resolve(context),
            Property::FlexDirection(direction, _) => {
                style.flex_direction = match direction {
                    FlexDirection::Row => taffy::FlexDirection::Row,
                    FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
                    FlexDirection::Column => taffy::FlexDirection::Column,
                    FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
                }
            }
            Property::FlexWrap(wrap, _) => {
                style.flex_wrap = match wrap {
                    FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
                    FlexWrap::Wrap => taffy::FlexWrap::Wrap,
                    FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
                }
            }
            Property::FlexBasis(basis, _) => {
                style.flex_basis = match basis {
                    LengthPercentageOrAuto::Auto => Dimension::Auto,
                    LengthPercentageOrAuto::LengthPercentage(value) => {
                        resolve_dimension(value, context)
                    }
                }
            }
            Property::FlexGrow(grow, _) => style.flex_grow = *grow,
            Property::FlexShrink(shrink, _) => style.flex_shrink = *shrink,
            Property::GridTemplate(template) => {
                style.grid_template_rows = map_track_sizing(&template.rows, context);
                style.grid_template_columns = map_track_sizing(&template.columns, context);
                // template.areas
            }
            Property::GridTemplateRows(rows) => {
                style.grid_template_rows = map_track_sizing(rows, context)
            }
            Property::GridTemplateColumns(rows) => {
                style.grid_template_columns = map_track_sizing(rows, context)
            }
            Property::GridAutoColumns(_columns) => {
                error!("grid auto columns not supported");
            }
            Property::GridAutoRows(_rows) => {
                error!("grid auto rows not supported");
                // style.grid_auto_rows = map_track_sizing(rows, context);
            }
            Property::GridAutoFlow(flow) => {
                style.grid_auto_flow = match *flow {
                    GridAutoFlow::Row => taffy::GridAutoFlow::Row,
                    GridAutoFlow::Column => taffy::GridAutoFlow::Column,
                    flow => {
                        error!("grid flow {flow:?} not supported");
                        taffy::GridAutoFlow::default()
                    }
                }
            }
            Property::GridRow(row) => style.grid_row = row.resolve(context),
            Property::GridColumn(column) => style.grid_column = column.resolve(context),
            _ => {}
        }
    }
}

fn map_overflow(keyword: OverflowKeyword) -> Overflow {
    match keyword {
        OverflowKeyword::Visible => Overflow::Visible,
        OverflowKeyword::Hidden => Overflow::Hidden,
        OverflowKeyword::Clip => Overflow::Clip,
        OverflowKeyword::Scroll => Overflow::Scroll,
        OverflowKeyword::Auto => Overflow::Scroll,
    }
}

fn map_self_position(value: &SelfPosition) -> Option<taffy::AlignItems> {
    match value {
        SelfPosition::Center => Some(taffy::AlignItems::Center),
        SelfPosition::Start => Some(taffy::AlignItems::Start),
        SelfPosition::End => Some(taffy::AlignItems::End),
        SelfPosition::FlexStart => Some(taffy::AlignItems::FlexStart),
        SelfPosition::FlexEnd => Some(taffy::AlignItems::FlexEnd),
        pos => {
            error!("align {pos:?} not supported");
            None
        }
    }
}

fn map_track_item(item: &TrackListItem, context: SizeContext) -> TrackSizingFunction {
    match item {
        TrackListItem::TrackSize(size) => match size {
            TrackSize::TrackBreadth(breadth) => match breadth {
                TrackBreadth::Length(length) => match length {
                    LengthPercentage::Dimension(value) => match value {
                        LengthValue::Px(pixels) => TrackSizingFunction::from_length(*pixels),
                        length => {
                            error!("length {length:?} not supported");
                            TrackSizingFunction::AUTO
                        }
                    },
                    LengthPercentage::Percentage(percentage) => {
                        TrackSizingFunction::from_percent(percentage.0)
                    }
                    LengthPercentage::Calc(calc) => {
                        error!("calc {calc:?} not supported");
                        TrackSizingFunction::AUTO
                    }
                },
                TrackBreadth::Flex(flex) => TrackSizingFunction::from_flex(*flex),
                TrackBreadth::MinContent => TrackSizingFunction::MIN_CONTENT,
                TrackBreadth::MaxContent => TrackSizingFunction::MAX_CONTENT,
                TrackBreadth::Auto => TrackSizingFunction::AUTO,
            },
            TrackSize::MinMax { .. } => {
                error!("grid min max not supported yet");
                TrackSizingFunction::AUTO
            }
            TrackSize::FitContent(length) => {
                TrackSizingFunction::fit_content(resolve_length_percentage(length, context))
            }
        },
        TrackListItem::TrackRepeat(repeat) => {
            error!("grid repeat not supported yet");
            TrackSizingFunction::Repeat(
                match repeat.count {
                    RepeatCount::Number(count) => GridTrackRepetition::Count(count as u16),
                    RepeatCount::AutoFill => GridTrackRepetition::AutoFill,
                    RepeatCount::AutoFit => GridTrackRepetition::AutoFit,
                },
                vec![],
            )
        }
    }
}

fn map_track_sizing(sizing: &TrackSizing, context: SizeContext) -> Vec<TrackSizingFunction> {
    match sizing {
        TrackSizing::None => vec![],
        TrackSizing::TrackList(track) => {
            let mut result = vec![];
            for item in &track.items {
                result.push(map_track_item(item, context));
            }
            result
        }
    }
}

fn resolve_border(
    width: &BorderSideWidth,
    style: LineStyle,
    color: &CssColor,
    context: SizeContext,
) -> MyBorder {
    MyBorder {
        width: match width {
            BorderSideWidth::Length(length) => match length {
                Length::Value(length) => resolve_length(length, context),
                width => {
                    error!("border width {width:?} not supported");
                    1.0
                }
            },
            width => {
                error!("border width {width:?} not supported");
                1.0
            }
        },
        style,
        color: resolve_css_color(color),
    }
}

fn resolve_css_color(color: &CssColor) -> Rgba {
    match color {
        CssColor::RGBA(rgba) => [rgba.red, rgba.green, rgba.blue, rgba.alpha],
        _ => {
            error!("color {color:?} not supported");
            [0; 4]
        }
    }
}

fn resolve_font_family(declaration: &Vec<FontFamily>) -> String {
    match declaration.get(0) {
        None => {
            error!("empty font family declaration");
            TextStyle::DEFAULT_FONT_FAMILY.to_string()
        }
        Some(family) => match family {
            FontFamily::FamilyName(name) => name.to_string(),
            family => {
                error!("family {family:?} not supported");
                TextStyle::DEFAULT_FONT_FAMILY.to_string()
            }
        },
    }
}

fn resolve_font_stretch(stretch: &FontStretch) -> FontStretchKeyword {
    match stretch {
        FontStretch::Keyword(keyword) => keyword.clone(),
        stretch => {
            error!("stretch {stretch:?} not supported");
            TextStyle::DEFAULT_FONT_STRETCH
        }
    }
}

fn resolve_font_weight(weight: &FontWeight) -> u16 {
    match weight {
        FontWeight::Absolute(weight) => match weight {
            AbsoluteFontWeight::Weight(value) => {
                if *value > 100.0 && *value < 1000.0 {
                    *value as u16
                } else {
                    error!("weight {value} not supported");
                    TextStyle::DEFAULT_FONT_WEIGHT
                }
            }
            AbsoluteFontWeight::Normal => 400,
            AbsoluteFontWeight::Bold => 700,
        },
        weight => {
            error!("weight {weight:?} not supported");
            TextStyle::DEFAULT_FONT_WEIGHT
        }
    }
}

fn resolve_line_height(height: &LineHeight, context: SizeContext, font_size: f32) -> f32 {
    match height {
        LineHeight::Normal => 1.2 * font_size,
        LineHeight::Number(multiplier) => (multiplier * font_size).floor(),
        LineHeight::Length(height) => match height {
            LengthPercentage::Dimension(height) => resolve_length(height, context),
            height => {
                error!("line height {height:?} not supported");
                context.parent_font_size
            }
        },
    }
}

fn resolve_font_size(size: &FontSize, context: SizeContext) -> f32 {
    match size {
        FontSize::Length(size) => match size {
            LengthPercentage::Dimension(size) => resolve_length(size, context),
            size => {
                error!("font size {size:?} not supported");
                context.parent_font_size
            }
        },
        FontSize::Absolute(size) => match size {
            size => {
                error!("font size {size:?} not supported");
                context.parent_font_size
            }
        },
        FontSize::Relative(size) => match size {
            size => {
                error!("font size {size:?} not supported");
                context.parent_font_size
            }
        },
    }
}

fn resolve_dimension(value: &LengthPercentage, context: SizeContext) -> Dimension {
    match value {
        LengthPercentage::Dimension(value) => Dimension::Length(resolve_length(value, context)),
        LengthPercentage::Percentage(percentage) => Dimension::Percent(percentage.0),
        LengthPercentage::Calc(calc) => {
            error!("calc {calc:?} not supported");
            Dimension::Length(0.0)
        }
    }
}

fn resolve_length_percentage(
    value: &LengthPercentage,
    context: SizeContext,
) -> taffy::LengthPercentage {
    match value {
        LengthPercentage::Dimension(length) => {
            taffy::LengthPercentage::Length(resolve_length(length, context))
        }
        LengthPercentage::Percentage(percentage) => taffy::LengthPercentage::Percent(percentage.0),
        LengthPercentage::Calc(calc) => {
            error!("calc {calc:?} not supported");
            taffy::LengthPercentage::Length(0.0)
        }
    }
}

fn resolve_length(value: &LengthValue, context: SizeContext) -> f32 {
    match value {
        LengthValue::Px(value) => *value,
        LengthValue::Em(value) => context.parent_font_size * value,
        LengthValue::Rem(value) => context.root_font_size * value,
        LengthValue::Vw(value) => context.viewport_width * value / 100.0,
        LengthValue::Vh(value) => context.viewport_height * value / 100.0,
        value => {
            error!("length value {value:?} not supported");
            context.parent_font_size
        }
    }
}

trait Resolver<T> {
    fn resolve(&self, context: SizeContext) -> T;
}

impl Resolver<Dimension> for MaxSize {
    fn resolve(&self, context: SizeContext) -> Dimension {
        match self {
            MaxSize::None => Dimension::Auto,
            MaxSize::LengthPercentage(value) => resolve_dimension(value, context),
            dimension => {
                error!("max-size {dimension:?} not supported");
                Dimension::Length(0.0)
            }
        }
    }
}

impl Resolver<Dimension> for Size {
    fn resolve(&self, context: SizeContext) -> Dimension {
        match self {
            Size::Auto => Dimension::Auto,
            Size::LengthPercentage(value) => resolve_dimension(value, context),
            dimension => {
                error!("size {dimension:?} not supported");
                Dimension::Length(0.0)
            }
        }
    }
}

impl Resolver<LengthPercentageAuto> for LengthPercentageOrAuto {
    fn resolve(&self, context: SizeContext) -> LengthPercentageAuto {
        match self {
            LengthPercentageOrAuto::Auto => LengthPercentageAuto::Auto,
            LengthPercentageOrAuto::LengthPercentage(value) => match value {
                LengthPercentage::Dimension(length) => {
                    LengthPercentageAuto::Length(resolve_length(length, context))
                }
                LengthPercentage::Percentage(percentage) => {
                    LengthPercentageAuto::Percent(percentage.0)
                }
                LengthPercentage::Calc(calc) => {
                    error!("calc {calc:?} not supported");
                    LengthPercentageAuto::Length(0.0)
                }
            },
        }
    }
}

impl Resolver<taffy::LengthPercentage> for LengthPercentageOrAuto {
    fn resolve(&self, context: SizeContext) -> taffy::LengthPercentage {
        match self {
            LengthPercentageOrAuto::Auto => taffy::LengthPercentage::Length(0.0),
            LengthPercentageOrAuto::LengthPercentage(value) => {
                resolve_length_percentage(value, context)
            }
        }
    }
}

impl Resolver<taffy::LengthPercentage> for BorderSideWidth {
    fn resolve(&self, context: SizeContext) -> taffy::LengthPercentage {
        match self {
            BorderSideWidth::Thin => taffy::LengthPercentage::Length(1.0),
            BorderSideWidth::Medium => taffy::LengthPercentage::Length(2.0),
            BorderSideWidth::Thick => taffy::LengthPercentage::Length(3.0),
            BorderSideWidth::Length(length) => match length {
                Length::Value(length) => {
                    taffy::LengthPercentage::Length(resolve_length(length, context))
                }
                Length::Calc(calc) => {
                    error!("calc {calc:?} not supported");
                    taffy::LengthPercentage::Length(0.0)
                }
            },
        }
    }
}

impl Resolver<taffy::LengthPercentage> for GapValue {
    fn resolve(&self, context: SizeContext) -> taffy::LengthPercentage {
        match self {
            GapValue::Normal => taffy::LengthPercentage::Length(0.0),
            GapValue::LengthPercentage(value) => resolve_length_percentage(value, context),
        }
    }
}

fn map_grid_line(line: &GridLine) -> GridPlacement {
    match line {
        GridLine::Auto => GridPlacement::Auto,
        GridLine::Area { .. } => {
            error!("grid area placement not supported");
            GridPlacement::Auto
        }
        GridLine::Line { index, .. } => GridPlacement::Line((*index as i16).into()),
        GridLine::Span { index, .. } => GridPlacement::Span(*index as u16),
    }
}

impl Resolver<Line<GridPlacement>> for GridRow<'_> {
    fn resolve(&self, _context: SizeContext) -> Line<GridPlacement> {
        Line {
            start: map_grid_line(&self.start),
            end: map_grid_line(&self.end),
        }
    }
}

impl Resolver<Line<GridPlacement>> for GridColumn<'_> {
    fn resolve(&self, _context: SizeContext) -> Line<GridPlacement> {
        Line {
            start: map_grid_line(&self.start),
            end: map_grid_line(&self.end),
        }
    }
}

pub fn parse_presentation(code: &str) -> Presentation {
    let sheet = StyleSheet::parse(code, ParserOptions::default()).unwrap();
    let mut rules = vec![];
    let mut animations = HashMap::new();
    for rule in sheet.rules.0 {
        match rule {
            CssRule::Style(style) => {
                let style = style.into_owned();
                let style = Ruleset { style };
                rules.push(style);
            }
            CssRule::Keyframes(animation) => {
                let mut tracks: HashMap<String, Track> = HashMap::new();
                let name = match animation.name {
                    KeyframesName::Ident(ident) => ident.0.to_string(),
                    KeyframesName::Custom(name) => name.to_string(),
                };
                for keyframe in animation.keyframes {
                    for selector in keyframe.selectors {
                        let time = match selector {
                            KeyframeSelector::Percentage(time) => time.0,
                            KeyframeSelector::From => 0.0,
                            KeyframeSelector::To => 1.0,
                        };
                        for property in &keyframe.declarations.declarations {
                            let key = property.property_id().name().to_string();
                            let keyframe = Keyframe {
                                time,
                                property: property.clone().into_owned(),
                            };
                            tracks.entry(key).or_default().keyframes.push(keyframe);
                        }
                    }
                }
                let mut tracks: Vec<Track> = tracks.into_values().collect();
                for track in tracks.iter_mut() {
                    track.keyframes.sort_by(|a, b| a.time.total_cmp(&b.time));
                }
                animations.insert(name.clone(), Rc::new(Animation { name, tracks }));
            }
            _ => {}
        }
    }
    Presentation { rules, animations }
}

#[cfg(test)]
mod tests {
    use crate::styles::parse_presentation;

    #[test]
    pub fn test_something() {
        let css = r#"
        .myClass {
            background: red;
        }
        #myId {
            background: red;
        }
        div {
            background: red;
        }
        #myContainer > div > span {
            background: red;
        }
        .myA.myB {
            background: red;
        }
        .myA .myB {
            background: red;
        }
        input:focus {
            background: red;
        }
        dd:last-of-type {
            background: red;
        }
        di:last-child {
            background: red;
        }
        .todo[data-done="true"]:hover {
            background: red;
        }
        li:nth-child(even) {
            background: red;
        }

        "#;
        parse_presentation(css);
    }
}
