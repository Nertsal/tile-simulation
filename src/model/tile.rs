use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub tile_type: TileType,
    /// Physical properties.
    pub physics: TilePhysics,
    pub velocity: Vec2<Coord>,
    pub tick_velocity: Vec2<Coord>,
}

#[derive(Debug, Clone, Copy)]
pub struct TilePhysics {
    /// Indicates that the tile cannot move.
    pub is_static: bool,
    pub gravity_scale: R32,
    /// How much energy is preserved when bounced from a static tile.
    pub bounciness: R32,
    /// Impulse split coefficient determines how impulse gets redirected.
    pub impulse_split: R32,
}

/// Purely decorative information.
#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Empty,
    Barrier,
    Water,
    Steam,
}

impl Tile {
    pub fn new(tile_type: TileType, physics: TilePhysics) -> Self {
        Self {
            tile_type,
            physics,
            velocity: Vec2::ZERO,
            tick_velocity: Vec2::ZERO,
        }
    }

    pub fn empty() -> Self {
        Self::new(
            TileType::Empty,
            TilePhysics {
                is_static: false,
                gravity_scale: r32(0.0),
                bounciness: R32::ZERO,
                impulse_split: R32::ZERO,
            },
        )
    }
}
