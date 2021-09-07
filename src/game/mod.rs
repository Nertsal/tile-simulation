use macroquad::prelude::{is_mouse_button_down, ivec2, uvec2, IVec2, MouseButton};
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
pub mod tile;

use chunk::{tile_index_to_position, Chunk};
use renderer::Renderer;

use self::tile::Tile;

pub struct Game {
    chunks: HashMap<IVec2, Chunk>,
    renderer: Renderer,
    view_update: UpdateView,
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
        self.handle_input();
        self.renderer.update(delta_time);
    }

    pub fn fixed_update(&mut self, _delta_time: f32) {
        self.tick();
    }

    pub fn draw(&mut self) {
        self.renderer.draw(std::mem::take(&mut self.view_update));
    }

    fn handle_input(&mut self) {
        let selected_tile = if is_mouse_button_down(MouseButton::Left) {
            Some(Some(TileInfo::Sand))
        } else if is_mouse_button_down(MouseButton::Middle) {
            Some(Some(TileInfo::Barrier))
        } else if is_mouse_button_down(MouseButton::Right) {
            Some(None)
        } else {
            None
        };

        if let Some(selected_tile) = selected_tile {
            self.set_tile(self.mouse_over_tile(), selected_tile);
        }
    }

    fn set_tile(&mut self, tile: Tile, tile_info: Option<TileInfo>) {
        if let Some(chunk) = self.chunks.get_mut(&tile.chunk_pos) {
            chunk.set_tile(tile.index, tile_info.clone());
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
