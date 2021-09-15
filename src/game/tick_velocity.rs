use std::ops::SubAssign;

use macroquad::prelude::{ivec2, IVec2};

use super::tile_move_direction::TileMoveDirection;

#[derive(Clone, Copy, Debug)]
pub struct TickVelocity {
    pub velocity: IVec2,
}

impl TickVelocity {
    pub fn is_zero(&self) -> bool {
        self.velocity.x == 0 && self.velocity.y == 0
    }

    pub fn moves(&self) -> u32 {
        (self.velocity.x.abs() + self.velocity.y.abs()) as u32
    }

    pub fn direction(&self) -> Option<TileMoveDirection> {
        let x = self.velocity.x;
        let y = self.velocity.y;
        if x == 0 && y == 0 {
            return None;
        }

        if self.velocity.x.abs() > self.velocity.y.abs() {
            // Prioritize horizontal move
            Some(ivec2(x.signum(), 0).into())
        } else {
            // Prioritize vertical move
            Some(ivec2(0, y.signum()).into())
        }
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
