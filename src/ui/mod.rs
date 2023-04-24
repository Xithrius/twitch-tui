use std::collections::HashMap;

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
};

use crate::handlers::{app::App, config::FrontendConfig};

pub mod components;
pub mod render;
pub mod statics;
