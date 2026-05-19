//! Theme Resolution Engine
//!
//! Resolves hierarchical theme elements into concrete, fully-specified styles.
//! Implements ggplot2-style inheritance where child elements inherit from parents.
//!
//! Inheritance hierarchy:
//! ```text
//! text
//! ├── plot_title
//! └── axis_text
//!
//! rect
//! ├── plot_background
//! ├── panel_background
//! └── legend_background
//!
//! line
//! ├── axis_line
//! ├── axis_ticks
//! └── panel_grid_major
//!     └── panel_grid_minor
//! ```

use crate::parser::ast::{
    ElementLine, ElementRect, ElementText, LegendPosition, Theme, ThemeElement,
};
use plotters::style::RGBColor;

// === Resolved Types (no Options - fully concrete) ===

/// Fully resolved text style ready for rendering
#[derive(Debug, Clone)]
pub struct ResolvedText {
    pub family: String,
    pub color: RGBColor,
    pub size: f64,
    pub face: FontFace,
    pub angle: f64,
    pub hjust: f64,
    pub vjust: f64,
}

/// Fully resolved line style ready for rendering
#[derive(Debug, Clone)]
pub struct ResolvedLine {
    pub color: RGBColor,
    pub width: f64,
    pub linetype: LineType,
}

/// Fully resolved rectangle style ready for rendering
#[derive(Debug, Clone)]
pub struct ResolvedRect {
    pub fill: RGBColor,
    pub border_color: Option<RGBColor>,
    pub border_width: f64,
}

/// Font face variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontFace {
    Plain,
    Bold,
    Italic,
    BoldItalic,
}

/// Line type variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineType {
    Solid,
    Dashed,
    Dotted,
}

// === Fully Resolved Theme ===

/// Complete resolved theme with all elements fully specified
#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub plot_background: ResolvedRect,
    pub panel_background: ResolvedRect,
    pub plot_title: ResolvedText,
    pub panel_grid_major: Option<ResolvedLine>, // None if Blank
    pub panel_grid_minor: Option<ResolvedLine>, // None if Blank
    pub axis_text: ResolvedText,
    pub axis_line: Option<ResolvedLine>,  // None if Blank
    pub axis_ticks: Option<ResolvedLine>, // None if Blank
    pub legend_position: LegendPosition,
    pub legend_background: Option<ResolvedRect>, // None if Blank
    pub legend_text: ResolvedText,
    pub legend_margin: f64,
    pub legend_key_size: f64,
    /// True if user explicitly customized theme (vs using all defaults)
    pub has_customization: bool,
}

// === Default Values ===

impl Default for ResolvedText {
    fn default() -> Self {
        ResolvedText {
            family: "sans-serif".to_string(),
            color: RGBColor(0, 0, 0), // Black
            size: 12.0,
            face: FontFace::Plain,
            angle: 0.0,
            hjust: 0.5,
            vjust: 0.5,
        }
    }
}

impl Default for ResolvedLine {
    fn default() -> Self {
        ResolvedLine {
            color: RGBColor(0, 0, 0), // Black
            width: 1.0,
            linetype: LineType::Solid,
        }
    }
}

impl Default for ResolvedRect {
    fn default() -> Self {
        ResolvedRect {
            fill: RGBColor(255, 255, 255), // White
            border_color: None,
            border_width: 0.0,
        }
    }
}

// === Color Parsing ===

