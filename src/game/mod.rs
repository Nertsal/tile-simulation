use macroquad::prelude::{
    is_key_pressed, is_mouse_button_down, ivec2, uvec2, vec2, IVec2, KeyCode, MouseButton, Vec2,
};
use std::collections::HashMap;

use crate::{
    constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y},
    game::{chunk::tile_position_to_index, tile::TileInfo},
    update_view::UpdateView,
};

mod calculator;
mod chunk;
mod renderer;
mod tick;
mod tick_velocity;
pub mod tile;
mod tile_move_direction;
mod velocity;

use chunk::{tile_index_to_position, Chunk};
use renderer::Renderer;

use self::tile::{Tile, TileType};

pub struct Game {
    chunks: HashMap<IVec2, Chunk>,
    renderer: Renderer,
    view_update: UpdateView,
    selected_tile: Option<TileType>,
    last_mouse_pos: Vec2,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            chunks: {
                let mut chunks = HashMap::new();
                const CHUNKS: i32 = 1;
                for x in -CHUNKS..=CHUNKS {
                    for y in 0..=CHUNKS * 2 {
                        let pos = ivec2(x, y);
                        chunks.insert(pos, Chunk::empty(pos));
                    }
                }
                chunks
            },
            renderer: Renderer::new(),
            view_update: UpdateView::default(),
            selected_tile: None,
            last_mouse_pos: Vec2::ZERO,
        };

        game.view_update.update_view(
            game.chunks
                .iter()
                .map(|(&chunk_pos, chunk)| {
                    chunk.tiles().map(move |(index, tile)| {
                        (
                            tile_index_to_position(index)
                                + chunk_pos * ivec2(CHUNK_SIZE_X as i32, CHUNK_SIZE_Y as i32),
                            tile.clone(),
                        )
                    })
                })
                .flatten(),
        );

        game
    }

    pub fn update(&mut self, delta_time: f32) {
        self.handle_input(delta_time);
        self.renderer.update(delta_time);
    }

    pub fn fixed_update(&mut self, _delta_time: f32) {
        self.tick();
    }

    pub fn draw(&mut self) {
        self.renderer.draw(std::mem::take(&mut self.view_update));
    }

    fn handle_input(&mut self, delta_time: f32) {
        // Select tile
        if is_key_pressed(KeyCode::Key1) {
            self.selected_tile = Some(TileType::Barrier);
        } else if is_key_pressed(KeyCode::Key2) {
            self.selected_tile = Some(TileType::Sand);
        } else if is_key_pressed(KeyCode::Key3) {
            self.selected_tile = Some(TileType::Water);
        }

        // Place or delete tile
        let selected_tile = if is_mouse_button_down(MouseButton::Left) {
            Some(self.selected_tile.clone())
        } else if is_mouse_button_down(MouseButton::Right) {
            Some(None)
        } else {
            None
        };

        // Do thing
        let mouse_pos = self.renderer.mouse_world_pos();
        if let Some(selected_tile) = selected_tile {
            self.set_tile(
                self.mouse_over_tile(),
                selected_tile.map(|tile_type| {
                    let mouse_velocity = (mouse_pos - self.last_mouse_pos) / delta_time / 50.0;
                    let random_x = macroquad::rand::gen_range(0.01, 0.1)
                        * (macroquad::rand::gen_range(0, 2) * 2 - 1) as f32;
                    let extra_velocity = vec2(random_x, -1.0);
                    let velocity = mouse_velocity + extra_velocity;
                    TileInfo::new(tile_type, velocity.into())
                }),
            );
        }

        self.last_mouse_pos = mouse_pos;
    }

    fn set_tile(&mut self, tile: Tile, tile_info: Option<TileInfo>) {
        if let Some(chunk) = self.chunks.get_mut(&tile.chunk_pos) {
            for extra_update in chunk.set_tile(tile.index, tile_info.clone()) {
                if let Some(chunk) = self.chunks.get_mut(&extra_update.chunk_pos) {
                    chunk.queue_update(extra_update.index);
                }
            }
            self.view_update
                .update_tile(tile.global_position(), tile_info);
        }
    }

    fn mouse_over_tile(&self) -> Tile {
        let mouse_world_pos = self.renderer.mouse_world_pos();
        let tile_pos = ivec2(
            mouse_world_pos.x.floor() as i32,
            mouse_world_pos.y.floor() as i32,
        );

        let chunk_size = ivec2(CHUNK_SIZE_X as i32, CHUNK_SIZE_Y as i32);
        let mut chunk_pos = tile_pos / chunk_size;
        let mut tile_pos = tile_pos - chunk_pos * chunk_size;

        if tile_pos.x < 0 {
            chunk_pos += ivec2(-1, 0);
            tile_pos.x += CHUNK_SIZE_X as i32;
        }

        if tile_pos.y < 0 {
            chunk_pos += ivec2(0, -1);
            tile_pos.y += CHUNK_SIZE_Y as i32;
        }

        let tile_position = uvec2(tile_pos.x as u32, tile_pos.y as u32);

        let tile_index = tile_position_to_index(tile_position);
        Tile {
            chunk_pos,
            index: tile_index,
        }
    }
}
