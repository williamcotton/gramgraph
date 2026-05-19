#!/bin/bash

# Ensure we're in the project root
cd "$(dirname "$0")"

echo "Generating example images..."

# Grouped Line Chart
echo "Generating line_grouped.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | point() | theme_minimal()' --format svg > examples/line_grouped.svg

# Datetime Scale
echo "Generating weather_datetime.svg..."
cat examples/weather_hourly.csv | cargo run -- 'aes(x: time, y: temp) | line() | point() | theme_minimal() | scale_x_datetime(interval: "20h", format: "%b %-d %H:%M")' --format svg > examples/weather_datetime.svg

# Scatter Plot
echo "Generating scatter.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight, color: gender) | point(size: 5) | theme_minimal()' --format svg > examples/scatter.svg

# Shape and Alpha Mappings
echo "Generating shape_alpha.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight, shape: gender, alpha: gender) | point(size: 7, color: "steelblue") | labs(title: "Shape and Alpha Mapping", x: "Height (cm)", y: "Weight (kg)") | theme_minimal()' --format svg > examples/shape_alpha.svg

# Dodged Bar Chart
echo "Generating bar_dodge.svg..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | theme_minimal()' --format svg > examples/bar_dodge.svg

# Stacked Bar Chart
echo "Generating bar_stack.svg..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "stack") | theme_minimal()' --format svg > examples/bar_stack.svg

# Triple Dodged Bar Chart
echo "Generating bar_triple_dodge.svg..."
cat examples/financials_triple.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | theme_minimal()' --format svg > examples/bar_triple_dodge.svg

# Triple Stacked Bar Chart
echo "Generating bar_triple_stack.svg..."
cat examples/financials_triple.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "stack") | theme_minimal()' --format svg > examples/bar_triple_stack.svg

# Faceted Plot with Color Grouping
echo "Generating facets.svg..."
cat examples/regional_sales.csv | cargo run -- 'aes(x: time, y: sales, color: product) | line() | facet_wrap(by: region) | theme_minimal()' --format svg > examples/facets.svg

# --- New Examples ---

# Histogram with Theme
echo "Generating histogram.svg..."
cat examples/distribution.csv | cargo run -- 'aes(x: value) | histogram(bins: 25) | labs(title: "Distribution Analysis", x: "Value", y: "Count") | theme_minimal()' --format svg > examples/histogram.svg

# Horizontal Bar Chart (Coord Flip)
echo "Generating coord_flip.svg..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | coord_flip() | labs(title: "Financials (Horizontal)", subtitle: "Q1-Q4 Performance") | theme_minimal()' --format svg > examples/coord_flip.svg

# Ribbon Chart
echo "Generating ribbon.svg..."
cat examples/ribbon_data.csv | cargo run -- 'aes(x: x, y: y, ymin: lower, ymax: upper) | ribbon(color: "blue", alpha: 0.3) | line(color: "blue") | labs(title: "Model Prediction", caption: "Shaded area represents 95% CI") | theme_minimal()' --format svg > examples/ribbon.svg

# Area Chart
echo "Generating area.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | area(alpha: 0.25, baseline: 0) | line(width: 2) | labs(title: "Area Chart", x: "Time", y: "Value") | theme_minimal()' --format svg > examples/area.svg

# Step Line Chart
echo "Generating step.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | step(direction: "mid", width: 2) | point(size: 4) | labs(title: "Step Line Chart", x: "Time", y: "Value") | theme_minimal()' --format svg > examples/step.svg

# Reference Lines
echo "Generating reference_lines.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | hline(yintercept: 12, color: "red", width: 2, alpha: 0.8, label: "Target y = 12") | vline(xintercept: 3, color: "gray40", width: 2, label: "Time marker x = 3") | labs(title: "Reference Lines", x: "Time", y: "Value") | theme_minimal() | theme(legend_position: "bottom")' --format svg > examples/reference_lines.svg

# Smoothing (Linear Regression)
echo "Generating smooth.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight) | point(alpha: 0.5) | smooth() | labs(title: "Height vs Weight", subtitle: "Linear Regression Fit") | theme_minimal()' --format svg > examples/smooth.svg

# Smoothing (LOESS)
echo "Generating smooth_loess.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight) | point(alpha: 0.35) | smooth(method: "loess", span: 0.65, color: "red", width: 3) | labs(title: "Height vs Weight", subtitle: "LOESS Fit") | theme_minimal()' --format svg > examples/smooth_loess.svg

# Reverse Scale (Note: scales come LAST in parsing order)
echo "Generating scale_reverse.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | labs(title: "Reverse Time Axis") | theme_minimal() | scale_x_reverse()' --format svg > examples/scale_reverse.svg

# Log10 Scale
echo "Generating scale_log10.svg..."
cat examples/scales.csv | cargo run -- 'aes(x: x, y: value) | line(color: "steelblue", width: 2) | point(shape: "triangle", size: 6, color: "steelblue") | labs(title: "Log10 X Scale", x: "Input", y: "Value") | theme_minimal() | scale_x_log10()' --format svg > examples/scale_log10.svg

# Square Root Scale
echo "Generating scale_sqrt.svg..."
cat examples/scales.csv | cargo run -- 'aes(x: x, y: value) | line(color: "purple", width: 2) | point(shape: "diamond", size: 6, color: "purple") | labs(title: "Square Root X Scale", x: "Input", y: "Value") | theme_minimal() | scale_x_sqrt()' --format svg > examples/scale_sqrt.svg

