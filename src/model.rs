use super::*;

mod data_array;
mod position;
mod tick;

use data_array::*;
pub use position::*;

const WIDTH: usize = 10;

pub struct Model {
    tiles: DataArray<Tile>,
}

#[derive(Debug, Clone)]
pub enum Tile {
    Empty,
    Sand,
}

impl Model {
    pub fn new() -> Self {
        Self {
            tiles: DataArray::new(100, Tile::Empty),
        }
    }

    pub fn get_size(&self) -> Vec2<usize> {
        vec2(WIDTH, WIDTH)
    }

    pub fn get_tiles(&self) -> impl Iterator<Item = (Position, &Tile)> {
        self.tiles
            .iter()
            .enumerate()
            .map(|(i, tile)| (Position::from_index(i, WIDTH), tile))
    }

    pub fn set_tile(&mut self, tile_pos: Position, new_tile: Tile) {
        if let Some(tile) = self.tiles.get_mut(tile_pos.index(WIDTH)) {
            *tile = new_tile
        }
    }
}
