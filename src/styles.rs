use lightningcss::properties::align::{AlignContent, AlignItems, AlignSelf, BaselinePosition, ContentDistribution, ContentPosition, JustifyContent, JustifyItems, JustifySelf, SelfPosition};
use lightningcss::properties::border::BorderSideWidth;
use lightningcss::properties::display::{Display, DisplayInside, DisplayKeyword, DisplayOutside};
use lightningcss::properties::overflow::OverflowKeyword;
use lightningcss::properties::position::Position;
use lightningcss::properties::Property;
use lightningcss::properties::size::{MaxSize, Size};
use lightningcss::values::length::{Length, LengthPercentage, LengthPercentageOrAuto, LengthValue};
use lightningcss::values::ratio::Ratio;
use log::error;
use taffy::{Dimension, LengthPercentageAuto, Overflow, Style};
use taffy::prelude::length;
use crate::{Rectangle, Ruleset, SizeContext};

pub fn apply_rectangle_rules<'i>(ruleset: &Ruleset<'i>, rectangle: &mut Rectangle<'i>) {
    for property in &ruleset.style.declarations.declarations {
        match property {
            Property::BackgroundColor(_) => {}
            Property::BackgroundImage(_) => {}
            Property::BackgroundPositionX(_) => {}
            Property::BackgroundPositionY(_) => {}
            Property::BackgroundPosition(_) => {}
            Property::BackgroundSize(_) => {}
            Property::BackgroundRepeat(_) => {}
            Property::BackgroundAttachment(_) => {}
            Property::BackgroundClip(_, _) => {}
            Property::BackgroundOrigin(_) => {}
            Property::Background(background) => {
                rectangle.background = background[0].clone()
            }
            Property::BoxShadow(_, _) => {}
            Property::Opacity(_) => {}
            Property::Color(_) => {}
            Property::Display(_) => {}
            Property::Visibility(_) => {}
            Property::Width(_) => {}
            Property::Height(_) => {}
            Property::MinWidth(_) => {}
            Property::MinHeight(_) => {}
            Property::MaxWidth(_) => {}
            Property::MaxHeight(_) => {}
            Property::BlockSize(_) => {}
            Property::InlineSize(_) => {}
            Property::MinBlockSize(_) => {}
            Property::MinInlineSize(_) => {}
            Property::MaxBlockSize(_) => {}
            Property::MaxInlineSize(_) => {}
            Property::BoxSizing(_, _) => {}
            Property::AspectRatio(_) => {}
            Property::Overflow(_) => {}
            Property::OverflowX(_) => {}
            Property::OverflowY(_) => {}
            Property::TextOverflow(_, _) => {}
            Property::Position(_) => {}
            Property::Top(_) => {}
            Property::Bottom(_) => {}
            Property::Left(_) => {}
            Property::Right(_) => {}
            Property::InsetBlockStart(_) => {}
            Property::InsetBlockEnd(_) => {}
            Property::InsetInlineStart(_) => {}
            Property::InsetInlineEnd(_) => {}
            Property::InsetBlock(_) => {}
            Property::InsetInline(_) => {}
            Property::Inset(_) => {}
            Property::BorderSpacing(_) => {}
            Property::BorderTopColor(_) => {}
            Property::BorderBottomColor(_) => {}
            Property::BorderLeftColor(_) => {}
            Property::BorderRightColor(_) => {}
            Property::BorderBlockStartColor(_) => {}
            Property::BorderBlockEndColor(_) => {}
            Property::BorderInlineStartColor(_) => {}
            Property::BorderInlineEndColor(_) => {}
            Property::BorderTopStyle(_) => {}
            Property::BorderBottomStyle(_) => {}
            Property::BorderLeftStyle(_) => {}
            Property::BorderRightStyle(_) => {}
            Property::BorderBlockStartStyle(_) => {}
            Property::BorderBlockEndStyle(_) => {}
            Property::BorderInlineStartStyle(_) => {}
            Property::BorderInlineEndStyle(_) => {}
            Property::BorderTopWidth(_) => {}
            Property::BorderBottomWidth(_) => {}
            Property::BorderLeftWidth(_) => {}
            Property::BorderRightWidth(_) => {}
            Property::BorderBlockStartWidth(_) => {}
            Property::BorderBlockEndWidth(_) => {}
            Property::BorderInlineStartWidth(_) => {}
            Property::BorderInlineEndWidth(_) => {}
            Property::BorderTopLeftRadius(_, _) => {}
            Property::BorderTopRightRadius(_, _) => {}
            Property::BorderBottomLeftRadius(_, _) => {}
            Property::BorderBottomRightRadius(_, _) => {}
            Property::BorderStartStartRadius(_) => {}
            Property::BorderStartEndRadius(_) => {}
            Property::BorderEndStartRadius(_) => {}
            Property::BorderEndEndRadius(_) => {}
            Property::BorderRadius(_, _) => {}
            Property::BorderImageSource(_) => {}
            Property::BorderImageOutset(_) => {}
            Property::BorderImageRepeat(_) => {}
            Property::BorderImageWidth(_) => {}
            Property::BorderImageSlice(_) => {}
            Property::BorderImage(_, _) => {}
            Property::BorderColor(_) => {}
            Property::BorderStyle(_) => {}
            Property::BorderWidth(_) => {}
            Property::BorderBlockColor(_) => {}
            Property::BorderBlockStyle(_) => {}
            Property::BorderBlockWidth(_) => {}
            Property::BorderInlineColor(_) => {}
            Property::BorderInlineStyle(_) => {}
            Property::BorderInlineWidth(_) => {}
            Property::Border(_) => {}
            Property::BorderTop(_) => {}
            Property::BorderBottom(_) => {}
            Property::BorderLeft(_) => {}
            Property::BorderRight(_) => {}
            Property::BorderBlock(_) => {}
            Property::BorderBlockStart(_) => {}
            Property::BorderBlockEnd(_) => {}
            Property::BorderInline(_) => {}
            Property::BorderInlineStart(_) => {}
            Property::BorderInlineEnd(_) => {}
            Property::Outline(_) => {}
            Property::OutlineColor(_) => {}
            Property::OutlineStyle(_) => {}
            Property::OutlineWidth(_) => {}
            Property::FlexDirection(_, _) => {}
            Property::FlexWrap(_, _) => {}
            Property::FlexFlow(_, _) => {}
            Property::FlexGrow(_, _) => {}
            Property::FlexShrink(_, _) => {}
            Property::FlexBasis(_, _) => {}
            Property::Flex(_, _) => {}
            Property::Order(_, _) => {}
            Property::AlignContent(_, _) => {}
            Property::JustifyContent(_, _) => {}
            Property::PlaceContent(_) => {}
            Property::AlignSelf(_, _) => {}
            Property::JustifySelf(_) => {}
            Property::PlaceSelf(_) => {}
            Property::AlignItems(_, _) => {}
            Property::JustifyItems(_) => {}
            Property::PlaceItems(_) => {}
            Property::RowGap(_) => {}
            Property::ColumnGap(_) => {}
            Property::Gap(_) => {}
            Property::BoxOrient(_, _) => {}
            Property::BoxDirection(_, _) => {}
            Property::BoxOrdinalGroup(_, _) => {}
            Property::BoxAlign(_, _) => {}
            Property::BoxFlex(_, _) => {}
            Property::BoxFlexGroup(_, _) => {}
            Property::BoxPack(_, _) => {}
            Property::BoxLines(_, _) => {}
            Property::FlexPack(_, _) => {}
            Property::FlexOrder(_, _) => {}
            Property::FlexAlign(_, _) => {}
            Property::FlexItemAlign(_, _) => {}
            Property::FlexLinePack(_, _) => {}
            Property::FlexPositive(_, _) => {}
            Property::FlexNegative(_, _) => {}
            Property::FlexPreferredSize(_, _) => {}
            Property::GridTemplateColumns(_) => {}
            Property::GridTemplateRows(_) => {}
            Property::GridAutoColumns(_) => {}
            Property::GridAutoRows(_) => {}
            Property::GridAutoFlow(_) => {}
            Property::GridTemplateAreas(_) => {}
            Property::GridTemplate(_) => {}
            Property::Grid(_) => {}
            Property::GridRowStart(_) => {}
            Property::GridRowEnd(_) => {}
            Property::GridColumnStart(_) => {}
            Property::GridColumnEnd(_) => {}
            Property::GridRow(_) => {}
            Property::GridColumn(_) => {}
            Property::GridArea(_) => {}
            Property::MarginTop(_) => {}
            Property::MarginBottom(_) => {}
            Property::MarginLeft(_) => {}
            Property::MarginRight(_) => {}
            Property::MarginBlockStart(_) => {}
            Property::MarginBlockEnd(_) => {}
            Property::MarginInlineStart(_) => {}
            Property::MarginInlineEnd(_) => {}
            Property::MarginBlock(_) => {}
            Property::MarginInline(_) => {}
            Property::Margin(_) => {}
            Property::PaddingTop(_) => {}
            Property::PaddingBottom(_) => {}
            Property::PaddingLeft(_) => {}
            Property::PaddingRight(_) => {}
            Property::PaddingBlockStart(_) => {}
            Property::PaddingBlockEnd(_) => {}
            Property::PaddingInlineStart(_) => {}
            Property::PaddingInlineEnd(_) => {}
            Property::PaddingBlock(_) => {}
            Property::PaddingInline(_) => {}
            Property::Padding(_) => {}
            Property::ScrollMarginTop(_) => {}
            Property::ScrollMarginBottom(_) => {}
            Property::ScrollMarginLeft(_) => {}
            Property::ScrollMarginRight(_) => {}
            Property::ScrollMarginBlockStart(_) => {}
            Property::ScrollMarginBlockEnd(_) => {}
            Property::ScrollMarginInlineStart(_) => {}
            Property::ScrollMarginInlineEnd(_) => {}
            Property::ScrollMarginBlock(_) => {}
            Property::ScrollMarginInline(_) => {}
            Property::ScrollMargin(_) => {}
            Property::ScrollPaddingTop(_) => {}
            Property::ScrollPaddingBottom(_) => {}
            Property::ScrollPaddingLeft(_) => {}
            Property::ScrollPaddingRight(_) => {}
            Property::ScrollPaddingBlockStart(_) => {}
            Property::ScrollPaddingBlockEnd(_) => {}
            Property::ScrollPaddingInlineStart(_) => {}
            Property::ScrollPaddingInlineEnd(_) => {}
            Property::ScrollPaddingBlock(_) => {}
            Property::ScrollPaddingInline(_) => {}
            Property::ScrollPadding(_) => {}
            Property::FontWeight(_) => {}
            Property::FontSize(_) => {}
            Property::FontStretch(_) => {}
            Property::FontFamily(_) => {}
            Property::FontStyle(_) => {}
            Property::FontVariantCaps(_) => {}
            Property::LineHeight(_) => {}
            Property::Font(_) => {}
            Property::VerticalAlign(_) => {}
            Property::FontPalette(_) => {}
            Property::TransitionProperty(_, _) => {}
            Property::TransitionDuration(_, _) => {}
            Property::TransitionDelay(_, _) => {}
            Property::TransitionTimingFunction(_, _) => {}
            Property::Transition(_, _) => {}
            Property::AnimationName(_, _) => {}
            Property::AnimationDuration(_, _) => {}
            Property::AnimationTimingFunction(_, _) => {}
            Property::AnimationIterationCount(_, _) => {}
            Property::AnimationDirection(_, _) => {}
            Property::AnimationPlayState(_, _) => {}
            Property::AnimationDelay(_, _) => {}
            Property::AnimationFillMode(_, _) => {}
            Property::AnimationComposition(_) => {}
            Property::AnimationTimeline(_) => {}
            Property::Animation(_, _) => {}
            Property::Transform(_, _) => {}
            Property::TransformOrigin(_, _) => {}
            Property::TransformStyle(_, _) => {}
            Property::TransformBox(_) => {}
            Property::BackfaceVisibility(_, _) => {}
            Property::Perspective(_, _) => {}
            Property::PerspectiveOrigin(_, _) => {}
            Property::Translate(_) => {}
            Property::Rotate(_) => {}
            Property::Scale(_) => {}
            Property::TextTransform(_) => {}
            Property::WhiteSpace(_) => {}
            Property::TabSize(_, _) => {}
            Property::WordBreak(_) => {}
            Property::LineBreak(_) => {}
            Property::Hyphens(_, _) => {}
            Property::OverflowWrap(_) => {}
            Property::WordWrap(_) => {}
            Property::TextAlign(_) => {}
            Property::TextAlignLast(_, _) => {}
            Property::TextJustify(_) => {}
            Property::WordSpacing(_) => {}
            Property::LetterSpacing(_) => {}
            Property::TextIndent(_) => {}
            Property::TextDecorationLine(_, _) => {}
            Property::TextDecorationStyle(_, _) => {}
            Property::TextDecorationColor(_, _) => {}
            Property::TextDecorationThickness(_) => {}
            Property::TextDecoration(_, _) => {}
            Property::TextDecorationSkipInk(_, _) => {}
            Property::TextEmphasisStyle(_, _) => {}
            Property::TextEmphasisColor(_, _) => {}
            Property::TextEmphasis(_, _) => {}
            Property::TextEmphasisPosition(_, _) => {}
            Property::TextShadow(_) => {}
            Property::TextSizeAdjust(_, _) => {}
            Property::Direction(_) => {}
            Property::UnicodeBidi(_) => {}
            Property::BoxDecorationBreak(_, _) => {}
            Property::Resize(_) => {}
            Property::Cursor(_) => {}
            Property::CaretColor(_) => {}
            Property::CaretShape(_) => {}
            Property::Caret(_) => {}
            Property::UserSelect(_, _) => {}
            Property::AccentColor(_) => {}
            Property::Appearance(_, _) => {}
            Property::ListStyleType(_) => {}
            Property::ListStyleImage(_) => {}
            Property::ListStylePosition(_) => {}
            Property::ListStyle(_) => {}
            Property::MarkerSide(_) => {}
            Property::Composes(_) => {}
            Property::Fill(_) => {}
            Property::FillRule(_) => {}
            Property::FillOpacity(_) => {}
            Property::Stroke(_) => {}
            Property::StrokeOpacity(_) => {}
            Property::StrokeWidth(_) => {}
            Property::StrokeLinecap(_) => {}
            Property::StrokeLinejoin(_) => {}
            Property::StrokeMiterlimit(_) => {}
            Property::StrokeDasharray(_) => {}
            Property::StrokeDashoffset(_) => {}
            Property::MarkerStart(_) => {}
            Property::MarkerMid(_) => {}
            Property::MarkerEnd(_) => {}
            Property::Marker(_) => {}
            Property::ColorInterpolation(_) => {}
            Property::ColorInterpolationFilters(_) => {}
            Property::ColorRendering(_) => {}
            Property::ShapeRendering(_) => {}
            Property::TextRendering(_) => {}
            Property::ImageRendering(_) => {}
            Property::ClipPath(_, _) => {}
            Property::ClipRule(_) => {}
            Property::MaskImage(_, _) => {}
            Property::MaskMode(_) => {}
            Property::MaskRepeat(_, _) => {}
            Property::MaskPositionX(_) => {}
            Property::MaskPositionY(_) => {}
            Property::MaskPosition(_, _) => {}
            Property::MaskClip(_, _) => {}
            Property::MaskOrigin(_, _) => {}
            Property::MaskSize(_, _) => {}
            Property::MaskComposite(_) => {}
            Property::MaskType(_) => {}
            Property::Mask(_, _) => {}
            Property::MaskBorderSource(_) => {}
            Property::MaskBorderMode(_) => {}
            Property::MaskBorderSlice(_) => {}
            Property::MaskBorderWidth(_) => {}
            Property::MaskBorderOutset(_) => {}
            Property::MaskBorderRepeat(_) => {}
            Property::MaskBorder(_) => {}
            Property::WebKitMaskComposite(_) => {}
            Property::WebKitMaskSourceType(_, _) => {}
            Property::WebKitMaskBoxImage(_, _) => {}
            Property::WebKitMaskBoxImageSource(_, _) => {}
            Property::WebKitMaskBoxImageSlice(_, _) => {}
            Property::WebKitMaskBoxImageWidth(_, _) => {}
            Property::WebKitMaskBoxImageOutset(_, _) => {}
            Property::WebKitMaskBoxImageRepeat(_, _) => {}
            Property::Filter(_, _) => {}
            Property::BackdropFilter(_, _) => {}
            Property::ZIndex(_) => {}
            Property::ContainerType(_) => {}
            Property::ContainerName(_) => {}
            Property::Container(_) => {}
            Property::ViewTransitionName(_) => {}
            Property::ColorScheme(_) => {}
            Property::All(_) => {}
            Property::Unparsed(_) => {}
            Property::Custom(_) => {}
        }
    }
}

