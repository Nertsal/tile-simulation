use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub tile_type: TileType,
    pub velocity: Vec2<Coord>,
    pub tick_velocity: Vec2<Coord>,
}

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Empty,
    Sand,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            velocity: Vec2::ZERO,
            tick_velocity: Vec2::ZERO,
        }
    }

    pub fn empty() -> Self {
        Self::new(TileType::Empty)
    }
}
