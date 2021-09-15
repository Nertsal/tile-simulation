use macroquad::prelude::{ivec2, vec2, IVec2, Vec2};

use crate::constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y, DRAG, TICK_GRAVITY};

use super::{
    chunk::tile_index_to_position, tick_velocity::TickVelocity,
    tile_move_direction::TileMoveDirection, velocity::Velocity,
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
pub struct TileInfo {
    pub mass: f32,
    pub gravity_scale: Vec2,
    pub velocity: Velocity,
    pub process_velocity: Velocity,
    pub tick_moves: u32,
    pub tick_velocity: TickVelocity,
    pub tile_type: TileType,
}

impl TileInfo {
    pub fn new(tile_type: TileType, velocity: Velocity) -> Self {
        Self {
            mass: 1.0,
            gravity_scale: vec2(0.0, -1.0),
            velocity,
            process_velocity: Vec2::ZERO.into(),
            tick_velocity: IVec2::ZERO.into(),
            tick_moves: 0,
            tile_type,
        }
    }

    pub fn gravity(&self) -> Vec2 {
        self.gravity_scale * TICK_GRAVITY
    }

    pub fn prepare_tick(&mut self) {
        self.velocity += self.gravity();
        self.velocity *= 1.0 - DRAG;
        self.process_velocity += self.velocity;
        self.tick_velocity = self.process_velocity.tick_velocity();
        self.tick_moves = self.tick_velocity.moves();
    }

    pub fn reset_velocity(&mut self, direction: TileMoveDirection) {
        let direction = direction.direction().as_f32();
        let projection = direction * direction.dot(self.velocity.velocity);
        self.velocity -= projection;

        self.process_velocity += Velocity::from(self.tick_velocity);
        let projection = direction * direction.dot(self.process_velocity.velocity);
        self.process_velocity -= projection;

        self.tick_velocity = self.process_velocity.tick_velocity();
        self.tick_moves = self.tick_velocity.moves();
    }

    pub fn lazy(&mut self) {
        self.velocity = (self.gravity_scale * TICK_GRAVITY).into();
    }

    pub fn collide(
        &self,
        other: &Self,
    ) -> (
        Velocity,
        Velocity,
        TickVelocity,
        Velocity,
        Velocity,
        TickVelocity,
    ) {
        let mut tile_velocity = self.velocity;
        let mass_tile = self.mass;

        let mut other_velocity = other.velocity;
        let mass_other = other.mass;

        let relative_velocity = tile_velocity - other_velocity;

        let total_mass = mass_tile + mass_other;
        let coef_tile = mass_tile / total_mass;
        let coef_other = mass_other / total_mass;

        // Update velocity
        other_velocity += relative_velocity * coef_tile;
        tile_velocity -= relative_velocity * coef_other;

        // Update tick and process velocity
        let mut tile_process_velocity = self.process_velocity;
        let mut tile_tick_velocity = self.tick_velocity;
        tile_process_velocity += Velocity::from(tile_tick_velocity) * coef_tile;
        tile_tick_velocity = tile_process_velocity.tick_velocity();

        let mut other_process_velocity = other.process_velocity;
        let mut other_tick_velocity = other.tick_velocity;
        other_process_velocity += Velocity::from(other_tick_velocity) * coef_other;
        other_tick_velocity = other_process_velocity.tick_velocity();

        (
            tile_velocity,
            tile_process_velocity,
            tile_tick_velocity,
            other_velocity,
            other_process_velocity,
            other_tick_velocity,
        )
    }
}

#[derive(Clone, Debug)]
pub enum TileType {
    Barrier,
    Sand,
    Water,
}
