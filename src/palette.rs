// Color and size palettes for data-driven aesthetics

use std::collections::HashMap;

/// Color palette for categorical data
pub struct ColorPalette {
    colors: Vec<String>,
}

impl ColorPalette {
    /// Create a Category10 color palette (D3-inspired)
    /// Colors: blue, orange, green, red, purple, brown, pink, gray, olive, cyan
    pub fn category10() -> Self {
        ColorPalette {
            colors: vec![
                "blue".to_string(),
                "orange".to_string(),
                "green".to_string(),
                "red".to_string(),
                "purple".to_string(),
                "brown".to_string(),
                "pink".to_string(),
                "gray".to_string(),
                "olive".to_string(),
                "cyan".to_string(),
            ],
        }
    }

    /// Get color for a specific index (wraps around if index > palette size)
    pub fn get_color(&self, index: usize) -> String {
        self.colors[index % self.colors.len()].clone()
    }

    /// Assign colors to a list of group keys
    /// Returns a HashMap mapping each group key to its assigned color
    pub fn assign_colors(&self, group_keys: &[String]) -> HashMap<String, String> {
        group_keys
            .iter()
            .enumerate()
            .map(|(i, key)| (key.clone(), self.get_color(i)))
            .collect()
    }
}

/// Size palette for categorical or continuous size mapping
pub struct SizePalette {
    min_size: f64,
    max_size: f64,
}

impl SizePalette {
    /// Create a new size palette with min and max sizes
    pub fn new(min_size: f64, max_size: f64) -> Self {
        SizePalette { min_size, max_size }
    }

    /// Default size palette (3.0 to 15.0)
    pub fn default_range() -> Self {
        SizePalette::new(3.0, 15.0)
    }

    /// Assign discrete sizes to a list of group keys
    /// Sizes are evenly distributed between min and max
    pub fn assign_sizes(&self, group_keys: &[String]) -> HashMap<String, f64> {
        let num_groups = group_keys.len();
        if num_groups == 0 {
            return HashMap::new();
        }

        if num_groups == 1 {
            // Single group gets middle size
            let mid_size = (self.min_size + self.max_size) / 2.0;
            return vec![(group_keys[0].clone(), mid_size)]
                .into_iter()
                .collect();
        }

        // Multiple groups: distribute evenly
        group_keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let fraction = i as f64 / (num_groups - 1) as f64;
                let size = self.min_size + (self.max_size - self.min_size) * fraction;
                (key.clone(), size)
            })
            .collect()
    }
}

/// Shape palette for categorical shape mapping
pub struct ShapePalette {
    shapes: Vec<String>,
}

impl ShapePalette {
    /// Create a palette with common shapes
    pub fn default_shapes() -> Self {
        ShapePalette {
            shapes: vec![
                "circle".to_string(),
                "square".to_string(),
                "triangle".to_string(),
                "diamond".to_string(),
                "cross".to_string(),
                "star".to_string(),
            ],
        }
    }

    /// Get shape for a specific index (wraps around)
    pub fn get_shape(&self, index: usize) -> String {
        self.shapes[index % self.shapes.len()].clone()
    }

    /// Assign shapes to a list of group keys
    pub fn assign_shapes(&self, group_keys: &[String]) -> HashMap<String, String> {
        group_keys
            .iter()
            .enumerate()
            .map(|(i, key)| (key.clone(), self.get_shape(i)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_palette_category10() {
        let palette = ColorPalette::category10();
        assert_eq!(palette.get_color(0), "blue");
        assert_eq!(palette.get_color(1), "orange");
        assert_eq!(palette.get_color(9), "cyan");
        // Test wrapping
        assert_eq!(palette.get_color(10), "blue");
        assert_eq!(palette.get_color(11), "orange");
    }

    #[test]
    fn test_color_palette_assign_colors() {
        let palette = ColorPalette::category10();
        let groups = vec!["North".to_string(), "South".to_string(), "East".to_string()];
        let colors = palette.assign_colors(&groups);

        assert_eq!(colors.get("North"), Some(&"blue".to_string()));
        assert_eq!(colors.get("South"), Some(&"orange".to_string()));
        assert_eq!(colors.get("East"), Some(&"green".to_string()));
        assert_eq!(colors.len(), 3);
    }

    #[test]
    fn test_size_palette_default_range() {
        let palette = SizePalette::default_range();
        assert_eq!(palette.min_size, 3.0);
        assert_eq!(palette.max_size, 15.0);
    }

    #[test]
    fn test_size_palette_assign_sizes_single_group() {
        let palette = SizePalette::new(5.0, 15.0);
        let groups = vec!["A".to_string()];
        let sizes = palette.assign_sizes(&groups);

        assert_eq!(sizes.get("A"), Some(&10.0)); // Middle: (5+15)/2 = 10
    }

    #[test]
    fn test_size_palette_assign_sizes_two_groups() {
        let palette = SizePalette::new(5.0, 15.0);
        let groups = vec!["A".to_string(), "B".to_string()];
        let sizes = palette.assign_sizes(&groups);

        assert_eq!(sizes.get("A"), Some(&5.0));  // Min
        assert_eq!(sizes.get("B"), Some(&15.0)); // Max
    }

    #[test]
    fn test_size_palette_assign_sizes_three_groups() {
        let palette = SizePalette::new(5.0, 15.0);
        let groups = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let sizes = palette.assign_sizes(&groups);

        assert_eq!(sizes.get("A"), Some(&5.0));  // Min
        assert_eq!(sizes.get("B"), Some(&10.0)); // Middle
        assert_eq!(sizes.get("C"), Some(&15.0)); // Max
    }

    #[test]
    fn test_size_palette_assign_sizes_empty() {
        let palette = SizePalette::new(5.0, 15.0);
        let groups: Vec<String> = vec![];
        let sizes = palette.assign_sizes(&groups);

        assert_eq!(sizes.len(), 0);
    }

    #[test]
    fn test_shape_palette_default_shapes() {
        let palette = ShapePalette::default_shapes();
        assert_eq!(palette.get_shape(0), "circle");
        assert_eq!(palette.get_shape(1), "square");
        assert_eq!(palette.get_shape(5), "star");
        // Test wrapping
        assert_eq!(palette.get_shape(6), "circle");
    }

    #[test]
    fn test_shape_palette_assign_shapes() {
        let palette = ShapePalette::default_shapes();
        let groups = vec!["A".to_string(), "B".to_string()];
        let shapes = palette.assign_shapes(&groups);

        assert_eq!(shapes.get("A"), Some(&"circle".to_string()));
        assert_eq!(shapes.get("B"), Some(&"square".to_string()));
    }
}
