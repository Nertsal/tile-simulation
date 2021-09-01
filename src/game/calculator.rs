use macroquad::prelude::IVec2;
use std::collections::{HashMap, HashSet};

use super::{
    chunk::{data_array, default_data_array, DataArray, MoveInfo},
    tile::{Tile, TileInfo},
};

pub struct Calculator {
    calculations: HashMap<IVec2, DataArray<MoveInfo>>,
    extra_updates: HashMap<IVec2, DataArray<bool>>,
    cross_moves: HashMap<IVec2, DataArray<Option<TileInfo>>>,
    chunks: usize,
    updated: HashSet<IVec2>,
}

impl Calculator {
    pub fn new(chunk_positions: impl Iterator<Item = IVec2>) -> Self {
        let calculations = chunk_positions
            .map(|pos| (pos, data_array(MoveInfo::Unknown)))
            .collect::<HashMap<_, _>>();
        Self {
            extra_updates: HashMap::new(),
            cross_moves: HashMap::new(),
            chunks: calculations.len(),
            updated: HashSet::new(),
            calculations,
        }
    }

    pub fn is_done(&self) -> bool {
        self.updated.len() == self.chunks
    }

    pub fn update(
        &mut self,
        chunk_pos: IVec2,
        chunk_updates: DataArray<Option<MoveInfo>>,
        extra_updates: Vec<Tile>,
        cross_moves: HashMap<Tile, TileInfo>,
        dependencies: &mut HashMap<Tile, MoveInfo>,
    ) -> (Option<DataArray<bool>>, Option<DataArray<Option<TileInfo>>>) {
        // Assume this chunk has not finished updating
        self.updated.remove(&chunk_pos);

        // Queue updates for other chunks
        for update_tile in
            extra_updates
                .iter()
                .chain(
                    dependencies
                        .iter()
                        .filter_map(|(tile, move_info)| match move_info {
                            MoveInfo::Unknown => Some(tile),
                            _ => None,
                        }),
                )
        {
            let updates = self
                .extra_updates
                .entry(update_tile.chunk_pos)
                .or_insert_with(|| data_array(false));
            updates[update_tile.index] = true;
        }

        // Register cross moves
        for (cross_tile, cross_tile_info) in cross_moves {
            let cross_moves = self
                .cross_moves
                .entry(cross_tile.chunk_pos)
                .or_insert_with(|| default_data_array());
            cross_moves[cross_tile.index] = Some(cross_tile_info);
        }

        // Find or create chunk's calculation
        let tiles = self.calculations.get_mut(&chunk_pos).unwrap();

        // Update this chunk's calculation
        for (index, update) in chunk_updates
            .into_iter()
            .enumerate()
            .filter_map(|(index, update)| update.map(|update| (index, update)))
        {
            tiles[index] = update;
        }

        // Update dependencies
        let mut updated = false;
        for (tile, move_info) in dependencies {
            if let MoveInfo::Unknown = *move_info {
                *move_info = match self.calculations.get(&tile.chunk_pos) {
                    Some(calculation) => calculation[tile.index],
                    None => MoveInfo::Impossible,
                };
                updated = true;
            }
        }

        // This chunk has not received new updates
        if !updated {
            self.updated.insert(chunk_pos);
        }

        // Return updates from other chunks
        (
            self.extra_updates.remove(&chunk_pos),
            self.cross_moves.remove(&chunk_pos),
        )
    }
}
