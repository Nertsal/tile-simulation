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
    /// How much energy is preserved when bounced from a static tile.
    pub bounciness: R32,
}

/// Purely decorative information.
#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Empty,
    Barrier,
    Water,
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
                bounciness: R32::ZERO,
            },
        )
    }
}
