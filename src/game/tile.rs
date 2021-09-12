use macroquad::prelude::{ivec2, IVec2, Vec2};

use crate::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y, TICK_GRAVITY};

use super::{chunk::tile_index_to_position, tick_velocity::TickVelocity, velocity::Velocity};

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
pub struct TileInfo {
    pub mass: f32,
    pub gravity_scale: Vec2,
    pub velocity: Velocity,
    pub process_velocity: Velocity,
    pub tick_velocity: TickVelocity,
    pub tile_type: TileType,
}

impl TileInfo {
    pub fn prepare_tick(&mut self) {
        self.velocity += self.gravity_scale * TICK_GRAVITY;
        self.process_velocity += self.velocity;
        self.tick_velocity = self.process_velocity.tick_velocity();
    }

    pub fn lazy(&mut self) {
        self.velocity = (self.gravity_scale * TICK_GRAVITY).into();
    }
}

#[derive(Clone, Debug)]
pub enum TileType {
    Barrier,
    Sand,
    Water,
}
