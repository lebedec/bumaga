use crate::rendering::Renderer;
use crate::{Element, ViewError, ViewModel};
use taffy::{NodeId, TaffyTree};

const BACKGROUND: usize = 0;

impl Renderer {
    pub(crate) fn render_img(&mut self, img: &mut Element) -> Result<[NodeId; 1], ViewError> {
        let undefined = String::new();
        let src = img.attrs.get("src").unwrap_or(&undefined);
        let background = self.render_bg_image(src.clone())?;
        Ok([background])
    }
}

impl ViewModel {
    pub(crate) fn update_img_src(
        &mut self,
        img: NodeId,
        src: String,
        tree: &mut TaffyTree<Element>,
    ) -> Result<(), ViewError> {
        let background = tree.child_at_index(img, BACKGROUND)?;
        let background = tree
            .get_node_context_mut(background)
            .expect("img background has element");
        background.background.image = Some(src);
        let node = background.node;
        tree.mark_dirty(node)?;
        Ok(())
    }
}
