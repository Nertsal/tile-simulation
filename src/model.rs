use super::*;

const WIDTH: usize = 10;

pub struct Model {
    tiles: DataArray2d<Tile>,
}

#[derive(Debug, Clone)]
pub enum Tile {
    Empty,
    Sand,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    // TODO: chunk position
    pub position: Vec2<usize>,
}

impl Position {
    pub fn from_index(index: usize, width: usize) -> Self {
        Self {
            position: vec2(index % width, index / width),
        }
    }

    pub fn index(self, width: usize) -> usize {
        self.position.x + self.position.y * width
    }
}

pub struct DataArray2d<T> {
    inner: Vec<T>,
}

impl<T> DataArray2d<T> {
    pub fn new(size: usize, default_element: T) -> Self
    where
        T: Clone,
    {
        Self {
            inner: vec![default_element; size],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner.iter_mut()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }
}

impl Model {
    pub fn new() -> Self {
        Self {
            tiles: DataArray2d::new(100, Tile::Empty),
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
