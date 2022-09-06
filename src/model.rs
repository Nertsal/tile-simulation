use super::*;

mod data_array;
mod position;
mod tick;

use data_array::*;
pub use position::*;

const WIDTH: usize = 30;
const GRAVITY: Vec2<f32> = vec2(0.0, -0.5);

pub type Coord = R32;

pub struct Model {
    tiles: DataArray<Tile>,
    velocities: DataArray<Vec2<Coord>>,
    tick_velocities: DataArray<Vec2<Coord>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Tile {
    Empty,
    Sand,
}

impl Model {
    pub fn new() -> Self {
        let size = WIDTH * WIDTH;
        Self {
            tiles: DataArray::new(size, Tile::Empty),
            velocities: DataArray::new(size, Vec2::ZERO),
            tick_velocities: DataArray::new(size, Vec2::ZERO),
        }
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

    pub fn set_tile(&mut self, tile_pos: Position, velocity: Vec2<Coord>, new_tile: Tile) {
        let index = tile_pos.to_index(WIDTH);
        if let Some(tile) = self.tiles.get_mut(index) {
            *tile = new_tile;
            *self.velocities.get_mut(index).unwrap() = velocity;
        }
    }
}
