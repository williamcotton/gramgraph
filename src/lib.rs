// Library exports for gramgraph

pub mod csv_reader;
pub mod data;
pub mod datetime;
pub mod graph;
pub mod palette;
pub mod parser;
pub mod runtime;

// New Architecture Modules
pub mod compiler;
pub mod ir;
pub mod preprocessor;
pub mod resolve;
pub mod scale;
pub mod theme_resolve;
pub mod transform;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub enum OutputFormat {
    #[serde(rename = "png")]
    #[default]
    Png,
    #[serde(rename = "svg")]
    Svg,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RenderOptions {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default, rename = "type")]
    pub format: OutputFormat,
}

fn default_width() -> u32 {
    800
}
fn default_height() -> u32 {
    600
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            format: OutputFormat::Png,
        }
    }
}
