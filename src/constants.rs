use crate::FIXED_DELTA_TIME;

pub const CHUNK_SIZE_X: usize = 50;
pub const CHUNK_SIZE_Y: usize = 50;
pub const CHUNK_SIZE: usize = CHUNK_SIZE_X * CHUNK_SIZE_Y;

const GRAVITY: f32 = 9.8;
pub const TICK_GRAVITY: f32 = GRAVITY * FIXED_DELTA_TIME;

pub const DRAG: f32 = 0.01;
