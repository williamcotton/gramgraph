# GramGraph

A command-line tool for plotting data from CSV files using a grammar of graphics syntax.

## Usage

Pipe CSV data into `gramgraph` and provide a plot specification.

```bash
cat data.csv | gramgraph 'aes(x: time, y: value) | line()' > output.png
```

## Examples

### Grouped Line Chart

```bash
cat examples/timeseries.csv | gramgraph 'aes(x: time, y: value, color: series) | line() | point()' > examples/line_grouped.png
```

![Grouped Line Chart](examples/line_grouped.png)

### Scatter Plot

```bash
cat examples/demographics.csv | gramgraph 'aes(x: height, y: weight, color: gender) | point(size: 5)' > examples/scatter.png
```

![Scatter Plot](examples/scatter.png)

### Dodged Bar Chart

```bash
cat examples/financials.csv | gramgraph 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge")' > examples/bar_dodge.png
```

![Dodged Bar Chart](examples/bar_dodge.png)

### Stacked Bar Chart

```bash
cat examples/financials.csv | gramgraph 'aes(x: quarter, y: amount, color: type) | bar(position: "stack")' > examples/bar_stack.png
```

![Stacked Bar Chart](examples/bar_stack.png)

### Triple Dodged Bar Chart

```bash
cat examples/financials_triple.csv | gramgraph 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge")' > examples/bar_triple_dodge.png
```

![Triple Dodged Bar Chart](examples/bar_triple_dodge.png)

### Triple Stacked Bar Chart

```bash
cat examples/financials_triple.csv | gramgraph 'aes(x: quarter, y: amount, color: type) | bar(position: "stack")' > examples/bar_triple_stack.png
```

![Triple Stacked Bar Chart](examples/bar_triple_stack.png)

### Faceted Plot with Color Grouping

```bash
cat examples/regional_sales.csv | gramgraph 'aes(x: time, y: sales, color: product) | line() | facet_wrap(by: region)' > examples/facets.png
```

![Faceted Plot](examples/facets.png)

### Histogram with Theme

```bash
cat examples/distribution.csv | gramgraph 'aes(x: value) | histogram(bins: 25) | labs(title: "Distribution Analysis", x: "Value", y: "Count") | theme_minimal()' > examples/histogram.png
```

![Histogram](examples/histogram.png)

### Horizontal Bar Chart (Coord Flip)

```bash
cat examples/financials.csv | gramgraph 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | coord_flip() | labs(title: "Financials (Horizontal)", subtitle: "Q1-Q4 Performance")' > examples/coord_flip.png
```

![Horizontal Bar Chart](examples/coord_flip.png)

### Ribbon Chart

```bash
cat examples/ribbon_data.csv | gramgraph 'aes(x: x, y: y, ymin: lower, ymax: upper) | ribbon(color: "blue", alpha: 0.3) | line(color: "blue") | labs(title: "Model Prediction", caption: "Shaded area represents 95% CI") | theme_minimal()' > examples/ribbon.png
```

![Ribbon Chart](examples/ribbon.png)

### Smoothing (Linear Regression)

```bash
cat examples/demographics.csv | gramgraph 'aes(x: height, y: weight) | point(alpha: 0.5) | smooth() | labs(title: "Height vs Weight", subtitle: "Linear Regression Fit")' > examples/smooth.png
```

![Smoothing](examples/smooth.png)

### Boxplot

```bash
cat examples/demographics.csv | gramgraph 'aes(x: gender, y: height, color: gender) | boxplot()' > examples/boxplot.png
```

![Boxplot](examples/boxplot.png)

### Violin Plot

```bash
cat examples/demographics.csv | gramgraph 'aes(x: gender, y: height, color: gender) | violin(draw_quantiles: [0.25, 0.5, 0.75]) | theme_minimal()' > examples/violin.png
```

![Violin Plot](examples/violin.png)

### Reverse Scale

