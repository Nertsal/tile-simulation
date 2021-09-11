use std::ops::{AddAssign, Mul, SubAssign};

use macroquad::prelude::{ivec2, vec2, Vec2};

use super::{tick_velocity::TickVelocity, tile_move_direction::TileMoveDirection};

#[derive(Clone, Copy, Debug, Default)]
pub struct Velocity {
    pub velocity: Vec2,
}

impl Velocity {
    pub fn is_zero(&self) -> bool {
        self.velocity.x == 0.0 && self.velocity.y == 0.0
    }

    pub fn tick_velocity(&mut self) -> TickVelocity {
        let x = self.velocity.x.abs().floor() * self.velocity.x.signum();
        let y = self.velocity.y.abs().floor() * self.velocity.y.signum();
        self.velocity -= vec2(x, y);
        ivec2(x as i32, y as i32).into()
    }
}

impl From<Vec2> for Velocity {
    fn from(velocity: Vec2) -> Self {
        Self { velocity }
    }
}

impl AddAssign<Vec2> for Velocity {
    fn add_assign(&mut self, rhs: Vec2) {
        self.velocity += rhs;
    }
}

impl AddAssign for Velocity {
    fn add_assign(&mut self, rhs: Self) {
        self.velocity += rhs.velocity;
    }
}

impl SubAssign<TileMoveDirection> for Velocity {
    fn sub_assign(&mut self, rhs: TileMoveDirection) {
        self.velocity -= rhs.direction_f32();
    }
}

impl SubAssign<Vec2> for Velocity {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.velocity -= rhs;
    }
}

impl Mul<f32> for Velocity {
    type Output = Velocity;

    fn mul(self, rhs: f32) -> Self::Output {
        (self.velocity * rhs).into()
    }
}
