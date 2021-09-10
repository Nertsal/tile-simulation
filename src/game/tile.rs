use macroquad::prelude::{ivec2, IVec2};

use crate::{
    constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y},
    game::tile_move::TileMove,
};

use super::{
    chunk::tile_index_to_position, tile_move::HorizontalMove,
    tile_move_direction::TileMoveDirection,
};

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
    Water { priority: HorizontalMove },
}

impl TileInfo {
    pub fn register_move(&mut self, tile_move: TileMoveDirection) {
        match self {
            Self::Water { priority } => {
                if let Some(hor_move) = HorizontalMove::from_tile_move(tile_move) {
                    *priority = hor_move;
                }
            }
            _ => (),
        }
    }

    pub fn movement_directions(&self) -> Vec<TileMoveDirection> {
        match self {
            TileInfo::Barrier => vec![],
            TileInfo::Sand => vec![
                ivec2(0, -1).into(),
                ivec2(-1, -1).into(),
                ivec2(1, -1).into(),
            ],
            TileInfo::Water { priority } => vec![
                ivec2(0, -1).into(),
                ivec2(-1, -1).into(),
                ivec2(1, -1).into(),
                priority.to_direction(),
                priority.opposite().to_direction(),
            ],
        }
    }
}
