#!/bin/bash

# Ensure we're in the project root
cd "$(dirname "$0")"

echo "Generating example images..."

# Grouped Line Chart
echo "Generating line_grouped.png..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | point()' > examples/line_grouped.png

# Scatter Plot
echo "Generating scatter.png..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight, color: gender) | point(size: 5)' > examples/scatter.png

# Dodged Bar Chart
echo "Generating bar_dodge.png..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge")' > examples/bar_dodge.png

# Stacked Bar Chart
echo "Generating bar_stack.png..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "stack")' > examples/bar_stack.png

# Triple Dodged Bar Chart
echo "Generating bar_triple_dodge.png..."
cat examples/financials_triple.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge")' > examples/bar_triple_dodge.png

# Triple Stacked Bar Chart
echo "Generating bar_triple_stack.png..."
cat examples/financials_triple.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "stack")' > examples/bar_triple_stack.png

# Faceted Plot with Color Grouping
echo "Generating facets.png..."
cat examples/regional_sales.csv | cargo run -- 'aes(x: time, y: sales, color: product) | line() | facet_wrap(by: region)' > examples/facets.png

# --- New Examples ---

# Histogram with Theme
echo "Generating histogram.png..."
cat examples/distribution.csv | cargo run -- 'aes(x: value) | histogram(bins: 25) | labs(title: "Distribution Analysis", x: "Value", y: "Count") | theme_minimal()' > examples/histogram.png

# Horizontal Bar Chart (Coord Flip)
echo "Generating coord_flip.png..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | coord_flip() | labs(title: "Financials (Horizontal)", subtitle: "Q1-Q4 Performance")' > examples/coord_flip.png

# Ribbon Chart
echo "Generating ribbon.png..."
cat examples/ribbon_data.csv | cargo run -- 'aes(x: x, y: y, ymin: lower, ymax: upper) | ribbon(color: "blue", alpha: 0.3) | line(color: "blue") | labs(title: "Model Prediction", caption: "Shaded area represents 95% CI") | theme_minimal()' > examples/ribbon.png

# Smoothing (Linear Regression)
echo "Generating smooth.png..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight) | point(alpha: 0.5) | smooth() | labs(title: "Height vs Weight", subtitle: "Linear Regression Fit")' > examples/smooth.png

# Reverse Scale (Note: scales come LAST in parsing order)
echo "Generating scale_reverse.png..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | labs(title: "Reverse Time Axis") | scale_x_reverse()' > examples/scale_reverse.png

# Boxplot
echo "Generating boxplot.png..."
cat examples/demographics.csv | cargo run -- 'aes(x: gender, y: height, color: gender) | boxplot()' > examples/boxplot.png

# Violin Plot
echo "Generating violin.png..."
cat examples/demographics.csv | cargo run -- 'aes(x: gender, y: height, color: gender) | violin(draw_quantiles: [0.25, 0.5, 0.75]) | theme_minimal()' > examples/violin.png

# --- Theme Examples ---

# Custom Theme with Element Functions
echo "Generating theme_custom.png..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | labs(title: "Custom Styled Chart") | theme(plot_title: element_text(size: 24, color: "#2E86AB"), panel_grid_minor: element_blank(), axis_line: element_blank())' > examples/theme_custom.png

# Dark Theme Example
echo "Generating theme_dark.png..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight, color: gender) | point(size: 5) | labs(title: "Dark Theme Example") | theme(plot_background: element_rect(fill: "#1a1a2e"), panel_background: element_rect(fill: "#16213e"), text: element_text(color: "#eaeaea"), axis_text: element_text(color: "#a0a0a0"), panel_grid_minor: element_line(color: "#6e6e6e", width: 0.5), panel_grid_major: element_line(color: "white", width: 0.5), axis_line: element_line(color: "#ffffff", width: 1))' > examples/theme_dark.png

# Merged Themes (theme_minimal + customization)
echo "Generating theme_merged.png..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Merged Theme Example") | theme_minimal() | theme(plot_title: element_text(size: 20, face: "bold"))' > examples/theme_merged.png

# --- Axis Text Styling Examples ---

# Bold Axis Text
echo "Generating axis_bold.png..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Bold Axis Labels") | theme(axis_text: element_text(face: "bold", size: 14))' > examples/axis_bold.png

# Rotated X-Axis Labels (90 degrees)
echo "Generating axis_rotated.png..."
cat examples/financials.csv | cargo run -- 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Rotated X-Axis Labels") | theme(axis_text: element_text(angle: 90, size: 12))' > examples/axis_rotated.png

# Hidden Ticks (element_blank)
echo "Generating axis_no_ticks.png..."
cat examples/timeseries.csv | cargo run -- 'aes(x: time, y: value, color: series) | line() | labs(title: "Clean Look - No Ticks") | theme_minimal() | theme(axis_ticks: element_blank())' > examples/axis_no_ticks.png

# Combined: Bold, Rotated, Custom Color
echo "Generating axis_styled.png..."
cat examples/regional_sales.csv | cargo run -- 'aes(x: region, y: sales, color: product) | bar(position: "dodge") | labs(title: "Fully Styled Axes") | theme(axis_text: element_text(face: "bold", angle: 90, color: "#2E86AB", size: 11), axis_line: element_line(color: "#333333", width: 2))' > examples/axis_styled.png

# --- Variable Injection Examples ---

# Variable injection in aesthetics
echo "Generating variable_aes.png..."
cat examples/timeseries.csv | cargo run -- 'aes(x: $xcol, y: $ycol, color: series) | line() | labs(title: $title)' -D xcol=time -D ycol=value -D title='"Variable Injection Example"' > examples/variable_aes.png

# Variable injection in geometry styling
echo "Generating variable_geom.png..."
cat examples/demographics.csv | cargo run -- 'aes(x: height, y: weight) | point(color: $color, size: $size) | labs(title: "Styled with Variables")' -D color='"blue"' -D size=8 > examples/variable_geom.png

echo "Done! All examples generated in the examples/ directory."