pub fn apply_style_rules(ruleset: &Ruleset, style: &mut Style, context: SizeContext) {
    // let def = Style {
    //     display: Default::default(),
    //     overflow: Default::default(),
    //     scrollbar_width: 0.0,
    //     position: Default::default(),
    //     inset: Rect {},
    //     size: Size {},
    //     min_size: Size {},
    //     max_size: Size {},
    //     aspect_ratio: None,
    //     margin: Rect {},
    //     padding: Rect {},
    //     border: Rect {},
    //     align_items: None,
    //     align_self: None,
    //     justify_items: None,
    //     justify_self: None,
    //     align_content: None,
    //     justify_content: None,
    //     gap: Size {},
    //     flex_direction: Default::default(),
    //     flex_wrap: Default::default(),
    //     flex_basis: Dimension::Auto,
    //     flex_grow: 0.0,
    //     flex_shrink: 0.0,
    //     grid_template_rows: vec![],
    //     grid_template_columns: vec![],
    //     grid_auto_rows: vec![],
    //     grid_auto_columns: vec![],
    //     grid_auto_flow: Default::default(),
    //     grid_row: Default::default(),
    //     grid_column: Default::default(),
    // }

    for property in &ruleset.style.declarations.declarations {
        println!("PROP {property:?}");
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
                align => error!("align {align:?} not supported")
            },
            Property::AlignSelf(align, _) => match align {
                AlignSelf::Auto => style.align_self = None,
                AlignSelf::Normal => style.align_self = None,
                AlignSelf::Stretch => style.align_self = Some(taffy::AlignSelf::Stretch),
                AlignSelf::BaselinePosition(_) => style.align_self = Some(taffy::AlignSelf::Baseline),
                AlignSelf::SelfPosition { value, .. } => style.align_self = map_self_position(value),
                align => error!("align {align:?} not supported")
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
            _ => {}
        }
    }
}

