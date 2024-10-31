use crate::Element;

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
    // font-styles
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
    // list-styles-image
    // list-styles-position
    // list-styles-type
    // list-styles
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
