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
        // Calculate chunks mostly in parallel
        let mut calculator = Calculator::new(self.chunks.keys().copied());
        calculator.tick(
            self.chunks
                .iter_mut()
                .map(|(&pos, chunk)| (pos, chunk))
                .collect(),
        )
    }
}
