use anyhow::{Context, Result};
use image::ImageEncoder;
use plotters::prelude::*;
use crate::ir::{SceneGraph, PanelScene, DrawCommand};

/// Style configuration for line layers
#[derive(Debug, Clone, Default)]
pub struct LineStyle {
    pub color: Option<String>,
    pub width: Option<f64>,
    pub alpha: Option<f64>,
}

/// Style configuration for point layers
#[derive(Debug, Clone, Default)]
pub struct PointStyle {
    pub color: Option<String>,
    pub size: Option<f64>,
    pub shape: Option<String>,
    pub alpha: Option<f64>,
}

/// Style configuration for bar layers
#[derive(Debug, Clone, Default)]
pub struct BarStyle {
    pub color: Option<String>,
    pub alpha: Option<f64>,
    pub width: Option<f64>,
}

/// The Rendering Backend
pub struct Canvas;

impl Canvas {
    /// Execute the SceneGraph and produce a PNG
    pub fn execute(scene: SceneGraph) -> Result<Vec<u8>> {
        let width = scene.width;
        let height = scene.height;
        let mut buffer = vec![0u8; (width * height * 3) as usize];

        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
                .into_drawing_area();

            root.fill(&WHITE).context("Failed to fill background")?;

            // Determine Grid Layout
            let max_row = scene.panels.iter().map(|p| p.row).max().unwrap_or(0);
            let max_col = scene.panels.iter().map(|p| p.col).max().unwrap_or(0);
            
            let rows = max_row + 1;
            let cols = max_col + 1;

            let areas = root.split_evenly((rows, cols));

            for panel in scene.panels {
                let area_idx = panel.row * cols + panel.col;
                if area_idx >= areas.len() { continue; }
                
                let area = &areas[area_idx];
                Canvas::draw_panel(area, panel)?;
            }
            
            root.present().context("Failed to present drawing")?;
        }

        // Encode as PNG
        let mut png_bytes = Vec::new();
        {
            let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
            encoder
                .write_image(
                    &buffer,
                    width,
                    height,
                    image::ColorType::Rgb8,
                )
                .context("Failed to encode PNG")?;
        }

        Ok(png_bytes)
    }

    fn draw_panel<DB: DrawingBackend>(area: &DrawingArea<DB, plotters::coord::Shift>, panel: PanelScene) -> Result<()> 
    where <DB as plotters::prelude::DrawingBackend>::ErrorType: 'static
    {
        let x_range = panel.x_scale.range.0..panel.x_scale.range.1;
        let y_range = panel.y_scale.range.0..panel.y_scale.range.1;

        let mut chart_builder = ChartBuilder::on(area);
            
        chart_builder
            .margin(10)
            .caption(panel.title.unwrap_or_default(), ("sans-serif", 15))
            .x_label_area_size(30)
            .y_label_area_size(40);

        let mut chart = chart_builder
            .build_cartesian_2d(x_range, y_range)
            .context("Failed to build chart")?;

        // Configure Mesh & Labels
        let mut mesh = chart.configure_mesh();
        
        // Custom X Labels if categorical
        let categories = panel.x_scale.categories.clone();
        let formatter = move |v: &f64| {
            // Check if value is integer (within epsilon)
            if (v - v.round()).abs() > 1e-6 {
                return "".to_string();
            }
            
            let idx = v.round() as usize;
            if idx < categories.len() {
                categories[idx].clone()
            } else {
                "".to_string()
            }
        };

        if panel.x_scale.is_categorical {
            mesh.x_label_formatter(&formatter);
            // Ideally set ticks at integers, but Plotters auto-ticks are usually fine
        }
        
        mesh.draw().context("Failed to draw mesh")?;

        // Draw Commands
        for cmd in panel.commands {
            match cmd {
                DrawCommand::DrawLine { points, style, legend } => {
                    let color = parse_color(&style.color);
                    let stroke_width = style.width.unwrap_or(2.0) as u32;
                    let alpha = style.alpha.unwrap_or(1.0);
                    let color_style = color.mix(alpha).stroke_width(stroke_width);

                    let series = chart.draw_series(LineSeries::new(points, color_style))
                        .context("Failed to draw line")?;

                    if let Some(label) = legend {
                        series.label(label)
                            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.mix(alpha).stroke_width(stroke_width)));
                    }
                }
                DrawCommand::DrawPoint { points, style, legend } => {
                    let color = parse_color(&style.color);
                    let size = style.size.unwrap_or(3.0) as i32;
                    let alpha = style.alpha.unwrap_or(1.0);
                    let color_style = color.mix(alpha).filled();

                    let series = chart.draw_series(points.iter().map(|(x, y)| {
                        Circle::new((*x, *y), size, color_style)
                    })).context("Failed to draw points")?;

                    if let Some(label) = legend {
                        series.label(label)
                            .legend(move |(x, y)| Circle::new((x + 10, y), size, color.mix(alpha).filled()));
                    }
                }
                DrawCommand::DrawRect { tl, br, style, legend } => {
                    let color = parse_color(&style.color);
                    let alpha = style.alpha.unwrap_or(1.0);
                    let color_style = color.mix(alpha).filled();

                    let series = chart.draw_series(std::iter::once(Rectangle::new(
                        [tl, br],
                        color_style
                    ))).context("Failed to draw rect")?;
                    
                    if let Some(label) = legend {
                        series.label(label)
                            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 15, y + 5)], color.mix(alpha).filled()));
                    }
                }
            }
        }
        
        // Draw Legend if any items
        // Note: Plotters draws legend only if series were labeled
        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()
            .context("Failed to draw legend")?;

        Ok(())
    }
}

/// Parse color string to RGBColor
fn parse_color(color_str: &Option<String>) -> RGBColor {
    match color_str.as_deref() {
        Some("red") => RED,
        Some("green") => GREEN,
        Some("blue") => BLUE,
        Some("black") => BLACK,
        Some("yellow") => YELLOW,
        Some("cyan") => CYAN,
        Some("magenta") => MAGENTA,
        Some("white") => WHITE,
        Some("orange") => RGBColor(255, 165, 0),
        Some("purple") => RGBColor(128, 0, 128),
        Some("brown") => RGBColor(165, 42, 42),
        Some("pink") => RGBColor(255, 192, 203),
        Some("gray") => RGBColor(128, 128, 128),
        Some("olive") => RGBColor(128, 128, 0),
        _ => BLUE, // default
    }
}