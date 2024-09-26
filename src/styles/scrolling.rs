use taffy::Layout;

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
