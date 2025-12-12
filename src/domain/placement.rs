//! Placement specification for design positioning
//!
//! This mirrors the Go PlacementSpec for zero-drift compatibility between
//! preview mockups and actual printing.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Display template dimensions (for Cloudinary preview)
pub const DISPLAY_TEMPLATE_WIDTH: i32 = 1000;
pub const DISPLAY_TEMPLATE_HEIGHT: i32 = 1400;

/// Print template dimensions (for actual printing)
pub const PRINT_TEMPLATE_WIDTH: i32 = 1800;
pub const PRINT_TEMPLATE_HEIGHT: i32 = 2400;

/// Placement errors
#[derive(Debug, Error)]
pub enum PlacementError {
    #[error("Scale must be between 0.1 and 1.0, got {0}")]
    InvalidScale(f64),
    #[error("Design extends outside print area: left={0}, right={1}, print_width={2}")]
    OutOfBoundsHorizontal(i32, i32, i32),
    #[error("Design extends outside print area: top={0}, bottom={1}, print_height={2}")]
    OutOfBoundsVertical(i32, i32, i32),
}

/// Coordinate space for placement calculations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateSpace {
    Display,
    Print,
}

impl Default for CoordinateSpace {
    fn default() -> Self {
        CoordinateSpace::Print
    }
}

/// Placement type (front, back, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementType {
    Front,
    Back,
    SleeveLeft,
    SleeveRight,
}

impl Default for PlacementType {
    fn default() -> Self {
        PlacementType::Front
    }
}

/// Placement specification for design positioning
///
/// This struct defines where a design should be placed on a t-shirt template.
/// It uses a center-based coordinate system with scale and offset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementSpec {
    /// Scale factor (0.1 to 1.0) - percentage of print area width
    pub scale: f64,

    /// Horizontal offset from center in pixels
    pub offset_x: i32,

    /// Vertical offset from center in pixels (negative = up, positive = down)
    pub offset_y: i32,

    /// Placement type (front, back, etc.)
    #[serde(default)]
    pub placement: PlacementType,

    /// Print area width in pixels
    #[serde(default = "default_print_width")]
    pub print_area_width: i32,

    /// Print area height in pixels
    #[serde(default = "default_print_height")]
    pub print_area_height: i32,

    /// Coordinate space (display or print)
    #[serde(default)]
    pub coordinate_space: CoordinateSpace,
}

fn default_print_width() -> i32 { PRINT_TEMPLATE_WIDTH }
fn default_print_height() -> i32 { PRINT_TEMPLATE_HEIGHT }

impl PlacementSpec {
    /// Create a new placement specification
    pub fn new(scale: f64, offset_x: i32, offset_y: i32, placement: PlacementType) -> Self {
        PlacementSpec {
            scale,
            offset_x,
            offset_y,
            placement,
            print_area_width: PRINT_TEMPLATE_WIDTH,
            print_area_height: PRINT_TEMPLATE_HEIGHT,
            coordinate_space: CoordinateSpace::Print,
        }
    }

    /// Validate the placement specification
    pub fn validate(&self) -> Result<(), PlacementError> {
        // Validate scale
        if self.scale < 0.1 || self.scale > 1.0 {
            return Err(PlacementError::InvalidScale(self.scale));
        }

        // Calculate design dimensions
        let (design_width, design_height) = self.get_design_dimensions();

        // Calculate absolute position
        let (abs_x, abs_y) = self.get_absolute_position();

        // Check horizontal bounds
        let left_edge = abs_x;
        let right_edge = abs_x + design_width;
        if left_edge < 0 || right_edge > self.print_area_width {
            return Err(PlacementError::OutOfBoundsHorizontal(
                left_edge,
                right_edge,
                self.print_area_width,
            ));
        }

        // Check vertical bounds
        let top_edge = abs_y;
        let bottom_edge = abs_y + design_height;
        if top_edge < 0 || bottom_edge > self.print_area_height {
            return Err(PlacementError::OutOfBoundsVertical(
                top_edge,
                bottom_edge,
                self.print_area_height,
            ));
        }

        Ok(())
    }

    /// Get design dimensions based on scale and print area
    pub fn get_design_dimensions(&self) -> (i32, i32) {
        let width = (self.print_area_width as f64 * self.scale) as i32;
        let height = (self.print_area_height as f64 * self.scale) as i32;
        (width, height)
    }

    /// Get absolute position (top-left corner) of the design
    pub fn get_absolute_position(&self) -> (i32, i32) {
        let (design_width, design_height) = self.get_design_dimensions();

        // Center of print area
        let center_x = self.print_area_width / 2;
        let center_y = self.print_area_height / 2;

        // Design position (top-left corner)
        let abs_x = center_x + self.offset_x - (design_width / 2);
        let abs_y = center_y + self.offset_y - (design_height / 2);

        (abs_x, abs_y)
    }

    /// Convert to display space coordinates
    pub fn to_display_space(&self) -> PlacementSpec {
        if self.coordinate_space == CoordinateSpace::Display {
            return self.clone();
        }

        let scale_factor = DISPLAY_TEMPLATE_WIDTH as f64 / PRINT_TEMPLATE_WIDTH as f64;

        PlacementSpec {
            scale: self.scale,
            offset_x: (self.offset_x as f64 * scale_factor) as i32,
            offset_y: (self.offset_y as f64 * scale_factor) as i32,
            placement: self.placement.clone(),
            print_area_width: DISPLAY_TEMPLATE_WIDTH,
            print_area_height: DISPLAY_TEMPLATE_HEIGHT,
            coordinate_space: CoordinateSpace::Display,
        }
    }

    /// Convert to print space coordinates
    pub fn to_print_space(&self) -> PlacementSpec {
        if self.coordinate_space == CoordinateSpace::Print {
            return self.clone();
        }

        let scale_factor = PRINT_TEMPLATE_WIDTH as f64 / DISPLAY_TEMPLATE_WIDTH as f64;

        PlacementSpec {
            scale: self.scale,
            offset_x: (self.offset_x as f64 * scale_factor) as i32,
            offset_y: (self.offset_y as f64 * scale_factor) as i32,
            placement: self.placement.clone(),
            print_area_width: PRINT_TEMPLATE_WIDTH,
            print_area_height: PRINT_TEMPLATE_HEIGHT,
            coordinate_space: CoordinateSpace::Print,
        }
    }
}

impl Default for PlacementSpec {
    fn default() -> Self {
        PlacementSpec {
            scale: 0.5,
            offset_x: 0,
            offset_y: -50, // Slightly above center for chest placement
            placement: PlacementType::Front,
            print_area_width: PRINT_TEMPLATE_WIDTH,
            print_area_height: PRINT_TEMPLATE_HEIGHT,
            coordinate_space: CoordinateSpace::Print,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_placement() {
        let spec = PlacementSpec::default();
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_invalid_scale() {
        let mut spec = PlacementSpec::default();
        spec.scale = 1.5;
        assert!(matches!(spec.validate(), Err(PlacementError::InvalidScale(_))));
    }

    #[test]
    fn test_coordinate_conversion() {
        let print_spec = PlacementSpec::new(0.5, 100, -50, PlacementType::Front);
        let display_spec = print_spec.to_display_space();

        assert_eq!(display_spec.coordinate_space, CoordinateSpace::Display);
        assert_eq!(display_spec.print_area_width, DISPLAY_TEMPLATE_WIDTH);
    }
}
