use crate::rendering::Renderer;
use crate::tree::ViewTreeExtensions;
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
        let child_node = tree.child_at_index(img, BACKGROUND)?;
        let child = tree.get_element_mut(child_node)?;
        child.get_background_mut(0).image = Some(src);
        child.get_background_mut(0).is_src = true;
        tree.mark_dirty(child_node)?;
        Ok(())
    }
}
