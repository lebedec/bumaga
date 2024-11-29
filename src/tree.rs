use crate::{Element, ViewError};
use taffy::{NodeId, TaffyTree};

pub type ViewTree = TaffyTree<Element>;

pub trait ViewTreeExtensions {
    fn get_element_mut(&mut self, node: NodeId) -> Result<&mut Element, ViewError>;
    fn get_element(&self, node: NodeId) -> Result<&Element, ViewError>;
}

impl ViewTreeExtensions for ViewTree {
    #[inline(always)]
    fn get_element_mut(&mut self, node: NodeId) -> Result<&mut Element, ViewError> {
        self.get_node_context_mut(node)
            .ok_or(ViewError::ElementNotFound(node))
    }

    #[inline(always)]
    fn get_element(&self, node: NodeId) -> Result<&Element, ViewError> {
        self.get_node_context(node)
            .ok_or(ViewError::ElementNotFound(node))
    }
}
