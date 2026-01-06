use anyhow::{Context, Result};
use image::ImageEncoder;
use plotters::prelude::*;

pub struct GraphConfig {
    pub title: Option<String>,
    pub x_label: String,
    pub y_label: String,
    pub width: u32,
    pub height: u32,
}

pub fn generate_line_graph(
    x_values: Vec<f64>,
    y_values: Vec<f64>,
    config: GraphConfig,
) -> Result<Vec<u8>> {
    if x_values.len() != y_values.len() {
        anyhow::bail!(
            "X and Y data must have the same length (x: {}, y: {})",
            x_values.len(),
            y_values.len()
        );
    }

    if x_values.is_empty() {
        anyhow::bail!("Cannot create graph with no data points");
    }

    let mut buffer = vec![0u8; (config.width * config.height * 3) as usize];

    {
        let root = BitMapBackend::with_buffer(&mut buffer, (config.width, config.height))
            .into_drawing_area();

        root.fill(&WHITE)
            .context("Failed to fill background")?;

        let x_min = x_values
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let x_max = x_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let y_min = y_values
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let y_max = y_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        let x_range = if x_min == x_max {
            (x_min - 1.0)..(x_max + 1.0)
        } else {
            let padding = (x_max - x_min) * 0.05;
            (x_min - padding)..(x_max + padding)
        };

        let y_range = if y_min == y_max {
            (y_min - 1.0)..(y_max + 1.0)
        } else {
            let padding = (y_max - y_min) * 0.05;
            (y_min - padding)..(y_max + padding)
        };

        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .caption(
                config.title.as_deref().unwrap_or(""),
                ("sans-serif", 20),
            )
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(x_range, y_range)
            .context("Failed to build chart")?;

        chart
            .configure_mesh()
            .x_desc(&config.x_label)
            .y_desc(&config.y_label)
            .draw()
            .context("Failed to draw mesh")?;

        let points: Vec<(f64, f64)> = x_values
            .into_iter()
            .zip(y_values.into_iter())
            .collect();

        chart
            .draw_series(LineSeries::new(points, &BLUE.mix(0.8)))
            .context("Failed to draw line series")?;

        root.present().context("Failed to present drawing")?;
    }

    let mut png_bytes = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder
            .write_image(
                &buffer,
                config.width,
                config.height,
                image::ColorType::Rgb8,
            )
            .context("Failed to encode PNG")?;
    }

    Ok(png_bytes)
}
