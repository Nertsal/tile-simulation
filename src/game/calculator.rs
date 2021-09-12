use macroquad::prelude::IVec2;
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use super::{
    chunk::{
        data_array, default_data_array, Chunk, ChunkCalculation, DataArray, Dependencies, MoveInfo,
    },
    tile::{Tile, TileInfo},
};

type ChunkInformation<'a, 'b> = (
    &'b mut &'a mut Chunk,
    ChunkCalculation,
    Dependencies,
    Option<DataArray<bool>>, // Extra updates
    Option<DataArray<bool>>, // Cross-chunk moves
);

pub type ViewUpdates = HashMap<IVec2, DataArray<Option<Option<TileInfo>>>>;

pub struct Calculator {
    chunk_calculations: HashMap<IVec2, DataArray<MoveInfo>>,
    extra_updates: HashMap<IVec2, DataArray<bool>>,
    cross_moves: HashMap<IVec2, DataArray<bool>>,
    calculations: HashMap<IVec2, (ChunkCalculation, Dependencies)>,
    update_queue: HashSet<IVec2>,
}

impl Calculator {
    pub fn new(chunk_positions: impl Iterator<Item = IVec2>) -> Self {
        let mut update_queue = HashSet::new();
        let mut extra_updates = HashMap::new();
        let mut cross_moves = HashMap::new();
        let mut chunk_calculations = HashMap::new();
        for chunk_pos in chunk_positions {
            update_queue.insert(chunk_pos);
            extra_updates.insert(chunk_pos, data_array(false));
            cross_moves.insert(chunk_pos, default_data_array());
            chunk_calculations.insert(chunk_pos, data_array(MoveInfo::Unknown));
        }
        let calculations = HashMap::with_capacity(chunk_calculations.len());

        Self {
            extra_updates,
            cross_moves,
            calculations,
            chunk_calculations,
            update_queue,
        }
    }

