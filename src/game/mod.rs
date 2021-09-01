use macroquad::prelude::{ivec2, IVec2};
use std::collections::HashMap;

use crate::{
    constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y},
    game::tile::TileInfo,
    update_view::UpdateView,
};

mod calculator;
mod chunk;
mod renderer;
mod tick;
pub mod tile;

use chunk::{tile_index_to_position, Chunk};
use renderer::Renderer;

pub struct Game {
    chunks: HashMap<IVec2, Chunk>,
    renderer: Renderer,
    update_view: UpdateView,
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
                        let mut chunk = Chunk::empty(pos);
                        for tile in 0..5 {
                            chunk.set_tile(tile, Some(TileInfo::Sand));
                        }
                        chunks.insert(pos, chunk);
                    }
                }
                chunks
            },
            renderer: Renderer::new(),
            update_view: UpdateView::default(),
        };

        game.update_view.update_view(
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
        self.renderer.update(delta_time);
    }

    pub fn fixed_update(&mut self, _delta_time: f32) {
        self.tick();
    }

    pub fn draw(&mut self) {
        self.renderer.draw(std::mem::take(&mut self.update_view));
    }
}
