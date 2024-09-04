use crate::rendering::Renderer;
use crate::{Element, View, ViewError};
use taffy::NodeId;

const BACKGROUND: usize = 0;

impl Renderer {
    pub(crate) fn render_img(&mut self, img: &mut Element) -> Result<[NodeId; 1], ViewError> {
        let undefined = String::new();
        let src = img.attrs.get("src").unwrap_or(&undefined);
        let background = self.render_bg_image(src.clone())?;
        Ok([background])
    }
}

impl View {
    pub(crate) fn update_img_view(&mut self, img: NodeId, src: String) -> Result<(), ViewError> {
        let background = self.tree.child_at_index(img, BACKGROUND)?;
        let background = self
            .tree
            .get_node_context_mut(background)
            .expect("img background has element");
        background.background.image = Some(src);
        let node = background.node;
        self.tree.mark_dirty(node)?;
        Ok(())
    }
}
