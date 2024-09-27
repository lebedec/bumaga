use crate::animation::{
    AnimationDirection, AnimationFillMode, AnimationIterations, TimingFunction,
};
use crate::css::ComputedValue::{Keyword, Time};
use crate::css::{ComputedValue, Dim, PropertyKey, Units};
use crate::styles::{Cascade, CascadeError};
use crate::{Element, Length, PointerEvents, TextAlign, TransformFunction};
use log::{debug, error};
use taffy::{Dimension, LengthPercentage, LengthPercentageAuto, Overflow};

impl<'c> Cascade<'c> {
    pub(crate) fn apply(
        &mut self,
        key: PropertyKey,
        index: usize,
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
            (PropertyKey::BoxSizing, _) => {
                // TODO: calculate size manually?
                debug!("ignore box-sizing property, not supported");
            }
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
            // Transition
            //
            (PropertyKey::TransitionProperty, Keyword(name)) => {
                let key = match PropertyKey::parse(name) {
                    Some(key) => key,
                    None => return CascadeError::invalid_keyword(name),
                };
                element.get_transition_mut(index).key = Some(key);
            }
            (PropertyKey::TransitionDuration, Time(duration)) => {
                element.get_transition_mut(index).animator.duration = *duration;
            }
            (PropertyKey::TransitionDelay, Time(delay)) => {
                element.get_transition_mut(index).animator.delay = *delay;
            }
            (PropertyKey::TransitionTimingFunction, timing) => {
                let timing = resolve_timing(timing, self)?;
                element.get_transition_mut(index).animator.timing = timing;
            }
            //
            // Animation
            //
            (PropertyKey::AnimationName, Keyword(name)) => {
                element.get_animator_mut(index).name = name.to_string();
            }
            (PropertyKey::AnimationDelay, Time(delay)) => {
                element.get_animator_mut(index).delay = *delay;
            }
            (PropertyKey::AnimationDirection, Keyword(keyword)) => {
                element.get_animator_mut(index).direction = match keyword.as_str() {
                    "normal" => AnimationDirection::Normal,
                    "reverse" => AnimationDirection::Reverse,
                    "alternate" => AnimationDirection::Alternate,
                    "alternate-reverse" => AnimationDirection::AlternateReverse,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationDuration, Time(duration)) => {
                element.get_animator_mut(index).duration = *duration;
            }
            (PropertyKey::AnimationFillMode, Keyword(keyword)) => {
                element.get_animator_mut(index).fill_mode = match keyword.as_str() {
                    "none" => AnimationFillMode::None,
                    "forwards" => AnimationFillMode::Forwards,
                    "backwards" => AnimationFillMode::Backwards,
                    "both" => AnimationFillMode::Both,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationIterationCount, iterations) => {
                element.get_animator_mut(index).iterations = resolve_iterations(iterations, self)?;
            }
            (PropertyKey::AnimationPlayState, Keyword(keyword)) => {
                element.get_animator_mut(index).running = match keyword.as_str() {
                    "running" => true,
                    "paused" => false,
                    keyword => return CascadeError::invalid_keyword(keyword),
                }
            }
            (PropertyKey::AnimationTimingFunction, timing) => {
                element.get_animator_mut(index).timing = resolve_timing(timing, self)?
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
        ComputedValue::Zero => Dimension::Length(0.0),
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