    pub fn step(
        &mut self,
        chunks: &mut HashMap<IVec2, &mut Chunk>,
        view_update: &mut ViewUpdates,
    ) -> bool {
        let chunks_count = self.chunk_calculations.len();

        // Prepare chunks for calculation
        self.prepare_chunks(chunks.values_mut().collect());

        // Update chunks, while there are any updates queued
        while !self.update_queue.is_empty() {
            // Get chunks to update
            let update_queue = chunks
                .iter_mut()
                .filter_map(|(chunk_pos, chunk)| {
                    if self.update_queue.remove(chunk_pos) {
                        let (calculation, dependencies) =
                            self.calculations.remove(chunk_pos).unwrap();
                        let (extra_updates, cross_moves) = self.take_updates_moves(chunk_pos);
                        Some((chunk, calculation, dependencies, extra_updates, cross_moves))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            // Update chunks
            self.update_chunks(update_queue.into_par_iter());
        }

        // Collect collision information
        let mut collisions = self
            .calculations
            .par_iter()
            .map(|(&chunk_pos, (calculation, _))| {
                let chunk = chunks.get(&chunk_pos).unwrap();
                let mut collisions = Vec::with_capacity(calculation.collisions.len() * 2);
                let chunk_collisions = Mutex::new(&mut collisions);

                calculation
                    .collisions
                    .par_iter()
                    .for_each(|&(tile, other)| {
                        let result = chunk.tile_info[tile]
                            .as_ref()
                            .unwrap()
                            .collide(chunk.tile_info[other].as_ref().unwrap());
                        let mut chunk_collisions = chunk_collisions.lock().unwrap();
                        chunk_collisions.push((tile, result.0, result.1, result.2));
                        chunk_collisions.push((other, result.3, result.4, result.5));
                    });

                (chunk_pos, collisions)
            })
            .collect::<HashMap<_, _>>();

        // Collect cross-chunk collision information
        let chunk_collisions = Mutex::new(&mut collisions);
        self.calculations
            .par_iter()
            .for_each(|(&chunk_pos, (calculation, _))| {
                let chunk = chunks.get(&chunk_pos).unwrap();
                calculation
                    .cross_collisions
                    .par_iter()
                    .for_each(|&(index, tile)| {
                        if let Some(other_chunk) = chunks.get(&tile.chunk_pos) {
                            if let Some(other) = &other_chunk.tile_info[tile.index] {
                                let result =
                                    chunk.tile_info[index].as_ref().unwrap().collide(other);
                                let mut collisions = chunk_collisions.lock().unwrap();
                                collisions
                                    .get_mut(&chunk_pos)
                                    .unwrap()
                                    .push((index, result.0, result.1, result.2));
                                collisions
                                    .get_mut(&tile.chunk_pos)
                                    .unwrap()
                                    .push((tile.index, result.3, result.4, result.5));
                            }
                        }
                    });
            });

        // Update tile information according to collisions
        chunks.par_iter_mut().for_each(|(chunk_pos, chunk)| {
            if let Some(collisions) = collisions.get(chunk_pos) {
                for &(update_index, velocity, process_velocity, tick_velocity) in collisions {
                    let tile = chunk.tile_info[update_index].as_mut().unwrap();
                    tile.velocity = velocity;
                    tile.process_velocity = process_velocity;
                    tile.tick_velocity = tick_velocity;
                }
            }
        });

        // Collect movement
        let ((mut chunk_moves, mut view_updates), cross_moves) = chunks
            .par_iter_mut()
            .map(|(chunk_pos, chunk)| {
                let (local_moves, cross_moves, view_updates) =
                    chunk.collect_movement(&self.calculations[chunk_pos].0);
                (
                    ((*chunk_pos, local_moves), (*chunk_pos, view_updates)),
                    cross_moves,
                )
            })
            .collect::<((HashMap<_, _>, HashMap<_, _>), Vec<_>)>();

        // Collect cross-chunk moves
        for (cross_move, cross_tile) in cross_moves
            .into_iter()
            .map(|cross_moves| cross_moves.into_iter())
            .flatten()
        {
            let chunk_moves = chunk_moves.get_mut(&cross_move.chunk_pos).unwrap();
            chunk_moves[cross_move.index] = Some(cross_tile);
        }

        // Perform movement
        let chunk_moves = Mutex::new((chunk_moves, &mut view_updates));
        let chunks_done: usize = chunks
            .par_iter_mut()
            .map(|(chunk_pos, chunk)| {
                let mut chunk_moves = chunk_moves.lock().unwrap();
                let moves = chunk_moves.0.remove(chunk_pos).unwrap();
                if chunk.movement(moves, chunk_moves.1.get_mut(chunk_pos).unwrap()) {
                    1
                } else {
                    0
                }
            })
            .sum();

        // Collect view updates
        for chunk_pos in chunks.keys() {
            let view_update = view_update
                .entry(*chunk_pos)
                .or_insert_with(|| default_data_array());
            let updates = view_updates.remove(chunk_pos).unwrap();
            for (index, update) in updates
                .into_iter()
                .enumerate()
                .filter_map(|(index, update)| update.map(|update| (index, update)))
            {
                view_update[index] = Some(update);
            }
        }

        chunks_done == chunks_count
    }

    fn prepare_chunks<'a>(&mut self, update_queue: Vec<&mut &'a mut Chunk>) {
        // Prepare chunks for calculation
        let calculations = Mutex::new(&mut self.calculations);
        update_queue
            .into_par_iter()
            .map(|chunk| (chunk.chunk_pos, chunk.prepare_calculation()))
            .for_each(|(chunk_pos, calculation)| {
                calculations.lock().unwrap().insert(chunk_pos, calculation);
            });
    }

    fn update_chunks<'a: 'b, 'b>(
        &mut self,
        update_chunks: impl ParallelIterator<Item = ChunkInformation<'a, 'b>>,
    ) {
        // Update chunks in parallel
        let calculator = Mutex::new(self);
        update_chunks.for_each(
            |(chunk, mut calculation, mut dependencies, updates, cross_moves)| {
                // Calculation cycle is independent from other chunks
                let (chunk_updates, extra_updates, cross_moves) = chunk.calculation_cycle(
                    &mut calculation,
                    &mut dependencies,
                    updates,
                    cross_moves,
                );

                // Update information about chunk
                let mut calculator = calculator.lock().unwrap();
                calculator.update_information(
                    chunk.chunk_pos,
                    chunk_updates,
                    extra_updates,
                    cross_moves,
                    &mut dependencies,
                    &calculation.dependencies,
                );

                calculator
                    .calculations
                    .insert(chunk.chunk_pos, (calculation, dependencies));
            },
        );
    }

    fn update_information(
        &mut self,
        chunk_pos: IVec2,
        chunk_updates: DataArray<Option<MoveInfo>>,
        extra_updates: Vec<Tile>,
        cross_moves: HashSet<Tile>,
        dependencies: &mut Dependencies,
        tile_dependencies: &DataArray<Option<Tile>>,
    ) {
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
            if let Some(updates) = self.extra_updates.get_mut(&update_tile.chunk_pos) {
                updates[update_tile.index] = true;
                // Queue chunk update
                self.update_queue.insert(update_tile.chunk_pos);
            }
        }

        // Register cross moves
        for cross_tile in cross_moves {
            if let Some(cross_moves) = self.cross_moves.get_mut(&cross_tile.chunk_pos) {
                cross_moves[cross_tile.index] = true;
                // Queue chunk update
                self.update_queue.insert(cross_tile.chunk_pos);
            }
        }

        // Find or create chunk's calculation
        let tiles = self.chunk_calculations.get_mut(&chunk_pos).unwrap();

        // Update this chunk's calculation
        for (index, update) in chunk_updates
            .into_iter()
            .enumerate()
            .filter_map(|(index, update)| update.map(|update| (index, update)))
        {
            tiles[index] = update;
        }

        // Update dependencies
        let mut need_update = false;
        for (tile, depend_tile) in
            tile_dependencies
                .iter()
                .enumerate()
                .filter_map(|(tile, depend_tile)| {
                    depend_tile.as_ref().map(|depend_tile| (tile, depend_tile))
                })
        {
            // Only update those that are still unknown
            let move_info = dependencies.get_mut(depend_tile).unwrap();
            if let MoveInfo::Unknown = move_info {
                // Look for evaluated chunks
                *move_info = match self.chunk_calculations.get(&depend_tile.chunk_pos) {
                    Some(calculation) => {
                        // If dependent tile depends on current tile, then movement is not allowed
                        match self.calculations.get(&depend_tile.chunk_pos) {
                            Some((calculation, _))
                                if calculation.dependencies[depend_tile.index]
                                    == Some(Tile {
                                        chunk_pos,
                                        index: tile,
                                    }) =>
                            {
                                MoveInfo::Recursive
                            }
                            _ => calculation[depend_tile.index],
                        }
                    }
                    None => MoveInfo::Impossible,
                };
                need_update = true;
            }
        }

        // If need_update, then queue update
        if need_update {
            self.update_queue.insert(chunk_pos);
        }
    }

    fn take_updates_moves(
        &mut self,
        chunk_pos: &IVec2,
    ) -> (Option<DataArray<bool>>, Option<DataArray<bool>>) {
        (
            self.extra_updates
                .get_mut(&chunk_pos)
                .map(|extra_updates| std::mem::replace(extra_updates, data_array(false))),
            self.cross_moves
                .get_mut(&chunk_pos)
                .map(|cross_moves| std::mem::replace(cross_moves, default_data_array())),
        )
    }
}
