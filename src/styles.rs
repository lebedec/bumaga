use lightningcss::properties::align::{AlignContent, AlignItems, AlignSelf, ContentDistribution, ContentPosition, GapValue, JustifyContent, JustifyItems, JustifySelf, SelfPosition};
use lightningcss::properties::border::BorderSideWidth;
use lightningcss::properties::display::{Display, DisplayInside, DisplayKeyword, DisplayOutside};
use lightningcss::properties::flex::{FlexDirection, FlexWrap};
use lightningcss::properties::font::{FontSize};
use lightningcss::properties::grid::{GridAutoFlow, GridColumn, GridLine, GridRow, RepeatCount, TrackBreadth, TrackListItem, TrackSize, TrackSizing};
use lightningcss::properties::overflow::OverflowKeyword;
use lightningcss::properties::position::Position;
use lightningcss::properties::Property;
use lightningcss::properties::size::{MaxSize, Size};
use lightningcss::values::color::CssColor;
use lightningcss::values::image::Image;
use lightningcss::values::length::{Length, LengthPercentage, LengthPercentageOrAuto, LengthValue};
use log::error;
use taffy::{Dimension, GridPlacement, GridTrackRepetition, LengthPercentageAuto, Line, Overflow, Style, TrackSizingFunction};
use taffy::prelude::{FromFlex, FromPercent, TaffyFitContent, TaffyMaxContent, TaffyMinContent};
use crate::{Rectangle, Ruleset, SizeContext};
use taffy::prelude::FromLength;
use taffy::prelude::TaffyAuto;

pub fn inherit<'i>(parent: &Rectangle, rectangle: &mut Rectangle) {
    // border-collapse
    // border-spacing
    // caption-side
    // color
    rectangle.color = parent.color;
    // cursor
    // direction
    // empty-cells
    // font-family
    // font-size
    rectangle.font_size = parent.font_size;
    // font-style
    // font-variant
    // font-weight
    // font-size-adjust
    // font-stretch
    // font
    // letter-spacing
    // line-height
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
}

pub fn apply_rectangle_rules<'i>(ruleset: &Ruleset<'i>, parent: &Rectangle, rectangle: &mut Rectangle, context: SizeContext) {
    inherit(parent, rectangle);
    for property in &ruleset.style.declarations.declarations {
        match property {
            Property::Background(background) => {
                if background.len() > 1 {
                    error!("multiple background not supported");
                }
                let background = &background[0];
                rectangle.background.color = background.color.clone();
                rectangle.background.image = match &background.image {
                    Image::None => None,
                    Image::Url(url) => Some(url.url.to_string()),
                    image => {
                        error!("background image {image:?} not supported");
                        None
                    }
                };
                rectangle.background.position = background.position.clone();
                rectangle.background.repeat = background.repeat.clone();
                rectangle.background.size = background.size.clone();
                rectangle.background.attachment = background.attachment.clone();
                rectangle.background.clip = background.clip.clone();
                rectangle.background.origin = background.origin.clone();
            }
            Property::BackgroundColor(color) => rectangle.background.color = color.clone(),
            Property::BackgroundImage(image) => rectangle.background.image = match &image[0] {
                Image::None => None,
                Image::Url(url) => Some(url.url.to_string()),
                image => {
                    error!("background image {image:?} not supported");
                    None
                }
            },
            Property::BackgroundPosition(position) => rectangle.background.position = position[0].clone(),
            Property::BackgroundPositionX(position) => rectangle.background.position.x = position[0].clone(),
            Property::BackgroundPositionY(position) => rectangle.background.position.y = position[0].clone(),
            Property::BackgroundRepeat(repeat) => rectangle.background.repeat = repeat[0].clone(),
            Property::BackgroundSize(size) => rectangle.background.size = size[0].clone(),
            Property::BackgroundAttachment(attach) => rectangle.background.attachment = attach[0].clone(),
            Property::BackgroundClip(clip, _) => rectangle.background.clip = clip[0].clone(),
            Property::BackgroundOrigin(origin) => rectangle.background.origin = origin[0].clone(),
            Property::Color(color) => {
                match color {
                    CssColor::RGBA(color) => rectangle.color = *color,
                    color => error!("color {color:?} not supported")
                };
            }
            Property::FontSize(size) => rectangle.font_size = resolve_font_size(size, context),
            Property::Font(font) => {
                rectangle.font_size = resolve_font_size(&font.size, context)
            }
            _ => {}
        }
    }
}

