use crate::data::PlotData;
use crate::parser::ast::PlotSpec;
use crate::{compiler, graph, resolve, scale, transform, RenderOptions};
use anyhow::Result;

/// Render a plot specification to PNG bytes using the Ideal GoG Pipeline
pub fn render_plot(spec: PlotSpec, data: PlotData, options: RenderOptions) -> Result<Vec<u8>> {
    // Check for empty data (maintain legacy behavior for tests)
    if data.rows.is_empty() {
        anyhow::bail!("Plot requires at least one data row");
    }

    // PHASE 1: RESOLUTION
    // Resolve all aesthetics for all layers once.
    // Variables are substituted during resolution.
    let resolved_spec = resolve::resolve_plot_aesthetics(&spec, &data)?;

    // PHASE 2: TRANSFORMATION
    // Apply stats (binning) and positions (stacking/dodging).
    // Returns RenderData with normalized geometry points.
    let render_data = transform::apply_transformations(&resolved_spec, &data)?;

    // 3. Scaling
    let scales = scale::build_scales(&render_data, &resolved_spec)?;

    // PHASE 4: COMPILATION (MAPPING)
    // Convert data units to drawing commands.
    let scene = compiler::compile_geometry(render_data, scales, &resolved_spec, &options)?;

    // PHASE 5: RENDERING
    // Execute drawing commands on the canvas.
    graph::Canvas::execute(scene, &options)
}
