use anyhow::{Context, Result};
use image::ImageEncoder;
use plotters::prelude::*;
use plotters::style::{FontStyle, FontTransform, text_anchor::{HPos, VPos, Pos}};
use crate::ir::{SceneGraph, PanelScene, DrawCommand};
use crate::{OutputFormat, RenderOptions};
use crate::theme_resolve::{ResolvedTheme, FontFace, parse_color as resolve_color};

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

/// Style configuration for ribbon layers
#[derive(Debug, Clone, Default)]
pub struct RibbonStyle {
    pub color: Option<String>,
    pub alpha: Option<f64>,
}

/// Style configuration for boxplot layers
#[derive(Debug, Clone, Default)]
pub struct BoxplotStyle {
    pub color: Option<String>,
    pub width: Option<f64>,
    pub alpha: Option<f64>,
    pub outlier_color: Option<String>,
    pub outlier_size: Option<f64>,
    pub outlier_shape: Option<String>,
}

/// Style configuration for violin layers
#[derive(Debug, Clone, Default)]
pub struct ViolinStyle {
    pub color: Option<String>,
    pub width: Option<f64>,
    pub alpha: Option<f64>,
    pub draw_quantiles: Vec<f64>,
}

/// Style configuration for density layers
#[derive(Debug, Clone, Default)]
pub struct DensityStyle {
    pub color: Option<String>,
    pub alpha: Option<f64>,
}

/// Style configuration for heatmap layers
#[derive(Debug, Clone, Default)]
pub struct HeatmapStyle {
    pub alpha: Option<f64>,
    pub value_min: f64,
    pub value_max: f64,
}

/// Convert angle to plotters FontTransform (90-degree increments only)
fn angle_to_font_transform(angle: f64) -> FontTransform {
    let normalized = ((angle % 360.0) + 360.0) % 360.0;
    if normalized >= 315.0 || normalized < 45.0 {
        FontTransform::None
    } else if normalized >= 45.0 && normalized < 135.0 {
        FontTransform::Rotate90
    } else if normalized >= 135.0 && normalized < 225.0 {
        FontTransform::Rotate180
    } else {
        FontTransform::Rotate270
    }
}

/// Convert hjust (0.0-1.0) to plotters HPos
fn hjust_to_hpos(hjust: f64) -> HPos {
    if hjust <= 0.25 {
        HPos::Left
    } else if hjust >= 0.75 {
        HPos::Right
    } else {
        HPos::Center
    }
}

/// Convert vjust (0.0-1.0) to plotters VPos
fn vjust_to_vpos(vjust: f64) -> VPos {
    if vjust <= 0.25 {
        VPos::Top
    } else if vjust >= 0.75 {
        VPos::Bottom
    } else {
        VPos::Center
    }
}

/// The Rendering Backend
pub struct Canvas;

impl Canvas {
    /// Execute the SceneGraph and produce a byte vector (PNG or SVG)
    pub fn execute(scene: SceneGraph, options: &RenderOptions) -> Result<Vec<u8>> {
        match options.format {
            OutputFormat::Png => Self::render_png(scene, options),
            OutputFormat::Svg => Self::render_svg(scene, options),
        }
    }