pub fn apply_style_rules(ruleset: &Ruleset, style: &mut Style, context: SizeContext) {
    for property in &ruleset.style.declarations.declarations {
        match property {
            Property::Display(value) => match value {
                Display::Keyword(keyword) => match keyword {
                    DisplayKeyword::None => style.display = taffy::Display::None,
                    keyword => {
                        error!("display keyword {keyword:?} not supported")
                    }
                }
                Display::Pair(pair) => {
                    match pair.outside {
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
                    }
                }
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
            }
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
                Some(ratio) => style.aspect_ratio = Some(ratio.0 / ratio.1)
            }
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
                AlignItems::BaselinePosition(_) => style.align_items = Some(taffy::AlignItems::Baseline),
                AlignItems::SelfPosition { value, .. } => style.align_items = map_self_position(value),
                // align => error!("align {align:?} not supported")
            },
            Property::AlignSelf(align, _) => match align {
                AlignSelf::Auto => style.align_self = None,
                AlignSelf::Normal => style.align_self = None,
                AlignSelf::Stretch => style.align_self = Some(taffy::AlignSelf::Stretch),
                AlignSelf::BaselinePosition(_) => style.align_self = Some(taffy::AlignSelf::Baseline),
                AlignSelf::SelfPosition { value, .. } => style.align_self = map_self_position(value),
                // align => error!("align {align:?} not supported")
            }
            Property::JustifyItems(justify) => match justify {
                JustifyItems::Normal => style.justify_items = None,
                JustifyItems::Stretch => style.justify_items = Some(taffy::JustifyItems::Stretch),
                JustifyItems::BaselinePosition(_) => style.justify_items = Some(taffy::JustifyItems::Baseline),
                JustifyItems::SelfPosition { value, .. } => style.justify_items = map_self_position(value),
                justify => error!("justify {justify:?} not supported")
            }
            Property::JustifySelf(justify) => match justify {
                JustifySelf::Auto => style.justify_self = None,
                JustifySelf::Normal => style.justify_self = None,
                JustifySelf::Stretch => style.justify_self = Some(taffy::JustifySelf::Stretch),
                JustifySelf::BaselinePosition(_) => style.justify_self = Some(taffy::JustifySelf::Baseline),
                JustifySelf::SelfPosition { value, .. } => style.justify_self = map_self_position(value),
                justify => error!("justify {justify:?} not supported")
            }
            Property::AlignContent(align, _) => match align {
                AlignContent::ContentDistribution(distribution) => match distribution {
                    ContentDistribution::SpaceBetween => style.align_content = Some(taffy::AlignContent::SpaceBetween),
                    ContentDistribution::SpaceAround => style.align_content = Some(taffy::AlignContent::SpaceAround),
                    ContentDistribution::SpaceEvenly => style.align_content = Some(taffy::AlignContent::SpaceEvenly),
                    ContentDistribution::Stretch => style.align_content = Some(taffy::AlignContent::Stretch),
                },
                AlignContent::ContentPosition { value, .. } => style.align_content = match value {
                    ContentPosition::Center => Some(taffy::AlignContent::Center),
                    ContentPosition::Start => Some(taffy::AlignContent::Start),
                    ContentPosition::End => Some(taffy::AlignContent::End),
                    ContentPosition::FlexStart => Some(taffy::AlignContent::FlexStart),
                    ContentPosition::FlexEnd => Some(taffy::AlignContent::FlexEnd),
                },
                align => error!("align content {align:?} not supported")
            }
            Property::JustifyContent(justify, _) => match justify {
                JustifyContent::ContentDistribution(distribution) => match distribution {
                    ContentDistribution::SpaceBetween => style.justify_content = Some(taffy::JustifyContent::SpaceBetween),
                    ContentDistribution::SpaceAround => style.justify_content = Some(taffy::JustifyContent::SpaceAround),
                    ContentDistribution::SpaceEvenly => style.justify_content = Some(taffy::JustifyContent::SpaceEvenly),
                    ContentDistribution::Stretch => style.justify_content = Some(taffy::JustifyContent::Stretch),
                },
                JustifyContent::ContentPosition { value, .. } => style.justify_content = match value {
                    ContentPosition::Center => Some(taffy::JustifyContent::Center),
                    ContentPosition::Start => Some(taffy::JustifyContent::Start),
                    ContentPosition::End => Some(taffy::JustifyContent::End),
                    ContentPosition::FlexStart => Some(taffy::JustifyContent::FlexStart),
                    ContentPosition::FlexEnd => Some(taffy::JustifyContent::FlexEnd),
                },
                justify => error!("justify content {justify:?} not supported")
            }
            Property::Gap(gap) => {
                style.gap.width = gap.column.resolve(context);
                style.gap.height = gap.row.resolve(context);
            }
            Property::ColumnGap(value) => style.gap.width = value.resolve(context),
            Property::RowGap(value) => style.gap.height = value.resolve(context),
            Property::FlexDirection(direction, _) => style.flex_direction = match direction {
                FlexDirection::Row => taffy::FlexDirection::Row,
                FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
                FlexDirection::Column => taffy::FlexDirection::Column,
                FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
            },
            Property::FlexWrap(wrap, _) => style.flex_wrap = match wrap {
                FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
                FlexWrap::Wrap => taffy::FlexWrap::Wrap,
                FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
            },
            Property::FlexBasis(basis, _) => style.flex_basis = match basis {
                LengthPercentageOrAuto::Auto => Dimension::Auto,
                LengthPercentageOrAuto::LengthPercentage(value) => resolve_dimension(value, context)
            },
            Property::FlexGrow(grow, _) => style.flex_grow = *grow,
            Property::FlexShrink(shrink, _) => style.flex_shrink = *shrink,
            Property::GridTemplate(template) => {
                style.grid_template_rows = map_track_sizing(&template.rows, context);
                style.grid_template_columns = map_track_sizing(&template.columns, context);
                // template.areas
            }
            Property::GridTemplateRows(rows) => style.grid_template_rows = map_track_sizing(rows, context),
            Property::GridTemplateColumns(rows) => style.grid_template_columns = map_track_sizing(rows, context),
            Property::GridAutoColumns(_columns) => {
                error!("grid auto columns not supported");
            }
            Property::GridAutoRows(_rows) => {
                error!("grid auto rows not supported");
                // style.grid_auto_rows = map_track_sizing(rows, context);
            }
            Property::GridAutoFlow(flow) => style.grid_auto_flow = match *flow {
                GridAutoFlow::Row => taffy::GridAutoFlow::Row,
                GridAutoFlow::Column => taffy::GridAutoFlow::Column,
                flow => {
                    error!("grid flow {flow:?} not supported");
                    taffy::GridAutoFlow::default()
                }
            },
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
        OverflowKeyword::Auto => Overflow::Scroll
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
                    }
                    LengthPercentage::Percentage(percentage) => TrackSizingFunction::from_percent(percentage.0),
                    LengthPercentage::Calc(calc) => {
                        error!("calc {calc:?} not supported");
                        TrackSizingFunction::AUTO
                    }
                },
                TrackBreadth::Flex(flex) => TrackSizingFunction::from_flex(*flex),
                TrackBreadth::MinContent => TrackSizingFunction::MIN_CONTENT,
                TrackBreadth::MaxContent => TrackSizingFunction::MAX_CONTENT,
                TrackBreadth::Auto => TrackSizingFunction::AUTO,
            }
            TrackSize::MinMax { .. } => {
                error!("grid min max not supported yet");
                TrackSizingFunction::AUTO
            }
            TrackSize::FitContent(length) => TrackSizingFunction::fit_content(resolve_length(length, context))
        },
        TrackListItem::TrackRepeat(repeat) => {
            error!("grid repeat not supported yet");
            TrackSizingFunction::Repeat(
                match repeat.count {
                    RepeatCount::Number(count) => GridTrackRepetition::Count(count as u16),
                    RepeatCount::AutoFill => GridTrackRepetition::AutoFill,
                    RepeatCount::AutoFit => GridTrackRepetition::AutoFit
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

fn resolve_font_size(size: &FontSize, context: SizeContext) -> f32 {
    match size {
        FontSize::Length(size) => match size {
            LengthPercentage::Dimension(size) => match size {
                LengthValue::Px(value) => *value,
                LengthValue::Em(value) => context.parent_font_size * value,
                LengthValue::Rem(value) => context.root_font_size * value,
                LengthValue::Vw(value) => context.viewport_width * value / 100.0,
                LengthValue::Vh(value) => context.viewport_height * value / 100.0,
                size => {
                    error!("font size {size:?} not supported");
                    context.parent_font_size
                }
            }
            size => {
                error!("font size {size:?} not supported");
                context.parent_font_size
            }
        }
        FontSize::Absolute(size) => match size {
            size => {
                error!("font size {size:?} not supported");
                context.parent_font_size
            },
        }
        FontSize::Relative(size) => match size {
            size => {
                error!("font size {size:?} not supported");
                context.parent_font_size
            }
        }
    }
}


fn resolve_dimension(value: &LengthPercentage, _context: SizeContext) -> Dimension {
    match value {
        LengthPercentage::Dimension(value) => match value {
            LengthValue::Px(px) => Dimension::Length(*px),
            dimension => {
                error!("dimension {dimension:?} not supported");
                Dimension::Length(0.0)
            }
        }
        LengthPercentage::Percentage(percentage) => Dimension::Percent(percentage.0),
        LengthPercentage::Calc(calc) => {
            error!("calc {calc:?} not supported");
            Dimension::Length(0.0)
        }
    }
}

fn resolve_length(value: &LengthPercentage, _context: SizeContext) -> taffy::LengthPercentage {
    match value {
        LengthPercentage::Dimension(length) => match length {
            LengthValue::Px(length) => taffy::LengthPercentage::Length(*length),
            length => {
                error!("length {length:?} not supported");
                taffy::LengthPercentage::Length(0.0)
            }
        }
        LengthPercentage::Percentage(percentage) => taffy::LengthPercentage::Percent(percentage.0),
        LengthPercentage::Calc(calc) => {
            error!("calc {calc:?} not supported");
            taffy::LengthPercentage::Length(0.0)
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
    fn resolve(&self, _context: SizeContext) -> LengthPercentageAuto {
        match self {
            LengthPercentageOrAuto::Auto => LengthPercentageAuto::Auto,
            LengthPercentageOrAuto::LengthPercentage(value) => match value {
                LengthPercentage::Dimension(length) => match length {
                    LengthValue::Px(length) => LengthPercentageAuto::Length(*length),
                    length => {
                        error!("length {length:?} not supported");
                        LengthPercentageAuto::Length(0.0)
                    }
                }
                LengthPercentage::Percentage(percentage) => LengthPercentageAuto::Percent(percentage.0),
                LengthPercentage::Calc(calc) => {
                    error!("calc {calc:?} not supported");
                    LengthPercentageAuto::Length(0.0)
                }
            }
        }
    }
}

impl Resolver<taffy::LengthPercentage> for LengthPercentageOrAuto {
    fn resolve(&self, context: SizeContext) -> taffy::LengthPercentage {
        match self {
            LengthPercentageOrAuto::Auto => taffy::LengthPercentage::Length(0.0),
            LengthPercentageOrAuto::LengthPercentage(value) => resolve_length(value, context)
        }
    }
}

impl Resolver<taffy::LengthPercentage> for BorderSideWidth {
    fn resolve(&self, _context: SizeContext) -> taffy::LengthPercentage {
        match self {
            BorderSideWidth::Thin => taffy::LengthPercentage::Length(1.0),
            BorderSideWidth::Medium => taffy::LengthPercentage::Length(2.0),
            BorderSideWidth::Thick => taffy::LengthPercentage::Length(3.0),
            BorderSideWidth::Length(length) => match length {
                Length::Value(length) => match length {
                    LengthValue::Px(length) => taffy::LengthPercentage::Length(*length),
                    length => {
                        error!("length {length:?} not supported");
                        taffy::LengthPercentage::Length(0.0)
                    }
                }
                Length::Calc(calc) => {
                    error!("calc {calc:?} not supported");
                    taffy::LengthPercentage::Length(0.0)
                }
            }
        }
    }
}

impl Resolver<taffy::LengthPercentage> for GapValue {
    fn resolve(&self, context: SizeContext) -> taffy::LengthPercentage {
        match self {
            GapValue::Normal => taffy::LengthPercentage::Length(0.0),
            GapValue::LengthPercentage(value) => resolve_length(value, context)
        }
    }
}

fn map_grid_line(line: &GridLine) -> GridPlacement {
    match line {
        GridLine::Auto => GridPlacement::Auto,
        GridLine::Area { .. } => {
            error!("grid area placement not supported");
            GridPlacement::Auto
        },
        GridLine::Line { index, .. } => GridPlacement::Line((*index as i16).into()),
        GridLine::Span { index, .. } => GridPlacement::Span(*index as u16)
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
