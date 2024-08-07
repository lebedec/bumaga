use taffy::TaffyError;

#[derive(Debug)]
pub enum ComponentError {
    Layout(TaffyError),
    ElementNotFoundInTree,
}

impl From<TaffyError> for ComponentError {
    fn from(error: TaffyError) -> Self {
        ComponentError::Layout(error)
    }
}