fn map_overflow(keyword: OverflowKeyword) -> Overflow {
    match keyword {
        OverflowKeyword::Visible =>  Overflow::Visible,
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


trait Resolver<T> {
    fn resolve(&self, context: SizeContext) -> T;
}

impl Resolver<Dimension> for MaxSize {
    fn resolve(&self, _context: SizeContext) -> Dimension {
        match self {
            MaxSize::None => Dimension::Auto,
            MaxSize::LengthPercentage(value) => match value {
                LengthPercentage::Dimension(value) => match value {
                    LengthValue::Px(px) => Dimension::Length(*px),
                    dimension => {
                        error!("max-size dimension {dimension:?} not supported");
                        Dimension::Length(0.0)
                    }
                }
                LengthPercentage::Percentage(percentage) => Dimension::Percent(percentage.0),
                LengthPercentage::Calc(calc) => {
                    error!("max-size calc {calc:?} not supported");
                    Dimension::Length(0.0)
                }
            }
            dimension => {
                error!("max-size {dimension:?} not supported");
                Dimension::Length(0.0)
            }
        }
    }
}

impl Resolver<Dimension> for Size {
    fn resolve(&self, _context: SizeContext) -> Dimension {
        match self {
            Size::Auto => Dimension::Auto,
            Size::LengthPercentage(value) => match value {
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
            dimension => {
                error!("dimension {dimension:?} not supported");
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
    fn resolve(&self, _context: SizeContext) -> taffy::LengthPercentage {
        match self {
            LengthPercentageOrAuto::Auto => taffy::LengthPercentage::Length(0.0),
            LengthPercentageOrAuto::LengthPercentage(value) => match value {
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
