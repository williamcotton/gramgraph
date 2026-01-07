use anyhow::Result;
use crate::parser::ast::PlotSpec;
use crate::csv_reader::CsvData;
use crate::{resolve, transform, scale, compiler, graph};

/// Render a plot specification to PNG bytes using the Ideal GoG Pipeline
pub fn render_plot(spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>> {
    // Check for empty data (maintain legacy behavior for tests)
    if csv_data.rows.is_empty() {
        anyhow::bail!("Plot requires at least one data row");
    }

    // PHASE 1: RESOLUTION
    // Resolve all aesthetics for all layers once.
    let resolved_spec = resolve::resolve_plot_aesthetics(&spec, &csv_data)?;

    // PHASE 2: TRANSFORMATION
    // Apply stats (binning) and positions (stacking/dodging).
    // Returns RenderData with normalized geometry points.
    let render_data = transform::apply_transformations(&resolved_spec, &csv_data)?;

    // PHASE 3: SCALING
    // Measure the RenderData to determine coordinate ranges.
    let scales = scale::build_scales(&render_data, resolved_spec.facet.as_ref())?;

    // PHASE 4: COMPILATION (MAPPING)
    // Convert data units to drawing commands.
    let scene = compiler::compile_geometry(render_data, scales, &resolved_spec)?;

    // PHASE 5: RENDERING
    // Execute drawing commands on the canvas.
    graph::Canvas::execute(scene)
}