    fn render_png(scene: SceneGraph, _options: &RenderOptions) -> Result<Vec<u8>> {
        let width = scene.width;
        let height = scene.height;
        let mut buffer = vec![0u8; (width * height * 3) as usize];

        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
                .into_drawing_area();
            Self::draw_scene(&root, &scene)?;
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

    fn render_svg(scene: SceneGraph, _options: &RenderOptions) -> Result<Vec<u8>> {
        let mut buffer = String::new();
        {
            let root = SVGBackend::with_string(&mut buffer, (scene.width, scene.height))
                .into_drawing_area();
            Self::draw_scene(&root, &scene)?;
        }
        Ok(buffer.into_bytes())
    }

    fn draw_scene<DB: DrawingBackend>(root: &DrawingArea<DB, plotters::coord::Shift>, scene: &SceneGraph) -> Result<()>
    where DB::ErrorType: 'static {
        // Resolve theme once at the start
        let resolved_theme = scene.theme.resolve();

        // Fill background with resolved theme color
        root.fill(&resolved_theme.plot_background.fill).context("Failed to fill background")?;

        // Calculate header height for title + subtitle
        let title_size = resolved_theme.plot_title.size;
        let has_title = scene.labels.title.is_some();
        let has_subtitle = scene.labels.subtitle.is_some();
        let has_caption = scene.labels.caption.is_some();

        let header_height: u32 = if has_title || has_subtitle {
            let mut h = 5u32; // top padding
            if has_title {
                h += title_size as u32 + 5;
            }
            if has_subtitle {
                h += (title_size * 0.7) as u32 + 5;
            }
            h + 5 // bottom padding
        } else {
            0
        };

        let caption_height: u32 = if has_caption { 30 } else { 0 };

        // Split root into header, main, footer
        let total_height = scene.height;
        let main_height = total_height.saturating_sub(header_height).saturating_sub(caption_height);

        let (header_area, rest) = root.split_vertically(header_height);
        let (main_area, footer_area) = rest.split_vertically(main_height);

        // Draw title and subtitle in header area
        if has_title || has_subtitle {
            let mut y_offset = 8i32;

            if let Some(title) = &scene.labels.title {
                let title_style = TextStyle::from((
                    resolved_theme.plot_title.family.as_str(),
                    resolved_theme.plot_title.size as i32
                ).into_font()).color(&resolved_theme.plot_title.color);
                header_area.draw_text(title, &title_style, (10, y_offset))?;
                y_offset += title_size as i32 + 4;
            }

            if let Some(subtitle) = &scene.labels.subtitle {
                let subtitle_size = (title_size * 0.7).max(10.0);
                let subtitle_style = TextStyle::from((
                    resolved_theme.plot_title.family.as_str(),
                    subtitle_size as i32
                ).into_font()).color(&resolved_theme.axis_text.color);
                header_area.draw_text(subtitle, &subtitle_style, (10, y_offset))?;
            }
        }

        // Draw caption in footer area (right-aligned, muted)
        if let Some(caption) = &scene.labels.caption {
            let caption_style = TextStyle::from((
                "sans-serif",
                11i32
            ).into_font()).color(&resolved_theme.axis_text.color)
                .pos(Pos::new(HPos::Right, VPos::Center));
            let (w, _h) = footer_area.dim_in_pixel();
            footer_area.draw_text(caption, &caption_style, ((w as i32) - 15, 10))?;
        }

        // Determine Grid Layout
        let max_row = scene.panels.iter().map(|p| p.row).max().unwrap_or(0);
        let max_col = scene.panels.iter().map(|p| p.col).max().unwrap_or(0);

        let rows = max_row + 1;
        let cols = max_col + 1;

        let areas = main_area.split_evenly((rows, cols));

        for panel in &scene.panels {
            let area_idx = panel.row * cols + panel.col;
            if area_idx >= areas.len() { continue; }

            let area = &areas[area_idx];
            Canvas::draw_panel(area, panel, &resolved_theme)?;
        }

        root.present().context("Failed to present drawing")?;
        Ok(())
    }

