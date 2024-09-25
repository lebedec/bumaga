use crate::tree::{ViewTree, ViewTreeExtensions};
use crate::{ViewError, ViewModel};
use serde_json::Value;
use taffy::NodeId;

impl ViewModel {
    pub fn handle_option_click(
        &mut self,
        node: NodeId,
        tree: &mut ViewTree,
    ) -> Result<(), ViewError> {
        let element = tree.get_element_mut(node)?;
        let selected = element.attrs.contains_key("selected");
        self.update_option_selected(node, !selected, tree)
    }

    pub fn update_option_selected(
        &mut self,
        node: NodeId,
        selected: bool,
        tree: &mut ViewTree,
    ) -> Result<(), ViewError> {
        let element = tree.get_element_mut(node)?;
        if selected {
            element.attrs.insert("selected".to_string(), "".to_string());
            element.state.checked = true;
        } else {
            element.attrs.remove("selected");
            element.state.checked = false;
        }
        let parent = tree.get_parent_node(node)?;
        let multiple_mode = {
            let select = tree.get_element(parent)?;
            select.attrs.contains_key("multiple")
        };
        let undefined = String::new();
        if multiple_mode {
            let mut selection = vec![];
            for option in tree.list_children(parent) {
                let option = tree.get_element_mut(option)?;
                if option.attrs.contains_key("selected") {
                    let value = option.value().unwrap_or(&undefined).clone();
                    selection.push(Value::String(value.clone()));
                }
            }
            let select = tree.get_element(parent)?;
            self.fire(select, "onchange", Value::Array(selection));
        } else {
            if selected {
                for option in tree.list_children(parent) {
                    let option = tree.get_element_mut(option)?;
                    if option.node != node {
                        option.attrs.remove("selected");
                        option.state.checked = false;
                    } else {
                        let value = option.value().unwrap_or(&undefined).clone();
                        let select = tree.get_element(parent)?;
                        self.fire(select, "onchange", Value::String(value))
                    }
                }
            }
        }
        Ok(())
    }
}
