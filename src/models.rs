use lightningcss::rules::style::StyleRule;
use scraper::Selector;

#[derive(Clone, Copy)]
pub struct SizeContext {
    pub root_font_size: f32,
    pub parent_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId {
    pub element_n: usize,
    pub hash: u64,
}

pub struct Ruleset<'i> {
    pub selector: Selector,
    pub style: StyleRule<'i>,
}

pub struct Presentation {
    pub rules: Vec<Ruleset<'static>>,
}
