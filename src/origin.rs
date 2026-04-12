use bevy::{
    ecs::component::Component,
    math::{Vec2, Vec3},
};

/// Anchor point of an SVG's mesh relative to its `Transform.translation`.
/// Consumed at spawn time: the offset is baked into a per-entity mesh and the
/// component is then removed.
#[derive(Clone, Copy, Component, Debug, Default, PartialEq)]
pub enum Origin {
    /// Bottom left of the image or viewbox.
    BottomLeft,
    /// Bottom right of the image or viewbox.
    BottomRight,
    /// Center of the image or viewbox.
    Center,
    #[default]
    /// Top left of the image or viewbox, this is the default for a SVG.
    TopLeft,
    /// Top right of the image or viewbox.
    TopRight,
    /// Custom origin, top left is (0, 0), bottom right is (1, 1)
    Custom(Vec2),
}

impl Origin {
    /// Per-vertex offset that places this anchor at the entity's local origin.
    pub fn compute_translation(&self, size: Vec2) -> Vec3 {
        match self {
            Origin::BottomLeft => Vec3::new(0.0, size.y, 0.0),
            Origin::BottomRight => Vec3::new(-size.x, size.y, 0.0),
            Origin::Center => Vec3::new(-size.x * 0.5, size.y * 0.5, 0.0),
            Origin::TopLeft => Vec3::ZERO,
            Origin::TopRight => Vec3::new(-size.x, 0.0, 0.0),
            Origin::Custom(coord) => Vec3::new(-size.x * coord.x, size.y * coord.y, 0.0),
        }
    }
}
