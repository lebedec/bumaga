use crate::css::ComputedValue::{Keyword, Time};
use crate::css::{ComputedStyle, ComputedValue, Definition, PropertyDescriptor, PropertyKey};
use crate::styles::Cascade;
use log::error;

impl<'c> Cascade<'c> {
    pub(crate) fn compute_style(
        &self,
        key: PropertyKey,
        index: usize,
        definition: &[Definition],
        style: &mut ComputedStyle,
    ) {
        let mut shorthand = vec![];
        if !self.compute_shorthand(definition, &mut shorthand) {
            return;
        }
        let mut overwrite = |key: PropertyKey, value: &ComputedValue| {
            style.insert(PropertyDescriptor::new(key, index), value.clone())
        };
        match (key, shorthand.as_slice()) {
            //
            // Element
            //
            (PropertyKey::Background, [color]) => {
                overwrite(PropertyKey::BackgroundColor, color);
            }
            (PropertyKey::BackgroundPosition, [value]) => {
                overwrite(PropertyKey::BackgroundPositionX, value);
                overwrite(PropertyKey::BackgroundPositionY, value);
            }
            (PropertyKey::BackgroundPosition, [x, y]) => {
                overwrite(PropertyKey::BackgroundPositionX, x);
                overwrite(PropertyKey::BackgroundPositionY, y);
            }
            //
            // Element + Layout
            //
            (PropertyKey::Border, [width, _style, color]) => {
                overwrite(PropertyKey::BorderTopWidth, width);
                overwrite(PropertyKey::BorderTopColor, color);
                overwrite(PropertyKey::BorderRightWidth, width);
                overwrite(PropertyKey::BorderRightColor, color);
                overwrite(PropertyKey::BorderBottomWidth, width);
                overwrite(PropertyKey::BorderBottomColor, color);
                overwrite(PropertyKey::BorderLeftWidth, width);
                overwrite(PropertyKey::BorderLeftColor, color);
            }
            (PropertyKey::BorderTop, [width, _style, color]) => {
                overwrite(PropertyKey::BorderTopWidth, width);
                overwrite(PropertyKey::BorderTopColor, color);
            }
            (PropertyKey::BorderRight, [width, _style, color]) => {
                overwrite(PropertyKey::BorderRightWidth, width);
                overwrite(PropertyKey::BorderRightColor, color);
            }
            (PropertyKey::BorderBottom, [width, _style, color]) => {
                overwrite(PropertyKey::BorderBottomWidth, width);
                overwrite(PropertyKey::BorderBottomColor, color);
            }
            (PropertyKey::BorderLeft, [width, _style, color]) => {
                overwrite(PropertyKey::BorderLeftWidth, width);
                overwrite(PropertyKey::BorderLeftColor, color);
            }
            (PropertyKey::BorderWidth, [top, right, bottom, left]) => {
                overwrite(PropertyKey::BorderTopWidth, top);
                overwrite(PropertyKey::BorderRightWidth, right);
                overwrite(PropertyKey::BorderBottomWidth, bottom);
                overwrite(PropertyKey::BorderLeftWidth, left);
            }
            (PropertyKey::BorderWidth, [top, h, bottom]) => {
                overwrite(PropertyKey::BorderTopWidth, top);
                overwrite(PropertyKey::BorderRightWidth, h);
                overwrite(PropertyKey::BorderBottomWidth, bottom);
                overwrite(PropertyKey::BorderLeftWidth, h);
            }
            (PropertyKey::BorderWidth, [v, h]) => {
                overwrite(PropertyKey::BorderTopWidth, v);
                overwrite(PropertyKey::BorderRightWidth, h);
                overwrite(PropertyKey::BorderBottomWidth, v);
                overwrite(PropertyKey::BorderLeftWidth, h);
            }
            (PropertyKey::BorderWidth, [value]) => {
                overwrite(PropertyKey::BorderTopWidth, value);
                overwrite(PropertyKey::BorderRightWidth, value);
                overwrite(PropertyKey::BorderBottomWidth, value);
                overwrite(PropertyKey::BorderLeftWidth, value);
            }
            (PropertyKey::BorderColor, [top, right, bottom, left]) => {
                overwrite(PropertyKey::BorderTopColor, top);
                overwrite(PropertyKey::BorderRightColor, right);
                overwrite(PropertyKey::BorderBottomColor, bottom);
                overwrite(PropertyKey::BorderLeftColor, left);
            }
            (PropertyKey::BorderColor, [top, h, bottom]) => {
                overwrite(PropertyKey::BorderTopColor, top);
                overwrite(PropertyKey::BorderRightColor, h);
                overwrite(PropertyKey::BorderBottomColor, bottom);
                overwrite(PropertyKey::BorderLeftColor, h);
            }
            (PropertyKey::BorderColor, [v, h]) => {
                overwrite(PropertyKey::BorderTopColor, v);
                overwrite(PropertyKey::BorderRightColor, h);
                overwrite(PropertyKey::BorderBottomColor, v);
                overwrite(PropertyKey::BorderLeftColor, h);
            }
            (PropertyKey::BorderColor, [value]) => {
                overwrite(PropertyKey::BorderTopColor, value);
                overwrite(PropertyKey::BorderRightColor, value);
                overwrite(PropertyKey::BorderBottomColor, value);
                overwrite(PropertyKey::BorderLeftColor, value);
            }
            (PropertyKey::BorderRadius, [a, b, c, d]) => {
                overwrite(PropertyKey::BorderTopLeftRadius, a);
                overwrite(PropertyKey::BorderTopRightRadius, b);
                overwrite(PropertyKey::BorderBottomRightRadius, c);
                overwrite(PropertyKey::BorderBottomLeftRadius, d);
            }
            (PropertyKey::BorderRadius, [a, b, c]) => {
                overwrite(PropertyKey::BorderTopLeftRadius, a);
                overwrite(PropertyKey::BorderTopRightRadius, b);
                overwrite(PropertyKey::BorderBottomRightRadius, c);
                overwrite(PropertyKey::BorderBottomLeftRadius, b);
            }
            (PropertyKey::BorderRadius, [a, b]) => {
                overwrite(PropertyKey::BorderTopLeftRadius, a);
                overwrite(PropertyKey::BorderTopRightRadius, b);
                overwrite(PropertyKey::BorderBottomRightRadius, a);
                overwrite(PropertyKey::BorderBottomLeftRadius, b);
            }
            (PropertyKey::BorderRadius, [value]) => {
                overwrite(PropertyKey::BorderTopLeftRadius, value);
                overwrite(PropertyKey::BorderTopRightRadius, value);
                overwrite(PropertyKey::BorderBottomRightRadius, value);
                overwrite(PropertyKey::BorderBottomLeftRadius, value);
            }
            //
            // Layout
            //
            (PropertyKey::Overflow, [value]) => {
                overwrite(PropertyKey::OverflowX, value);
                overwrite(PropertyKey::OverflowY, value);
            }
            (PropertyKey::Overflow, [x, y]) => {
                overwrite(PropertyKey::OverflowX, x);
                overwrite(PropertyKey::OverflowY, y);
            }
            (PropertyKey::Inset, [top, right, bottom, left]) => {
                overwrite(PropertyKey::Top, top);
                overwrite(PropertyKey::Right, right);
                overwrite(PropertyKey::Bottom, bottom);
                overwrite(PropertyKey::Left, left);
            }
            (PropertyKey::Inset, [value]) => {
                overwrite(PropertyKey::Top, value);
                overwrite(PropertyKey::Right, value);
                overwrite(PropertyKey::Bottom, value);
                overwrite(PropertyKey::Left, value);
            }
            (PropertyKey::Gap, [column, row]) => {
                overwrite(PropertyKey::RowGap, row);
                overwrite(PropertyKey::ColumnGap, column);
            }
            (PropertyKey::Gap, [value]) => {
                overwrite(PropertyKey::RowGap, value);
                overwrite(PropertyKey::ColumnGap, value);
            }
            (PropertyKey::Padding, [top, right, bottom, left]) => {
                overwrite(PropertyKey::PaddingTop, top);
                overwrite(PropertyKey::PaddingRight, right);
                overwrite(PropertyKey::PaddingBottom, bottom);
                overwrite(PropertyKey::PaddingLeft, left);
            }
            (PropertyKey::Padding, [top, h, bottom]) => {
                overwrite(PropertyKey::PaddingTop, top);
                overwrite(PropertyKey::PaddingRight, h);
                overwrite(PropertyKey::PaddingBottom, bottom);
                overwrite(PropertyKey::PaddingLeft, h);
            }
            (PropertyKey::Padding, [v, h]) => {
                overwrite(PropertyKey::PaddingTop, v);
                overwrite(PropertyKey::PaddingRight, h);
                overwrite(PropertyKey::PaddingBottom, v);
                overwrite(PropertyKey::PaddingLeft, h);
            }
            (PropertyKey::Padding, [value]) => {
                overwrite(PropertyKey::PaddingTop, value);
                overwrite(PropertyKey::PaddingRight, value);
                overwrite(PropertyKey::PaddingBottom, value);
                overwrite(PropertyKey::PaddingLeft, value);
            }
            (PropertyKey::Margin, [top, right, bottom, left]) => {
                overwrite(PropertyKey::MarginTop, top);
                overwrite(PropertyKey::MarginRight, right);
                overwrite(PropertyKey::MarginBottom, bottom);
                overwrite(PropertyKey::MarginLeft, left);
            }
            (PropertyKey::Margin, [top, h, bottom]) => {
                overwrite(PropertyKey::MarginTop, top);
                overwrite(PropertyKey::MarginRight, h);
                overwrite(PropertyKey::MarginBottom, bottom);
                overwrite(PropertyKey::MarginLeft, h);
            }
            (PropertyKey::Margin, [v, h]) => {
                overwrite(PropertyKey::MarginTop, v);
                overwrite(PropertyKey::MarginRight, h);
                overwrite(PropertyKey::MarginBottom, v);
                overwrite(PropertyKey::MarginLeft, h);
            }
            (PropertyKey::Margin, [value]) => {
                overwrite(PropertyKey::MarginTop, value);
                overwrite(PropertyKey::MarginRight, value);
                overwrite(PropertyKey::MarginBottom, value);
                overwrite(PropertyKey::MarginLeft, value);
            }
            //
            // Transform
            //
            // (PropertyKey::Transform, shorthand) => {
            //     element.transforms = resolve_transforms(shorthand, self)?;
            // }
            //
            // Transition
            //
            (PropertyKey::Transition, [property, duration]) => {
                overwrite(PropertyKey::TransitionProperty, property);
                overwrite(PropertyKey::TransitionDuration, duration);
            }
            (PropertyKey::Transition, [property, duration, timing]) => {
                overwrite(PropertyKey::TransitionProperty, property);
                overwrite(PropertyKey::TransitionDuration, duration);
                overwrite(PropertyKey::TransitionTimingFunction, timing);
            }
            (PropertyKey::Transition, [property, duration, timing, delay]) => {
                overwrite(PropertyKey::TransitionProperty, property);
                overwrite(PropertyKey::TransitionDuration, duration);
                overwrite(PropertyKey::TransitionTimingFunction, timing);
                overwrite(PropertyKey::TransitionDelay, delay);
            }
            //
            // Animation
            //
            // There is no static shorthand pattern, we should set values by it type and order
            // TODO: special animation shorthand parser
            (PropertyKey::Animation, [duration, timing, Time(delay), name]) => {
                overwrite(PropertyKey::AnimationDelay, &Time(*delay));
                overwrite(PropertyKey::AnimationDuration, duration);
                overwrite(PropertyKey::AnimationName, name);
                overwrite(PropertyKey::AnimationTimingFunction, timing);
            }
            (PropertyKey::Animation, [duration, timing, iterations, name]) => {
                overwrite(PropertyKey::AnimationDuration, duration);
                overwrite(PropertyKey::AnimationIterationCount, iterations);
                overwrite(PropertyKey::AnimationName, name);
                overwrite(PropertyKey::AnimationTimingFunction, timing);
            }
            (PropertyKey::Animation, [duration, timing, name]) => {
                overwrite(PropertyKey::AnimationDuration, duration);
                overwrite(PropertyKey::AnimationName, name);
                overwrite(PropertyKey::AnimationTimingFunction, timing);
            }
            (PropertyKey::Animation, [duration, name]) => {
                overwrite(PropertyKey::AnimationDuration, duration);
                overwrite(PropertyKey::AnimationName, name);
            }
            (key, [value]) => {
                overwrite(key, value);
            }
            (key, value) => {
                error!("unable to compute styles, property {key:?}: {value:?} not supported")
            }
        }
    }
}
