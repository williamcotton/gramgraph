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

echo "Done! All examples generated in the examples/ directory."