```bash
cat examples/timeseries.csv | gramgraph 'aes(x: time, y: value, color: series) | line() | labs(title: "Reverse Time Axis") | scale_x_reverse()' > examples/scale_reverse.png
```

![Reverse Scale](examples/scale_reverse.png)

### Custom Theme with Element Functions

```bash
cat examples/timeseries.csv | gramgraph 'aes(x: time, y: value, color: series) | line() | labs(title: "Custom Styled Chart") | theme(plot_title: element_text(size: 24, color: "#2E86AB"), panel_grid_minor: element_blank(), axis_line: element_blank())' > examples/theme_custom.png
```

![Custom Theme](examples/theme_custom.png)

### Dark Theme Example

```bash
cat examples/demographics.csv | gramgraph 'aes(x: height, y: weight, color: gender) | point(size: 5) | labs(title: "Dark Theme Example") | theme(plot_background: element_rect(fill: "#1a1a2e"), panel_background: element_rect(fill: "#16213e"), text: element_text(color: "#eaeaea"), axis_text: element_text(color: "#a0a0a0"), panel_grid_minor: element_line(color: "#6e6e6e", width: 0.5), panel_grid_major: element_line(color: "white", width: 0.5), axis_line: element_line(color: "#ffffff", width: 1))' > examples/theme_dark.png
```

![Dark Theme](examples/theme_dark.png)

### Merged Themes

```bash
cat examples/financials.csv | gramgraph 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Merged Theme Example") | theme_minimal() | theme(plot_title: element_text(size: 20, face: "bold"))' > examples/theme_merged.png
```

![Merged Theme](examples/theme_merged.png)

### Bold Axis Labels

```bash
cat examples/financials.csv | gramgraph 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Bold Axis Labels") | theme(axis_text: element_text(face: "bold", size: 14))' > examples/axis_bold.png
```

![Bold Axis Labels](examples/axis_bold.png)

### Rotated X-Axis Labels

```bash
cat examples/financials.csv | gramgraph 'aes(x: quarter, y: amount, color: type) | bar(position: "dodge") | labs(title: "Rotated X-Axis Labels") | theme(axis_text: element_text(angle: 90, size: 12))' > examples/axis_rotated.png
```

![Rotated X-Axis Labels](examples/axis_rotated.png)

### Hidden Ticks

```bash
cat examples/timeseries.csv | gramgraph 'aes(x: time, y: value, color: series) | line() | labs(title: "Clean Look - No Ticks") | theme_minimal() | theme(axis_ticks: element_blank())' > examples/axis_no_ticks.png
```

![Hidden Ticks](examples/axis_no_ticks.png)

### Fully Styled Axes

```bash
cat examples/regional_sales.csv | gramgraph 'aes(x: region, y: sales, color: product) | bar(position: "dodge") | labs(title: "Fully Styled Axes") | theme(axis_text: element_text(face: "bold", angle: 90, color: "#2E86AB", size: 11), axis_line: element_line(color: "#333333", width: 2))' > examples/axis_styled.png
```

![Fully Styled Axes](examples/axis_styled.png)

### Variable Injection

Use `-D` / `--define` to inject variables into your DSL at runtime. Variables use the `$name` syntax.

```bash
cat examples/timeseries.csv | gramgraph 'aes(x: $xcol, y: $ycol, color: series) | line() | labs(title: $title)' -D xcol=time -D ycol=value -D title="Variable Injection Example" > examples/variable_aes.png
```

![Variable Injection](examples/variable_aes.png)

Variables work in aesthetics, geometries, and labels:

```bash
cat examples/demographics.csv | gramgraph 'aes(x: height, y: weight) | point(color: $color, size: $size) | labs(title: "Styled with Variables")' -D color=blue -D size=8 > examples/variable_geom.png
```

![Variable Geometry](examples/variable_geom.png)

## Installation

```bash
cargo install --path .
```
