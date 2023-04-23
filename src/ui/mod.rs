use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
};

use crate::handlers::{app::App, config::FrontendConfig};

pub mod components;
pub mod render;
pub mod statics;

#[derive(Debug, Clone)]
pub struct LayoutAttributes {
    constraints: Vec<Constraint>,
    chunks: Vec<Rect>,
}

impl LayoutAttributes {
    pub fn new(constraints: Vec<Constraint>, chunks: Vec<Rect>) -> Self {
        Self {
            constraints,
            chunks,
        }
    }

    pub fn first_chunk(&self) -> Rect {
        self.chunks[0]
    }

    pub fn last_chunk(&self) -> Rect {
        self.chunks[self.chunks.len() - 1]
    }
}

pub struct WindowAttributes<'a, 'b, 'c, T: Backend> {
    frame: &'a mut Frame<'b, T>,
    app: &'c mut App,
    layout: Option<LayoutAttributes>,
    frontend: FrontendConfig,
}

impl<'a, 'b, 'c, T: Backend> WindowAttributes<'a, 'b, 'c, T> {
    pub fn new(
        frame: &'a mut Frame<'b, T>,
        app: &'c mut App,
        layout: Option<LayoutAttributes>,
        frontend: FrontendConfig,
    ) -> Self {
        Self {
            frame,
            app,
            layout,
            frontend,
        }
    }
}
