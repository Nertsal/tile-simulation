use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub tile_type: TileType,
    /// Indicates that the tile cannot move.
    pub is_static: bool,
    pub velocity: Vec2<Coord>,
    pub tick_velocity: Vec2<Coord>,
}

/// A purely decorative information.
#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Empty,
    Barrier,
    Sand,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            is_static: false,
            velocity: Vec2::ZERO,
            tick_velocity: Vec2::ZERO,
        }
    }

    pub fn new_static(tile_type: TileType) -> Self {
        let mut tile = Self::new(tile_type);
        tile.is_static = true;
        tile
    }

    pub fn empty() -> Self {
        Self::new(TileType::Empty)
    }
}
