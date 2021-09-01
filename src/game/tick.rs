use macroquad::prelude::{ivec2, IVec2};
use std::{collections::HashMap, sync::Mutex};

use crate::{
    constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y},
    game::calculator::Calculator,
};

use super::{
    chunk::{tile_index_to_position, DataArray},
    tile::TileInfo,
    Game,
};

impl Game {
    pub fn tick(&mut self) {
        // Calculate and perform movement
        let view_update = self.perform_tick();

        // Update view
        for (chunk_pos, update_view) in view_update {
            for (index, update) in update_view
                .into_iter()
                .enumerate()
                .filter_map(|(index, update)| update.map(|update| (index, update)))
            {
                let tile_pos = tile_index_to_position(index)
                    + chunk_pos * ivec2(CHUNK_SIZE_X as i32, CHUNK_SIZE_Y as i32);
                self.update_view.update_tile(tile_pos, update);
            }
        }
    }

    fn perform_tick(&mut self) -> HashMap<IVec2, DataArray<Option<Option<TileInfo>>>> {
        // Calculate chunks mostly in parallel (except for cross-chunk moves and updates)
        use rayon::prelude::*;
        let calculator = Mutex::new(Calculator::new(self.chunks.keys().copied()));

        let view_update = self
            .chunks
            .par_iter_mut()
            .map(|(&chunk_pos, chunk)| (chunk_pos, chunk.tick(&calculator)))
            .collect::<HashMap<_, _>>();

        view_update
    }
}
