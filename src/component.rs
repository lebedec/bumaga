use std::fs;
use std::mem::take;
use std::ops::Add;
use std::path::Path;
use std::time::SystemTime;

use log::error;
use serde_json::{Error, Map, Value};
use taffy::prelude::length;
use taffy::style_helpers::TaffyMaxContent;
use taffy::{
    AvailableSpace, Layout, NodeId, Point, Position, PrintTree, Size, Style, TaffyResult,
    TaffyTree, TraversePartialTree,
};

use crate::api::{CallOld, Component, Input, Output};
use crate::css::{read_css, read_css_unchecked, Css};
use crate::html::{read_html, read_html_unchecked, Html};
use crate::input::FakeFonts;
use crate::models::{ElementId, Sizes};
use crate::rendering::as_string;
use crate::state::State;
use crate::styles::create_element;
use crate::{Element, Fonts, Keys, Source, LEFT_MOUSE_BUTTON};

impl Component {
    pub fn watch_files<P: AsRef<Path>>(html_path: P, css_path: P, resources: P) -> Self {
        let css = Source::from_file(read_css_unchecked, css_path);
        let html = Source::from_file(read_html_unchecked, html_path);
        Self::compile_component(html, css, &resources.as_ref().display().to_string())
    }

    pub fn compile_files<P: AsRef<Path>>(html: P, css: P, resources: P) -> Self {
        let html_error = format!("unable to read html file {:?}", html.as_ref());
        let html = fs::read_to_string(html).expect(&html_error);
        let css_error = format!("unable to read css file {:?}", css.as_ref());
        let css = fs::read_to_string(css).expect(&css_error);

        Self::compile(&html, &css, &resources.as_ref().display().to_string())
    }

    pub fn compile(html: &str, css: &str, resources: &str) -> Component {
        let css = Source::from_content(read_css(css).expect("must be read"));
        let html = Source::from_content(read_html(html).expect("must be read"));

        Self::compile_component(html, css, resources)
    }

    pub fn compile_component(html: Source<Html>, css: Source<Css>, resources: &str) -> Component {
        let state = State::new();
        Self {
            css,
            html,
            state,
            resources: resources.to_string(),
            tree: TaffyTree::new(),
            root: NodeId::new(0),
        }
    }

    pub fn resources(mut self, resources: &str) -> Self {
        self.resources = resources.to_string();
        self
    }

    pub fn reset_state(&mut self) {
        self.state = State::new();
    }
}

impl<T> Source<T> {
    pub fn from_content(content: T) -> Self {
        Source {
            path: None,
            modified: SystemTime::now(),
            content,
        }
    }

    pub fn from_file<P: AsRef<Path>, F: FnOnce(&str) -> T>(reader: F, path: P) -> Self {
        let modified = match fs::metadata(path.as_ref()).and_then(|meta| meta.modified()) {
            Ok(modified) => modified,
            Err(error) => {
                error!(
                    "unable to get {} metadata, {error:?}",
                    path.as_ref().display()
                );
                SystemTime::now()
            }
        };
        let content_error = format!("unable to read css file {:?}", path.as_ref());
        let content = fs::read_to_string(path.as_ref()).expect(&content_error);
        let content = reader(&content);
        Source {
            path: Some(path.as_ref().to_path_buf()),
            modified,
            content,
        }
    }
}
