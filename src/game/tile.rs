use macroquad::prelude::IVec2;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Tile {
    pub chunk_pos: IVec2,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub enum TileInfo {
    Sand,
}
