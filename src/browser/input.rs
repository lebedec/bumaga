use taffy::{NodeId, TaffyTree};

use crate::html::TextBinding;
use crate::rendering::Renderer;
use crate::{Element, View, ViewError, ViewModel};

const VALUE: usize = 0;
const CARET: usize = 1;

impl Renderer {
    pub(crate) fn render_input(&mut self, input: &mut Element) -> Result<[NodeId; 2], ViewError> {
        let empty = String::new();
        let value = input.attrs.get("value").unwrap_or(&empty);
        input.state.value = Some(value.clone());
        let value = TextBinding::string(value);
        let value = self.render_text(value)?;
        let caret = TextBinding::string("|");
        let caret = self.render_text(caret)?;
        Ok([value, caret])
    }
}

impl ViewModel {
    pub(crate) fn update_input_value(
        &self,
        input: NodeId,
        value: String,
        view: &mut TaffyTree<Element>,
    ) -> Result<(), ViewError> {
        let value = value.clone();
        let text = view.child_at_index(input, VALUE)?;
        let text = view
            .get_node_context_mut(text)
            .expect("input value has element");
        text.text
            .as_mut()
            .expect("input value element has text")
            .spans[0] = value;
        let text = text.node;
        view.mark_dirty(text)?;
        Ok(())
    }
}

impl View {
    pub(crate) fn update_input_view(
        &mut self,
        input: NodeId,
        value: String,
    ) -> Result<(), ViewError> {
        let input = self
            .tree
            .get_node_context_mut(input)
            .ok_or(ViewError::ElementNotFound)?;
        input.state.value = Some(value.clone());
        let parent = input.node;
        let text = self.tree.child_at_index(parent, VALUE)?;
        let text = self
            .tree
            .get_node_context_mut(text)
            .expect("input value has element");
        text.text
            .as_mut()
            .expect("input value element has text")
            .spans[0] = value;
        let text = text.node;
        self.tree.mark_dirty(text)?;
        Ok(())
    }
}
