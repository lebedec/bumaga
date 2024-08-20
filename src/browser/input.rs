use serde_json::Value;
use std::collections::HashSet;
use taffy::{NodeId, TaffyTree};

use crate::html::TextBinding;
use crate::rendering::Renderer;
use crate::{Behaviour, Element, Output, ValueExtensions, View, ViewError, ViewModel};

const VALUE: usize = 0;
const CARET: usize = 1;

impl Renderer {
    pub(crate) fn render_input(&mut self, input: &mut Element) -> Result<[NodeId; 2], ViewError> {
        let empty = String::new();
        let value = input.attrs.get("value").unwrap_or(&empty);
        input.state.behaviour = Behaviour::Input(value.clone());
        let value = TextBinding::string(value);
        let value = self.render_text(value)?;
        let caret = TextBinding::string("|");
        let caret = self.render_text(caret)?;
        Ok([value, caret])
    }

    pub(crate) fn render_select(&mut self, select: &mut Element) -> Result<(), ViewError> {
        select.state.behaviour = if select.attrs.contains_key("multiple") {
            Behaviour::SelectMultiple(HashSet::new())
        } else {
            Behaviour::Select(String::new())
        };
        Ok(())
    }
}

impl ViewModel {
    pub(crate) fn update_option_value(
        &self,
        option: NodeId,
        view: &mut TaffyTree<Element>,
        output: &mut Output,
    ) -> Result<(), ViewError> {
        let undefined = String::new();
        let select: &mut Element = unsafe {
            let select = view.parent(option).ok_or(ViewError::ParentNotFound)?;
            let select = view
                .get_node_context_mut(select)
                .ok_or(ViewError::ElementNotFound)?;
            &mut *(select as *mut _)
        };
        let option = view
            .get_node_context_mut(option)
            .ok_or(ViewError::ElementNotFound)?;
        let option = option.attrs.get("value").unwrap_or(&undefined).clone();
        let value = if select.state.as_select_multiple().is_ok() {
            let selection = select.state.as_select_multiple()?;
            if selection.contains(&option) {
                selection.remove(&option);
            } else {
                selection.insert(option);
            }
            Value::Array(
                selection
                    .iter()
                    .map(|string| Value::String(string.clone()))
                    .collect(),
            )
        } else {
            let selection = select.state.as_select()?;
            *selection = option;
            Value::String(selection.clone())
        };
        self.fire(select, "onchange", value, output);
        update_select_options(select, view)
    }

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

fn update_select_options(
    select: &mut Element,
    view: &mut TaffyTree<Element>,
) -> Result<(), ViewError> {
    let undefined = String::new();
    if select.state.as_select_multiple().is_ok() {
        let selection = select.state.as_select_multiple()?;
        for option in view.children(select.node)? {
            let option = view
                .get_node_context_mut(option)
                .ok_or(ViewError::ElementNotFound)?;
            let variant = option.attrs.get("value").unwrap_or(&undefined);
            option.state.checked = selection.contains(variant)
        }
    } else {
        let selection = select.state.as_select()?;
        for option in view.children(select.node)? {
            let option = view
                .get_node_context_mut(option)
                .ok_or(ViewError::ElementNotFound)?;
            let variant = option.attrs.get("value").unwrap_or(&undefined);
            option.state.checked = variant == selection;
        }
    }
    Ok(())
}

impl View {
    pub(crate) fn update_select_view(
        &mut self,
        select: NodeId,
        value: Value,
    ) -> Result<(), ViewError> {
        let select: &mut Element = unsafe {
            let select = self
                .tree
                .get_node_context_mut(select)
                .ok_or(ViewError::ElementNotFound)?;
            &mut *(select as *mut _)
        };
        if select.state.as_select_multiple().is_ok() {
            let selection = select.state.as_select_multiple()?;
            *selection = HashSet::from_iter(value.eval_array());
        } else {
            let selection = select.state.as_select()?;
            *selection = value.eval_string();
        }
        update_select_options(select, &mut self.tree)
    }

    pub(crate) fn update_input_view(
        &mut self,
        input: NodeId,
        value: String,
    ) -> Result<(), ViewError> {
        let input = self.get_element_mut(input)?;
        *input.state.as_input()? = value.clone();
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
