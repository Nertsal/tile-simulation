use std::ops::SubAssign;

use macroquad::prelude::{ivec2, IVec2};

use super::{tile_move_direction::TileMoveDirection, velocity::Velocity};

#[derive(Clone, Copy, Debug)]
pub struct TickVelocity {
    velocity: IVec2,
}

impl TickVelocity {
    pub fn is_zero(&self) -> bool {
        self.velocity.x == 0 && self.velocity.y == 0
    }

    pub fn moves(&self) -> u32 {
        (self.velocity.x.abs() + self.velocity.y.abs()) as u32
    }

    pub fn directions(&self) -> Vec<TileMoveDirection> {
        if self.velocity.x.abs() > self.velocity.y.abs() {
            // Prioritize horizontal move
            MoveDirections::new(*self)
                .push_horizontal()
                .push_vertical()
                .moves
        } else {
            // Prioritize vertical move
            MoveDirections::new(*self)
                .push_vertical()
                .push_horizontal()
                .moves
        }
    }
}

struct MoveDirections {
    velocity: TickVelocity,
    moves: Vec<TileMoveDirection>,
}

impl MoveDirections {
    fn new(velocity: TickVelocity) -> Self {
        Self {
            velocity,
            moves: Vec::new(),
        }
    }

    fn push_horizontal(mut self) -> Self {
        if self.velocity.velocity.x.abs() >= 1 {
            let x = self.velocity.velocity.x.signum() as i32;
            self.moves.push(ivec2(x, 0).into());
        }
        self
    }

    fn push_vertical(mut self) -> Self {
        if self.velocity.velocity.y.abs() >= 1 {
            let y = self.velocity.velocity.y.signum() as i32;
            self.moves.push(ivec2(0, y).into());
        }
        self
    }
}

impl From<IVec2> for TickVelocity {
    fn from(velocity: IVec2) -> Self {
        Self { velocity }
    }
}

impl Into<Velocity> for TickVelocity {
    fn into(self) -> Velocity {
        Velocity {
            velocity: self.velocity.as_f32(),
        }
    }
}

impl SubAssign<TileMoveDirection> for TickVelocity {
    fn sub_assign(&mut self, rhs: TileMoveDirection) {
        self.velocity -= rhs.direction();
    }
}
