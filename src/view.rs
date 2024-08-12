use crate::view_model::ViewModel;
use crate::Element;
use taffy::TaffyTree;

pub struct View {
    view_model: ViewModel,
    tree: TaffyTree<Element>,
}

impl View {
    pub fn compile() -> Self {
        unimplemented!()
    }

    pub fn update() {}
}
