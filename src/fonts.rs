use crate::FontFace;

pub trait Fonts {
    fn measure(&self, text: &str, face: &FontFace, max_width: Option<f32>) -> [f32; 2];
}

pub(crate) struct DummyFonts;

impl Fonts for DummyFonts {
    fn measure(&self, text: &str, face: &FontFace, max_width: Option<f32>) -> [f32; 2] {
        // NOTE: incorrect implementation, approximately calculates the text size
        // you should provide your own Fonts implementation
        let width = text.len() as f32 * face.size * 0.75;
        match max_width {
            None => [width, face.size],
            Some(max_width) => {
                if max_width == 0.0 {
                    [0.0, 0.0]
                } else {
                    let lines = 1.0 + (width / max_width).floor();
                    [max_width, lines * face.size]
                }
            }
        }
    }
}
