use std::ops::SubAssign;

use macroquad::prelude::{ivec2, IVec2};

use super::tile_move_direction::TileMoveDirection;

#[derive(Clone, Copy, Debug)]
pub struct TickVelocity {
    velocity: IVec2,
}

impl TickVelocity {
    pub fn is_zero(&self) -> bool {
        self.velocity.x == 0 && self.velocity.y == 0
    }

    pub fn directions(&self) -> Vec<TileMoveDirection> {
        let mut moves = Vec::new();

        // Get x component
        let x = if self.velocity.x.abs() >= 1 {
            let x = self.velocity.x.signum() as i32;
            // Push horizontal move
            moves.push(ivec2(x, 0).into());
            x
        } else {
            0
        };

        // Get y component
        let y = if self.velocity.y.abs() >= 1 {
            let y = self.velocity.y.signum() as i32;
            // Push vertical move
            moves.push(ivec2(0, y).into());
            y
        } else {
            0
        };

        // Push diagonal move
        if x != 0 && y != 0 {
            moves.push(ivec2(x, y).into());
        }

        moves
    }
}

impl From<IVec2> for TickVelocity {
    fn from(velocity: IVec2) -> Self {
        Self { velocity }
    }
}

impl SubAssign<TileMoveDirection> for TickVelocity {
    fn sub_assign(&mut self, rhs: TileMoveDirection) {
        self.velocity -= rhs.direction();
    }
}
