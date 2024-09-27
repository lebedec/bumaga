use serde_json::Value;
use taffy::{NodeId, TaffyTree};

use crate::html::TextBinding;
use crate::rendering::Renderer;
use crate::tree::{ViewTree, ViewTreeExtensions};
use crate::{Element, Keys, ViewError, ViewModel};

const VALUE: usize = 0;
const CARET: usize = 1;

impl Renderer {
    pub(crate) fn render_input(&mut self, input: &mut Element) -> Result<[NodeId; 2], ViewError> {
        let empty = String::new();
        let value = input.attrs.get("value").unwrap_or(&empty);
        let value = TextBinding::string(value);
        let value = self.render_text(value)?;
        let caret = TextBinding::string("|");
        let caret = self.render_text(caret)?;
        Ok([value, caret])
    }
}

impl ViewModel {
    pub fn handle_input_char(
        &mut self,
        node: NodeId,
        char: char,
        tree: &mut ViewTree,
    ) -> Result<(), ViewError> {
        let element = tree.get_element_mut(node)?;
        if !element.state.focus {
            return Ok(());
        }
        let undefined = String::new();
        let mut value = element.value().unwrap_or(&undefined).clone();
        value.push(char);
        self.update_input_value(node, value, tree)
    }

    pub fn handle_input_key_up(
        &mut self,
        node: NodeId,
        key: Keys,
        tree: &mut ViewTree,
    ) -> Result<(), ViewError> {
        let element = tree.get_element_mut(node)?;
        if !element.state.focus {
            return Ok(());
        }
        let undefined = String::new();
        let value = element.value().unwrap_or(&undefined);
        if key == Keys::Enter {
            let this = Value::String(value.clone());
            self.fire(element, "onchange", this.clone());
        }
        if key == Keys::Backspace {
            if value.len() > 0 {
                let mut value = value.clone();
                value.pop();
                self.update_input_value(node, value, tree)?
            }
        }
        Ok(())
    }

    pub fn handle_input_blur(
        &mut self,
        node: NodeId,
        tree: &mut ViewTree,
    ) -> Result<(), ViewError> {
        let undefined = String::new();
        let element = tree.get_element_mut(node)?;
        let this = Value::String(element.value().unwrap_or(&undefined).clone());
        self.fire(element, "onchange", this.clone());
        self.fire(element, "onblur", this);
        Ok(())
    }

    pub fn update_input_disabled(
        &mut self,
        _node: NodeId,
        _disabled: bool,
        _tree: &mut ViewTree,
    ) -> Result<(), ViewError> {
        Ok(())
    }

    pub fn update_input_value(
        &mut self,
        node: NodeId,
        value: String,
        tree: &mut ViewTree,
    ) -> Result<(), ViewError> {
        let element = tree.get_element_mut(node)?;
        element.attrs.insert("value".to_string(), value.clone());
        let this = Value::String(value.clone());
        self.fire(element, "oninput", this);
        let text = tree.child_at_index(node, VALUE)?;
        let text = tree
            .get_node_context_mut(text)
            .expect("input value has element");
        text.text
            .as_mut()
            .expect("input value element has text")
            .set(0, value);
        let text = text.node;
        tree.mark_dirty(text).map_err(ViewError::from)
    }
}
