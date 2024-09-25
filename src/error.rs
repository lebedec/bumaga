use std::io;

use taffy::TaffyError;

use crate::css;
use crate::html;

#[derive(Debug)]
pub enum ViewError {
    Layout(TaffyError),
    ElementNotFound,
    ElementTextContentNotFound,
    ParentNotFound,
    Html(html::ReaderError),
    Css(css::ReaderError),
    Io(io::Error),
    BodyNotFound,
    ElementInvalidBehaviour,
    AttributeBindingNotFound(String),
}

impl From<TaffyError> for ViewError {
    fn from(error: TaffyError) -> Self {
        ViewError::Layout(error)
    }
}

impl From<html::ReaderError> for ViewError {
    fn from(error: html::ReaderError) -> Self {
        ViewError::Html(error)
    }
}

impl From<css::ReaderError> for ViewError {
    fn from(error: css::ReaderError) -> Self {
        ViewError::Css(error)
    }
}

impl From<io::Error> for ViewError {
    fn from(error: io::Error) -> Self {
        ViewError::Io(error)
    }
}