    fn draw_panel<DB: DrawingBackend>(
        area: &DrawingArea<DB, plotters::coord::Shift>,
        panel: &PanelScene,
        theme: &ResolvedTheme,
    ) -> Result<()>
    where <DB as plotters::prelude::DrawingBackend>::ErrorType: 'static
    {
        let x_range = panel.x_scale.range.0..panel.x_scale.range.1;
        let y_range = panel.y_scale.range.0..panel.y_scale.range.1;

        // Dynamically calculate x label area size based on axis text angle
        let x_label_area_size = if theme.has_customization {
            let angle = theme.axis_text.angle;
            let normalized = ((angle % 360.0) + 360.0) % 360.0;
            if (45.0..135.0).contains(&normalized) || (225.0..315.0).contains(&normalized) {
                // Rotated: need more vertical space for labels
                (theme.axis_text.size * 6.0).max(80.0) as u32
            } else {
                30
            }
        } else {
            30
        };

        let mut chart_builder = ChartBuilder::on(area);

        chart_builder
            .margin(10)
            .caption(panel.title.clone().unwrap_or_default(), ("sans-serif", 15))
            .x_label_area_size(x_label_area_size)
            .y_label_area_size(40);

        let mut chart = chart_builder
            .build_cartesian_2d(x_range, y_range)
            .context("Failed to build chart")?;

        // Configure Mesh & Labels
        let mut mesh = chart.configure_mesh();

        // Only apply custom styling if theme has explicit customizations
        // Otherwise use Plotters defaults for backward compatibility
        if theme.has_customization {
            // Major Grid
            match &theme.panel_grid_major {
                Some(grid_style) => {
                    let grid_color = grid_style.color.stroke_width(grid_style.width.ceil() as u32);
                    mesh.bold_line_style(grid_color);
                }
                None => {
                    // Blank - make transparent
                    mesh.bold_line_style(RGBColor(255, 255, 255).mix(0.0));
                }
            }

            // Minor Grid
            match &theme.panel_grid_minor {
                Some(grid_style) => {
                    let grid_color = grid_style.color.stroke_width(grid_style.width.ceil() as u32);
                    mesh.light_line_style(grid_color);
                }
                None => {
                    // Blank - make transparent
                    mesh.light_line_style(RGBColor(255, 255, 255).mix(0.0));
                }
            }

            // Axis line styling
            match &theme.axis_line {
                Some(axis_style) => {
                    mesh.axis_style(axis_style.color.stroke_width(axis_style.width.ceil() as u32));
                }
                None => {
                    // Blank - hide axis lines
                    mesh.axis_style(RGBColor(255, 255, 255).stroke_width(0));
                }
            }

            // Axis ticks visibility (color follows axis_line due to plotters limitation)
            if theme.axis_ticks.is_none() {
                // Blank - hide tick marks by setting size to 0
                mesh.set_all_tick_mark_size(0i32.percent());
            }
            // When axis_ticks is Some, keep default tick size
            // Note: tick color follows axis_style (plotters limitation)

            // Axis text styling with face, rotation, and anchor support
            let font_style = match theme.axis_text.face {
                FontFace::Bold => FontStyle::Bold,
                FontFace::Italic => FontStyle::Italic,
                FontFace::BoldItalic => FontStyle::Bold, // Plotters doesn't have BoldItalic
                FontFace::Plain => FontStyle::Normal,
            };

            let base_font = (theme.axis_text.family.as_str(), theme.axis_text.size as i32)
                .into_font()
                .style(font_style);

            let pos = Pos::new(
                hjust_to_hpos(theme.axis_text.hjust),
                vjust_to_vpos(theme.axis_text.vjust),
            );

            // X-axis labels with rotation support
            let x_transform = angle_to_font_transform(theme.axis_text.angle);
            let x_axis_style = TextStyle::from(base_font.clone().transform(x_transform))
                .color(&theme.axis_text.color)
                .pos(pos);

            // Y-axis labels without rotation (typically not rotated)
            let y_axis_style = TextStyle::from(base_font)
                .color(&theme.axis_text.color)
                .pos(pos);

            mesh.x_label_style(x_axis_style);
            mesh.y_label_style(y_axis_style);
        }

        if let Some(x_label) = &panel.x_label {
            mesh.x_desc(x_label);
        }
        if let Some(y_label) = &panel.y_label {
            mesh.y_desc(y_label);
        }
        
        // Custom X Labels if categorical
        let categories_x = panel.x_scale.categories.clone();
        let formatter_x = move |v: &f64| {
            // Check if value is integer (within epsilon)
            if (v - v.round()).abs() > 1e-6 {
                return "".to_string();
            }
            
            let idx = v.round() as usize;
            if idx < categories_x.len() {
                categories_x[idx].clone()
            } else {
                "".to_string()
            }
        };

        if panel.x_scale.is_categorical {
            mesh.x_label_formatter(&formatter_x);
        }

        // Custom Y Labels if categorical (e.g. coord_flip)
        let categories_y = panel.y_scale.categories.clone();
        let formatter_y = move |v: &f64| {
            if (v - v.round()).abs() > 1e-6 {
                return "".to_string();
            }
            let idx = v.round() as usize;
            if idx < categories_y.len() {
                categories_y[idx].clone()
            } else {
                "".to_string()
            }
        };

        if panel.y_scale.is_categorical {
            mesh.y_label_formatter(&formatter_y);
        }
        
        mesh.draw().context("Failed to draw mesh")?;

        // Draw Commands
        for cmd in &panel.commands {
            match cmd {
                DrawCommand::DrawLine { points, style, legend } => {
                    let color = parse_color(&style.color, BLUE);
                    let stroke_width = style.width.unwrap_or(2.0).ceil() as u32;
                    let alpha = style.alpha.unwrap_or(1.0);
                    let color_style = color.mix(alpha).stroke_width(stroke_width);

                    let series = chart.draw_series(LineSeries::new(points.iter().cloned(), color_style))
                        .context("Failed to draw line")?;

                    if let Some(label) = legend {
                        series.label(label)
                            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.mix(alpha).stroke_width(stroke_width)));
                    }
                }
                DrawCommand::DrawPoint { points, style, legend } => {
                    let color = parse_color(&style.color, BLUE);
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
                    let color = parse_color(&style.color, BLUE);
                    let alpha = style.alpha.unwrap_or(1.0);
                    let color_style = color.mix(alpha).filled();

                    let series = chart.draw_series(std::iter::once(Rectangle::new(
                        [*tl, *br],
                        color_style
                    ))).context("Failed to draw rect")?;
                    
                    if let Some(label) = legend {
                        series.label(label)
                            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 15, y + 5)], color.mix(alpha).filled()));
                    }
                }
                DrawCommand::DrawPolygon { points, style, legend } => {
                    let color = parse_color(&style.color, BLUE);
                    let alpha = style.alpha.unwrap_or(0.5);
                    let color_style = color.mix(alpha).filled();

                    let series = chart.draw_series(std::iter::once(Polygon::new(
                        points.clone(),
                        color_style.clone()
                    ))).context("Failed to draw polygon")?;

                    if let Some(label) = legend {
                        series.label(label)
                            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 15, y + 5)], color_style.clone()));
                    }
                }
            }
        }
        
        // Draw Legend only if there are actually labeled series
        use crate::parser::ast::LegendPosition;
        use plotters::chart::SeriesLabelPosition;

        let has_legend_entries = panel.commands.iter().any(|cmd| {
            match cmd {
                DrawCommand::DrawLine { legend, .. } => legend.is_some(),
                DrawCommand::DrawPoint { legend, .. } => legend.is_some(),
                DrawCommand::DrawRect { legend, .. } => legend.is_some(),
                DrawCommand::DrawPolygon { legend, .. } => legend.is_some(),
            }
        });

        if has_legend_entries && theme.legend_position != LegendPosition::None {
            let position = match theme.legend_position {
                LegendPosition::UpperLeft => SeriesLabelPosition::UpperLeft,
                LegendPosition::UpperMiddle => SeriesLabelPosition::UpperMiddle,
                LegendPosition::UpperRight => SeriesLabelPosition::UpperRight,
                LegendPosition::MiddleLeft => SeriesLabelPosition::MiddleLeft,
                LegendPosition::MiddleMiddle => SeriesLabelPosition::MiddleMiddle,
                LegendPosition::MiddleRight => SeriesLabelPosition::MiddleRight,
                LegendPosition::LowerLeft => SeriesLabelPosition::LowerLeft,
                LegendPosition::LowerMiddle => SeriesLabelPosition::LowerMiddle,
                LegendPosition::LowerRight => SeriesLabelPosition::LowerRight,
                LegendPosition::None => unreachable!(), // handled above
            };

            chart.configure_series_labels()
                .position(position)
                .background_style(theme.panel_background.fill.mix(0.8))
                .border_style(&theme.axis_text.color)
                .label_font(("sans-serif", 12).into_font().color(&theme.axis_text.color))
                .draw()
                .context("Failed to draw legend")?;
        }

        Ok(())
    }
}

/// Parse color string to RGBColor with hex color support
fn parse_color(color_str: &Option<String>, default_color: RGBColor) -> RGBColor {
    match color_str.as_deref() {
        Some(s) => resolve_color(s).unwrap_or(default_color),
        None => default_color,
    }
}