/// Parse a color string into RGBColor, supporting hex (#RRGGBB, #RGB) and named colors
pub fn parse_color(color_str: &str) -> Option<RGBColor> {
    let color_str = color_str.trim();

    // Hex color parsing
    if color_str.starts_with('#') {
        return parse_hex_color(color_str);
    }

    // Named colors (ggplot2-style gray scale + basic colors)
    match color_str.to_lowercase().as_str() {
        "white" => Some(RGBColor(255, 255, 255)),
        "black" => Some(RGBColor(0, 0, 0)),
        "red" => Some(RGBColor(255, 0, 0)),
        "green" => Some(RGBColor(0, 128, 0)),
        "blue" => Some(RGBColor(0, 0, 255)),
        "yellow" => Some(RGBColor(255, 255, 0)),
        "cyan" => Some(RGBColor(0, 255, 255)),
        "magenta" => Some(RGBColor(255, 0, 255)),
        "orange" => Some(RGBColor(255, 165, 0)),
        "purple" => Some(RGBColor(128, 0, 128)),
        "pink" => Some(RGBColor(255, 192, 203)),
        "brown" => Some(RGBColor(139, 69, 19)),
        "gray" | "grey" => Some(RGBColor(128, 128, 128)),
        "darkgray" | "darkgrey" => Some(RGBColor(64, 64, 64)),
        "lightgray" | "lightgrey" => Some(RGBColor(192, 192, 192)),
        // ggplot2-style grayscale (gray0 to gray100)
        s if s.starts_with("gray") || s.starts_with("grey") => {
            let num_str = &s[4..];
            if let Ok(n) = num_str.parse::<u8>() {
                // gray0 = black, gray100 = white
                // Use round() for correct conversion
                let v = (n as f64 * 2.55).round() as u8;
                Some(RGBColor(v, v, v))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse hex color (#RRGGBB or #RGB)
fn parse_hex_color(hex: &str) -> Option<RGBColor> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(RGBColor(r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some(RGBColor(r, g, b))
        }
        _ => None,
    }
}

/// Parse face string into FontFace
fn parse_face(face: &str) -> FontFace {
    match face.to_lowercase().as_str() {
        "bold" => FontFace::Bold,
        "italic" => FontFace::Italic,
        "bold.italic" | "bolditalic" => FontFace::BoldItalic,
        _ => FontFace::Plain,
    }
}

/// Parse linetype string into LineType
fn parse_linetype(linetype: &str) -> LineType {
    match linetype.to_lowercase().as_str() {
        "dashed" | "dash" => LineType::Dashed,
        "dotted" | "dot" => LineType::Dotted,
        _ => LineType::Solid,
    }
}

// === Resolution Logic ===

impl Theme {
    /// Resolve the theme into concrete styles using the inheritance hierarchy.
    ///
    /// Resolution order for each element:
    /// 1. Check the specific element (e.g., `axis_text`)
    /// 2. Check the parent element (e.g., `text`)
    /// 3. Use hardcoded default
    pub fn resolve(&self) -> ResolvedTheme {
        // Check if any theme customization was made (any field is not Inherit)
        let has_customization = self.line != ThemeElement::Inherit
            || self.rect != ThemeElement::Inherit
            || self.text != ThemeElement::Inherit
            || self.plot_background != ThemeElement::Inherit
            || self.plot_title != ThemeElement::Inherit
            || self.panel_background != ThemeElement::Inherit
            || self.panel_grid_major != ThemeElement::Inherit
            || self.panel_grid_minor != ThemeElement::Inherit
            || self.axis_text != ThemeElement::Inherit
            || self.axis_line != ThemeElement::Inherit
            || self.axis_ticks != ThemeElement::Inherit
            || self.legend_position.is_some()
            || self.legend_background != ThemeElement::Inherit
            || self.legend_text != ThemeElement::Inherit
            || self.legend_margin.is_some()
            || self.legend_key_size.is_some();

        // Resolve root element defaults
        let base_text = self.resolve_base_text();
        let base_line = self.resolve_base_line();
        let base_rect = self.resolve_base_rect();

        // Resolve specific elements with inheritance
        let plot_background = self.resolve_rect_element(&self.plot_background, &base_rect);
        let panel_background = self.resolve_rect_element(&self.panel_background, &base_rect);
        let plot_title = self.resolve_text_element(&self.plot_title, &base_text);
        let axis_text = self.resolve_text_element(&self.axis_text, &base_text);
        let legend_text = self.resolve_text_element(&self.legend_text, &base_text);

        // Resolve line elements (can be Blank)
        let axis_line = self.resolve_optional_line(&self.axis_line, &base_line);
        let axis_ticks = self.resolve_optional_line(&self.axis_ticks, &base_line);

        // Grid lines have their own inheritance: panel_grid_minor -> panel_grid_major -> line
        let panel_grid_major = self.resolve_optional_line(&self.panel_grid_major, &base_line);
        let panel_grid_minor = self.resolve_grid_minor(&panel_grid_major);
        let legend_background = self.resolve_legend_background(&panel_background, &legend_text);

        ResolvedTheme {
            plot_background,
            panel_background,
            plot_title,
            panel_grid_major,
            panel_grid_minor,
            axis_text,
            axis_line,
            axis_ticks,
            legend_position: self.legend_position.clone().unwrap_or_default(),
            legend_background,
            legend_text,
            legend_margin: self.legend_margin.unwrap_or(10.0),
            legend_key_size: self.legend_key_size.unwrap_or(30.0),
            has_customization,
        }
    }

    /// Resolve base text style from root `text` element
    fn resolve_base_text(&self) -> ResolvedText {
        let mut base = ResolvedText::default();
        if let ThemeElement::Text(t) = &self.text {
            apply_text_overrides(&mut base, t);
        }
        base
    }

    /// Resolve base line style from root `line` element
    fn resolve_base_line(&self) -> ResolvedLine {
        let mut base = ResolvedLine::default();
        if let ThemeElement::Line(l) = &self.line {
            apply_line_overrides(&mut base, l);
        }
        base
    }

    /// Resolve base rect style from root `rect` element
    fn resolve_base_rect(&self) -> ResolvedRect {
        let mut base = ResolvedRect::default();
        if let ThemeElement::Rect(r) = &self.rect {
            apply_rect_overrides(&mut base, r);
        }
        base
    }

    /// Resolve a text element, inheriting from base
    fn resolve_text_element(&self, element: &ThemeElement, base: &ResolvedText) -> ResolvedText {
        match element {
            ThemeElement::Text(t) => {
                let mut resolved = base.clone();
                apply_text_overrides(&mut resolved, t);
                resolved
            }
            ThemeElement::Inherit => base.clone(),
            _ => base.clone(), // Blank doesn't make sense for text, treat as inherit
        }
    }

    /// Resolve a rect element, inheriting from base
    fn resolve_rect_element(&self, element: &ThemeElement, base: &ResolvedRect) -> ResolvedRect {
        match element {
            ThemeElement::Rect(r) => {
                let mut resolved = base.clone();
                apply_rect_overrides(&mut resolved, r);
                resolved
            }
            ThemeElement::Inherit => base.clone(),
            _ => base.clone(), // Blank doesn't make sense for backgrounds, treat as inherit
        }
    }

    /// Resolve an optional line element (can be Blank)
    fn resolve_optional_line(
        &self,
        element: &ThemeElement,
        base: &ResolvedLine,
    ) -> Option<ResolvedLine> {
        match element {
            ThemeElement::Line(l) => {
                let mut resolved = base.clone();
                apply_line_overrides(&mut resolved, l);
                Some(resolved)
            }
            ThemeElement::Inherit => Some(base.clone()),
            ThemeElement::Blank => None,
            _ => Some(base.clone()),
        }
    }

    /// Resolve panel_grid_minor with special inheritance from panel_grid_major
    fn resolve_grid_minor(&self, major: &Option<ResolvedLine>) -> Option<ResolvedLine> {
        match &self.panel_grid_minor {
            ThemeElement::Blank => None,
            ThemeElement::Line(l) => {
                // Start from major grid or default
                let mut resolved = major.clone().unwrap_or_default();
                apply_line_overrides(&mut resolved, l);
                // Minor grid typically thinner
                if l.width.is_none() {
                    resolved.width = resolved.width * 0.5;
                }
                Some(resolved)
            }
            ThemeElement::Inherit => {
                // Inherit from major, but make thinner
                major.as_ref().map(|m| {
                    let mut resolved = m.clone();
                    resolved.width *= 0.5;
                    resolved
                })
            }
            _ => major.clone(),
        }
    }

    /// Resolve legend background. By default it follows the panel background
    /// and uses legend text color for the border.
    fn resolve_legend_background(
        &self,
        panel_background: &ResolvedRect,
        legend_text: &ResolvedText,
    ) -> Option<ResolvedRect> {
        match &self.legend_background {
            ThemeElement::Blank => None,
            ThemeElement::Rect(r) => {
                let mut resolved = ResolvedRect {
                    fill: panel_background.fill,
                    border_color: Some(legend_text.color),
                    border_width: 1.0,
                };
                apply_rect_overrides(&mut resolved, r);
                Some(resolved)
            }
            _ => Some(ResolvedRect {
                fill: panel_background.fill,
                border_color: Some(legend_text.color),
                border_width: 1.0,
            }),
        }
    }
}

// === Override Application ===

fn apply_text_overrides(resolved: &mut ResolvedText, element: &ElementText) {
    if let Some(ref family) = element.family {
        resolved.family = family.clone();
    }
    if let Some(ref color) = element.color {
        if let Some(c) = parse_color(color) {
            resolved.color = c;
        }
    }
    if let Some(size) = element.size {
        resolved.size = size;
    }
    if let Some(ref face) = element.face {
        resolved.face = parse_face(face);
    }
    if let Some(angle) = element.angle {
        resolved.angle = angle;
    }
    if let Some(hjust) = element.hjust {
        resolved.hjust = hjust;
    }
    if let Some(vjust) = element.vjust {
        resolved.vjust = vjust;
    }
}

fn apply_line_overrides(resolved: &mut ResolvedLine, element: &ElementLine) {
    if let Some(ref color) = element.color {
        if let Some(c) = parse_color(color) {
            resolved.color = c;
        }
    }
    if let Some(width) = element.width {
        resolved.width = width;
    }
    if let Some(ref linetype) = element.linetype {
        resolved.linetype = parse_linetype(linetype);
    }
}

fn apply_rect_overrides(resolved: &mut ResolvedRect, element: &ElementRect) {
    if let Some(ref fill) = element.fill {
        if let Some(c) = parse_color(fill) {
            resolved.fill = c;
        }
    }
    if let Some(ref color) = element.color {
        if let Some(c) = parse_color(color) {
            resolved.border_color = Some(c);
        }
    }
    if let Some(width) = element.width {
        resolved.border_width = width;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_color("#FF0000"), Some(RGBColor(255, 0, 0)));
        assert_eq!(parse_color("#00FF00"), Some(RGBColor(0, 255, 0)));
        assert_eq!(parse_color("#0000FF"), Some(RGBColor(0, 0, 255)));
        assert_eq!(parse_color("#F00"), Some(RGBColor(255, 0, 0)));
        assert_eq!(parse_color("#CCCCCC"), Some(RGBColor(204, 204, 204)));
    }

    #[test]
    fn test_parse_named_color() {
        assert_eq!(parse_color("white"), Some(RGBColor(255, 255, 255)));
        assert_eq!(parse_color("black"), Some(RGBColor(0, 0, 0)));
        assert_eq!(parse_color("red"), Some(RGBColor(255, 0, 0)));
    }

    #[test]
    fn test_parse_gray_scale() {
        assert_eq!(parse_color("gray0"), Some(RGBColor(0, 0, 0)));
        assert_eq!(parse_color("gray100"), Some(RGBColor(255, 255, 255)));
        assert_eq!(parse_color("gray50"), Some(RGBColor(127, 127, 127)));
        assert_eq!(parse_color("grey90"), Some(RGBColor(229, 229, 229)));
    }

    #[test]
    fn test_resolve_default_theme() {
        let theme = Theme::default();
        let resolved = theme.resolve();

        // Default should have white backgrounds
        assert_eq!(resolved.plot_background.fill, RGBColor(255, 255, 255));
        assert_eq!(resolved.panel_background.fill, RGBColor(255, 255, 255));

        // Default should have black text
        assert_eq!(resolved.axis_text.color, RGBColor(0, 0, 0));
    }

    #[test]
    fn test_resolve_with_blank_elements() {
        let mut theme = Theme::default();
        theme.axis_line = ThemeElement::Blank;
        theme.axis_ticks = ThemeElement::Blank;

        let resolved = theme.resolve();

        assert!(resolved.axis_line.is_none());
        assert!(resolved.axis_ticks.is_none());
    }

    #[test]
    fn test_resolve_with_custom_text() {
        let mut theme = Theme::default();
        theme.plot_title = ThemeElement::Text(ElementText {
            size: Some(24.0),
            face: Some("bold".to_string()),
            color: Some("#FF0000".to_string()),
            ..Default::default()
        });

        let resolved = theme.resolve();

        assert_eq!(resolved.plot_title.size, 24.0);
        assert_eq!(resolved.plot_title.face, FontFace::Bold);
        assert_eq!(resolved.plot_title.color, RGBColor(255, 0, 0));
    }

    #[test]
    fn test_inheritance_from_root() {
        let mut theme = Theme::default();
        // Set root text color
        theme.text = ThemeElement::Text(ElementText {
            color: Some("blue".to_string()),
            ..Default::default()
        });

        let resolved = theme.resolve();

        // axis_text should inherit the blue color from root text
        assert_eq!(resolved.axis_text.color, RGBColor(0, 0, 255));
        // plot_title should also inherit
        assert_eq!(resolved.plot_title.color, RGBColor(0, 0, 255));
    }
}
