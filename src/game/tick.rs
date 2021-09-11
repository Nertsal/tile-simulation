use std::collections::HashMap;

use super::{
    calculator::{Calculator, ViewUpdates},
    tile::Tile,
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
                let tile = Tile { chunk_pos, index };
                self.view_update.update_tile(tile.global_position(), update);
            }
        }
    }

    fn perform_tick(&mut self) -> ViewUpdates {
        use rayon::prelude::*;

        let chunk_positions: Vec<_> = self.chunks.keys().copied().collect();

        // Collect chunks for update
        let mut chunks: HashMap<_, _> = self
            .chunks
            .iter_mut()
            .map(|(&pos, chunk)| (pos, chunk))
            .collect();

        // Prepare for tick (apply gravity)
        chunks
            .par_iter_mut()
            .for_each(|(_, chunk)| chunk.prepare_tick());

        // Calculate chunks mostly in parallel
        let mut view_update = HashMap::new();
        loop {
            let mut calculator = Calculator::new(chunk_positions.iter().copied());
            let done = calculator.step(&mut chunks, &mut view_update);
            if done {
                break;
            }
        }
        view_update
    }
}
