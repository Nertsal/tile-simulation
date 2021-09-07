use macroquad::prelude::{ivec2, IVec2};

use crate::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y};

use super::chunk::tile_index_to_position;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Tile {
    pub chunk_pos: IVec2,
    pub index: usize,
}

impl Tile {
    pub fn global_position(&self) -> IVec2 {
        tile_index_to_position(self.index)
            + self.chunk_pos * ivec2(CHUNK_SIZE_X as i32, CHUNK_SIZE_Y as i32)
    }
}

#[derive(Clone, Debug)]
pub enum TileInfo {
    Barrier,
    Sand,
}

impl TileInfo {
    pub fn movement_directions(&self) -> Vec<IVec2> {
        match self {
            TileInfo::Barrier => vec![],
            TileInfo::Sand => vec![ivec2(0, -1), ivec2(-1, -1), ivec2(1, -1)],
        }
    }
}