# Boxplot
echo "Generating boxplot.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: gender, y: height, color: gender) | boxplot() | theme_minimal()' --format svg > examples/boxplot.svg

# Violin Plot
echo "Generating violin.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: gender, y: height, color: gender) | violin(draw_quantiles: [0.25, 0.5, 0.75]) | theme_minimal()' --format svg > examples/violin.svg

# Density Plot
echo "Generating density.svg..."
cat examples/distribution.csv | cargo run -- 'aes(x: value) | density() | labs(title: "Density Estimate", x: "Value", y: "Density") | theme_minimal()' --format svg > examples/density.svg

# Density Plot with Color Grouping
echo "Generating density_grouped.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, color: gender) | density(alpha: 0.4) | labs(title: "Height Distribution by Gender", x: "Height (cm)", y: "Density") | theme_minimal()' --format svg > examples/density_grouped.svg

# Heatmap (Categorical)
echo "Generating heatmap.svg..."
cat examples/heatmap_data.csv | cargo run -- 'aes(x: x, y: y, fill: value) | heatmap() | labs(title: "Weekly Activity Heatmap", x: "Day", y: "Time of Day") | theme_minimal()' --format svg > examples/heatmap.svg

# Nice Ticks (irregular data with clean axis labels)
echo "Generating nice_ticks.svg..."
cat examples/measurements.csv | cargo run -- 'aes(x: elapsed, y: temperature) | point(color: "steelblue", size: 4) | line(color: "steelblue", alpha: 0.5) | labs(title: "Sensor Readings", subtitle: "Nice ticks from irregular sample times", x: "Elapsed Time (hrs)", y: "Temperature (C)") | theme_minimal()' --format svg > examples/nice_ticks.svg

# --- Theme Examples ---

# Custom Theme with Element Functions
echo "Generating theme_custom.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | labs(title: "Custom Styled Chart") | theme_minimal() | theme(plot_title: element_text(size: 24, color: "#2E86AB"), panel_grid_minor: element_blank(), axis_line: element_blank())' --format svg > examples/theme_custom.svg

# Dark Theme Example
echo "Generating theme_dark.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight, color: gender) | point(size: 5) | labs(title: "Dark Theme Example") | theme_minimal() | theme_dark()' --format svg > examples/theme_dark.svg

# Classic Theme Example
echo "Generating theme_classic.svg..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Classic Theme Example") | theme_minimal() | theme_classic()' --format svg > examples/theme_classic.svg

# Void Theme Example
echo "Generating theme_void.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight, color: gender) | point(size: 5) | labs(title: "Void Theme Example") | theme_minimal() | theme_void()' --format svg > examples/theme_void.svg

# Custom Legend Configuration
echo "Generating legend_custom.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line(width: 3) | point(size: 4) | labs(title: "Custom Legend") | theme_minimal() | theme(legend_position: "bottom", legend_text: element_text(size: 14, color: "#222222"), legend_background: element_rect(fill: "#F7F7F7", color: "#333333", width: 1), legend_margin: 6, legend_key_size: 22)' --format svg > examples/legend_custom.svg

# Merged Themes (theme_minimal + customization)
echo "Generating theme_merged.svg..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Merged Theme Example") | theme_minimal() | theme(plot_title: element_text(size: 20, face: "bold"))' --format svg > examples/theme_merged.svg

# --- Axis Text Styling Examples ---

# Bold Axis Text
echo "Generating axis_bold.svg..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Bold Axis Labels") | theme_minimal() | theme(axis_text: element_text(face: "bold", size: 14))' --format svg > examples/axis_bold.svg

# Rotated X-Axis Labels (90 degrees)
echo "Generating axis_rotated.svg..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Rotated X-Axis Labels") | theme_minimal() | theme(axis_text: element_text(angle: 90, size: 12))' --format svg > examples/axis_rotated.svg

# Hidden Ticks (element_blank)
echo "Generating axis_no_ticks.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | labs(title: "Clean Look - No Ticks") | theme_minimal() | theme(axis_ticks: element_blank())' --format svg > examples/axis_no_ticks.svg

# Combined: Bold, Rotated, Custom Color
echo "Generating axis_styled.svg..."
cat examples/regional_sales.csv | cargo run -- 'aes(x: region, y: sales, color: product) | bar(position: "dodge") | labs(title: "Fully Styled Axes") | theme_minimal() | theme(axis_text: element_text(face: "bold", angle: 90, color: "#2E86AB", size: 11), axis_line: element_line(color: "#333333", width: 2))' --format svg > examples/axis_styled.svg

# --- Variable Injection Examples ---

# Variable injection in aesthetics
echo "Generating variable_aes.svg..."
cat examples/timeseries.csv | cargo run -- 'aes(x: $xcol, y: $ycol, color: series) | line() | labs(title: $title) | theme_minimal()' -D xcol=time -D ycol=value -D title='"Variable Injection Example"' --format svg > examples/variable_aes.svg

# Variable injection in geometry styling
echo "Generating variable_geom.svg..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight) | point(color: $color, size: $size) | labs(title: "Styled with Variables") | theme_minimal()' -D color='"blue"' -D size=8 --format svg > examples/variable_geom.svg

echo "Done! All examples generated in the examples/ directory."
