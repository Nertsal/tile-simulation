use macroquad::prelude::ivec2;

use super::tile_move_direction::TileMoveDirection;

pub trait TileMove {
    fn to_direction(&self) -> TileMoveDirection;

    fn opposite(&self) -> Self;
}

#[derive(Clone, Copy, Debug)]
pub enum HorizontalMove {
    Left,
    Right,
}

impl TileMove for HorizontalMove {
    fn to_direction(&self) -> TileMoveDirection {
        match self {
            Self::Left => ivec2(-1, 0).into(),
            Self::Right => ivec2(1, 0).into(),
        }
    }

    fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl HorizontalMove {
    pub fn from_tile_move(tile_move: TileMoveDirection) -> Option<Self> {
        let direction = tile_move.direction();
        if direction == ivec2(-1, 0) {
            Some(Self::Left)
        } else if direction == ivec2(1, 0) {
            Some(Self::Right)
        } else {
            None
        }
    }
}
