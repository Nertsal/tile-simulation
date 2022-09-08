use super::*;

mod data_array;
mod position;
mod tick;
mod tile;

use data_array::*;
pub use position::*;
pub use tile::*;

const WIDTH: usize = 30;
const GRAVITY: Vec2<f32> = vec2(0.0, -0.1);

pub type Coord = R32;

pub struct Model {
    tiles: DataArray<Tile>,
}

impl Model {
    pub fn new() -> Self {
        let size = WIDTH * WIDTH;
        Self {
            tiles: DataArray::new(size, Tile::empty()),
        }
    }

    pub fn get_tile(&self, position: Position) -> Option<&Tile> {
        self.tiles.get(position.to_index(self.get_size().x))
    }

    pub fn get_size(&self) -> Vec2<usize> {
        vec2(WIDTH, WIDTH)
    }

    pub fn get_tiles_count(&self) -> usize {
        let size = self.get_size();
        size.x * size.y
    }

    pub fn get_tiles(&self) -> impl Iterator<Item = (Position, &Tile)> {
        self.tiles
            .iter()
            .enumerate()
            .map(|(i, tile)| (Position::from_index(i, WIDTH), tile))
    }

    pub fn set_tile(&mut self, tile_pos: Position, new_tile: Tile) {
        let index = tile_pos.to_index(WIDTH);
        if let Some(tile) = self.tiles.get_mut(index) {
            *tile = new_tile;
        }
    }
}
