use std::collections::{HashMap, HashSet};

use macroquad::prelude::{ivec2, uvec2, IVec2, UVec2};

use crate::constants::{CHUNK_SIZE, CHUNK_SIZE_X, CHUNK_SIZE_Y};

use super::{
    tile::{Tile, TileInfo},
    tile_move_direction::TileMoveDirection,
};

pub type Dependencies = HashMap<Tile, MoveInfo>;

pub type DataArray<T> = Vec<T>;

// TODO: trait DataArrayTrait

pub fn data_array<T: Copy>(default_value: T) -> DataArray<T> {
    let mut data_array = Vec::with_capacity(CHUNK_SIZE);
    for _ in 0..CHUNK_SIZE {
        data_array.push(default_value);
    }
    data_array
}

pub fn default_data_array<T: Default>() -> DataArray<T> {
    let mut data_array = Vec::with_capacity(CHUNK_SIZE);
    for _ in 0..CHUNK_SIZE {
        data_array.push(T::default());
    }
    data_array
}

pub fn tile_index_to_position(tile_index: usize) -> IVec2 {
    let y = tile_index / CHUNK_SIZE_X;
    assert!(y < CHUNK_SIZE_Y);
    ivec2(tile_index as i32 % CHUNK_SIZE_X as i32, y as i32)
}

pub fn tile_position_to_index(tile_position: UVec2) -> usize {
    assert!(
        tile_position.x < CHUNK_SIZE_X as u32 && tile_position.y < CHUNK_SIZE_Y as u32,
        "position {} out of chunk bounds",
        tile_position
    );
    let index = tile_position.x as usize + tile_position.y as usize * CHUNK_SIZE_X;
    index
}

pub struct Chunk {
    pub chunk_pos: IVec2,
    pub tiles: DataArray<bool>,
    pub tile_info: DataArray<Option<TileInfo>>,
    pub need_update: DataArray<bool>,
    cant_move: DataArray<Option<TileMoveDirection>>,
}

impl Chunk {
    pub fn empty(chunk_pos: IVec2) -> Self {
        Self {
            chunk_pos,
            tiles: data_array(false),
            tile_info: default_data_array(),
            need_update: data_array(false),
            cant_move: default_data_array(),
        }
    }

    pub fn tiles(&self) -> impl Iterator<Item = (usize, &Option<TileInfo>)> {
        self.tile_info.iter().enumerate()
    }

    pub fn set_tile(&mut self, index: usize, tile_info: Option<TileInfo>) -> Vec<Tile> {
        self.need_update[index] = tile_info.is_some();
        self.tiles[index] = tile_info.is_some();
        self.tile_info[index] = tile_info;
        self.cant_move[index] = None;
        self.queue_updates_around(index, 1)
    }

    pub fn queue_update(&mut self, index: usize) {
        self.need_update[index] = true;
        self.cant_move[index] = None;
    }

    pub fn prepare_tick(&mut self) {
        // Apply gravity and calculate tick velocity
        let need_update = &self.need_update;
        self.tile_info
            .iter_mut()
            .enumerate()
            .filter_map(|(index, tile)| tile.as_mut().map(|tile| (index, tile)))
            .for_each(|(index, tile)| {
                if need_update[index] {
                    tile.prepare_tick();
                } else {
                    // Set velocity = gravity for extra updates
                    tile.lazy();
                }
            });
    }

    pub fn prepare_calculation(&mut self) -> (ChunkCalculation, Dependencies) {
        let calculation = ChunkCalculation {
            checked: data_array(false),
            moves_from: data_array(false),
            moves: data_array(None),
            collision: default_data_array(),
            cross_moves: default_data_array(),
            moves_to: default_data_array(),
            collisions: vec![],
            cross_collisions: vec![],
            update_tiles: {
                let mut update_tiles = Vec::new();
                for index in 0..self.need_update.len() {
                    if self.need_update[index] {
                        if self.tiles[index] {
                            update_tiles.push(index);
                            self.cant_move[index] = None;
                        } else {
                            self.need_update[index] = false;
                        }
                    }
                }
                update_tiles
            },
            unknown: data_array(false),
            dependencies: default_data_array(),
        };

        (calculation, HashMap::new())
    }

    pub fn calculation_cycle(
        &mut self,
        calculation: &mut ChunkCalculation,
        dependencies: &mut Dependencies,
        updates: Option<DataArray<bool>>,
        cross_moves: Option<DataArray<bool>>,
    ) -> (Vec<Option<MoveInfo>>, Vec<Tile>, HashSet<Tile>) {
        // Register extra updates
        if let Some(updates) = updates {
            for update_index in updates
                .into_iter()
                .enumerate()
                .filter_map(|(index, update)| if update { Some(index) } else { None })
            {
                self.update_tile(update_index, calculation);
            }
        }

        // Register cross-chunk moves
        if let Some(cross_moves) = cross_moves {
            for (move_to, _) in cross_moves
                .into_iter()
                .enumerate()
                .filter(|&(_, cross_move)| cross_move)
            {
                calculation.moves_to[move_to] = true;
                self.need_update[move_to] = true;
            }
        }

        // Clear unknowns
        for unknown_tile in
            calculation
                .unknown
                .iter_mut()
                .enumerate()
                .filter_map(|(index, unknown_tile)| {
                    if *unknown_tile {
                        *unknown_tile = false;
                        Some(index)
                    } else {
                        None
                    }
                })
        {
            calculation.update_tiles.push(unknown_tile);
            calculation.checked[unknown_tile] = false;
            self.cant_move[unknown_tile] = None;
        }

        // Calculate tiles
        let mut chunk_updates = data_array(None);
        let mut extra_updates = Vec::new();
        let mut cross_moves = HashSet::new();
        let mut collisions = Vec::new();
        while !calculation.update_tiles.is_empty() {
            let update_index = calculation.update_tiles.remove(0);
            let move_info = self.calculate_tile(
                update_index,
                calculation,
                &mut extra_updates,
                &mut cross_moves,
                dependencies,
                &mut collisions,
            );
            chunk_updates[update_index] = Some(move_info);
        }

        (chunk_updates, extra_updates, cross_moves)
    }

    fn calculate_tile(
        &mut self,
        update_index: usize,
        calculation: &mut ChunkCalculation,
        extra_updates: &mut Vec<Tile>,
        cross_moves: &mut HashSet<Tile>,
        dependencies: &mut Dependencies,
        collisions: &mut Vec<(TileInfo, Tile)>,
    ) -> MoveInfo {
        let friction_coef = self.tile_info[update_index]
            .as_ref()
            .map(|tile| tile.tile_type.friction_coef())
            .unwrap_or(0.0);

        // If collision calculated,
        // then perform collision
        if let Some(direction) = calculation.collision[update_index] {
            return MoveInfo::Collision {
                direction: Some(direction),
                friction_coef,
            };
        }

        // If this tile can't move
        // or another tile is going to move here,
        // then perform collision
        if self.cant_move[update_index].is_some() || calculation.moves_to[update_index] {
            return MoveInfo::Collision {
                direction: self.cant_move[update_index],
                friction_coef,
            };
        }

        // If there is no tile
        // or we've calculated that this tile can move,
        // then movement is allowed
        if !self.tiles[update_index] || calculation.moves_from[update_index] {
            return MoveInfo::Possible;
        }

        // If this tile's behaviour is unknown,
        // then movement is unknown
        if calculation.unknown[update_index] {
            return MoveInfo::Unknown;
        }

        // If we've alredy checked this tile
        // then perform collision
        let checked = calculation.checked[update_index];
        calculation.checked[update_index] = true;
        if checked {
            return MoveInfo::Collision {
                direction: None,
                friction_coef,
            };
        }

        // Check for possible moves
        let tile = self.tile_info[update_index].as_mut().unwrap();
        if let Some(direction) = tile.tick_velocity.direction() {
            // Check if target is inside the current chunk
            match Self::shift_position(self.chunk_pos, update_index, direction.direction()) {
                Ok(target_index) => {
                    // Inside the current chunk -> check if movement is possible
                    match self.calculate_tile(
                        target_index,
                        calculation,
                        extra_updates,
                        cross_moves,
                        dependencies,
                        collisions,
                    ) {
                        MoveInfo::Unknown => {
                            calculation.unknown[update_index] = true;
                            return MoveInfo::Unknown;
                        }
                        MoveInfo::Collision {
                            direction,
                            friction_coef,
                        } => {
                            let tile = self.tile_info[update_index].as_mut().unwrap();

                            if let Some(direction) = direction {
                                calculation.collision[update_index] = Some(direction);
                                // Reset velocity
                                tile.hit_wall(
                                    direction,
                                    tile.tile_type.friction_between(friction_coef),
                                );
                            }

                            calculation.collisions.push((update_index, target_index));
                            return MoveInfo::Collision {
                                direction,
                                friction_coef: tile.tile_type.friction_coef(),
                            };
                        }
                        MoveInfo::Impossible => unreachable!(),
                        MoveInfo::Recursive => unreachable!(),
                        MoveInfo::Possible => {
                            // Remove dependency
                            calculation.dependencies[update_index] = None;

                            // Register the move
                            let tile_info = self.tile_info[update_index].as_mut().unwrap();
                            tile_info.tick_velocity -= direction;
                            calculation.moves[update_index] = Some(target_index);
                            calculation.moves_from[update_index] = true;
                            self.cant_move[update_index] = None;

                            // Queue update for the next frame
                            self.need_update[target_index] = true;

                            // Update nearby lazy tiles
                            self.update_tiles_around(update_index, 1, calculation, extra_updates);
                            return MoveInfo::Possible;
                        }
                    }
                }
                Err(tile) => {
                    // Outside of the current chunk
                    let entry = dependencies.entry(tile);
                    let is_current_dependency =
                        calculation.dependencies[update_index] == Some(tile);
                    // If other tile depends on it, then movement is not allowed
                    let vacant = if let std::collections::hash_map::Entry::Vacant(_) = &entry {
                        true
                    } else {
                        false
                    };

                    if is_current_dependency || vacant {
                        if !is_current_dependency {
                            calculation.dependencies[update_index] = Some(tile);
                        }

                        // Check if it's been calculated, else queue calculation
                        let dependency = entry.or_insert(MoveInfo::Unknown);
                        match dependency {
                            MoveInfo::Unknown => {
                                calculation.unknown[update_index] = true;
                                return MoveInfo::Unknown;
                            }
                            MoveInfo::Collision {
                                direction,
                                friction_coef,
                            } => {
                                let update_tile = self.tile_info[update_index].as_mut().unwrap();

                                if let Some(direction) = *direction {
                                    calculation.collision[update_index] = Some(direction);
                                    // Reset velocity
                                    update_tile.hit_wall(
                                        direction,
                                        update_tile.tile_type.friction_between(*friction_coef),
                                    );
                                }

                                calculation.cross_collisions.push((update_index, tile));
                                return MoveInfo::Collision {
                                    direction: *direction,
                                    friction_coef: update_tile.tile_type.friction_coef(),
                                };
                            }
                            MoveInfo::Impossible => {}
                            MoveInfo::Recursive => {
                                // Reset dependency
                                if is_current_dependency {
                                    *dependency = MoveInfo::Unknown;
                                }
                            }
                            MoveInfo::Possible => {
                                // Register the move
                                let tile_info = self.tile_info[update_index].as_mut().unwrap();
                                tile_info.tick_velocity -= direction;
                                cross_moves.insert(tile);
                                calculation.cross_moves[update_index] = Some(tile);
                                calculation.moves_from[update_index] = true;

                                // Update nearby lazy tiles
                                self.update_tiles_around(
                                    update_index,
                                    1,
                                    calculation,
                                    extra_updates,
                                );
                                return MoveInfo::Possible;
                            }
                        }
                    }
                }
            }

            // No possible moves
            let tile = self.tile_info[update_index].as_mut().unwrap();
            tile.hit_wall(direction, 0.0);
            self.cant_move[update_index] = Some(direction);
            calculation.collision[update_index] = Some(direction);
            // self.need_update[update_index] = false;
            return MoveInfo::Collision {
                direction: Some(direction),
                friction_coef: tile.tile_type.friction_coef(),
            };
        }

        // There are no possible moves or velocity is zero
        // let tile = self.tile_info[update_index].as_mut().unwrap();
        // // Lazy if tile attempted to move but failed
        // let need_update = tile.tick_velocity.is_zero() || !tile.velocity.is_close_zero();
        // tile.tick_velocity = IVec2::ZERO.into();

        // self.cant_move[update_index] = !need_update;
        // self.need_update[update_index] = need_update;
        // if !need_update {
        //     tile.lazy();
        // }
        MoveInfo::Collision {
            direction: None,
            friction_coef: 0.0,
        }
    }

    fn queue_updates_around(&mut self, index: usize, distance: i32) -> Vec<Tile> {
        let mut extra_updates = Vec::new();
        // Queue updates in a square around a given tile
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                let shift = ivec2(dx, dy);
                match Self::shift_position(self.chunk_pos, index, shift) {
                    Ok(index) => {
                        if self.tiles[index] {
                            self.queue_update(index);
                        }
                    }
                    Err(tile) => {
                        extra_updates.push(tile);
                    }
                }
            }
        }
        extra_updates
    }

    fn update_tiles_around(
        &mut self,
        index: usize,
        distance: i32,
        calculation: &mut ChunkCalculation,
        extra_updates: &mut Vec<Tile>,
    ) {
        // Update tiles in a square around a given tile
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                let shift = ivec2(dx, dy);
                match Self::shift_position(self.chunk_pos, index, shift) {
                    Ok(index) => {
                        // Tile is inside the chunk
                        if self.tiles[index] && !self.need_update[index] {
                            self.update_tile(index, calculation);
                        }
                    }
                    Err(tile) => {
                        // Tile is outside the chunk
                        extra_updates.push(tile);
                    }
                }
            }
        }
    }

    fn update_tile(&mut self, index: usize, calculation: &mut ChunkCalculation) {
        calculation.update_tiles.push(index);
        calculation.checked[index] = false;
        self.cant_move[index] = None;
    }

    pub fn shift_position(
        chunk_pos: IVec2,
        tile_index: usize,
        shift: IVec2,
    ) -> Result<usize, Tile> {
        // Translate tile index into a vector
        let position = tile_index_to_position(tile_index) + shift;

        // Check if new position is outside the chunk
        let (chunk_shift_x, tile_pos_x) = if position.x < 0 {
            (-1, (position.x + CHUNK_SIZE_X as i32) as u32)
        } else if position.x >= CHUNK_SIZE_X as i32 {
            (1, (position.x - CHUNK_SIZE_X as i32) as u32)
        } else {
            (0, position.x as u32)
        };

        let (chunk_shift_y, tile_pos_y) = if position.y < 0 {
            (-1, (position.y + CHUNK_SIZE_Y as i32) as u32)
        } else if position.y >= CHUNK_SIZE_Y as i32 {
            (1, (position.y - CHUNK_SIZE_Y as i32) as u32)
        } else {
            (0, position.y as u32)
        };

        let tile_position = uvec2(tile_pos_x, tile_pos_y);
        let index = tile_position_to_index(tile_position);

        if chunk_shift_x == 0 && chunk_shift_y == 0 {
            // Inside the chunk
            Ok(index)
        } else {
            // Outside the chunk
            Err(Tile {
                chunk_pos: chunk_pos + ivec2(chunk_shift_x, chunk_shift_y),
                index,
            })
        }
    }

    pub fn collect_movement(
        &mut self,
        calculation: &ChunkCalculation,
    ) -> (
        DataArray<Option<TileInfo>>,
        HashMap<Tile, TileInfo>,
        DataArray<Option<Option<TileInfo>>>,
    ) {
        // Clear view update
        let mut view_update = default_data_array();
        for move_from in calculation
            .moves_from
            .iter()
            .enumerate()
            .filter_map(|(move_from, &do_move)| if do_move { Some(move_from) } else { None })
        {
            view_update[move_from] = Some(None);
        }

        // Collect chunk moves
        let mut local_moves = default_data_array();
        for (move_from, move_to) in calculation
            .moves
            .iter()
            .enumerate()
            .filter_map(|(move_from, move_to)| move_to.map(|move_to| (move_from, move_to)))
        {
            let tile_info = self.tile_info[move_from].take().unwrap();
            local_moves[move_to] = Some(tile_info);
        }

        // Collect cross-chunk moves
        let mut cross_moves = HashMap::with_capacity(calculation.cross_moves.len());
        for (index_from, tile_to) in calculation
            .cross_moves
            .iter()
            .enumerate()
            .filter_map(|(move_from, tile_to)| tile_to.map(|tile_to| (move_from, tile_to)))
        {
            let tile_info = self.tile_info[index_from].take().unwrap();
            cross_moves.insert(tile_to, tile_info);
        }

        (local_moves, cross_moves, view_update)
    }

    pub fn movement(
        &mut self,
        moves: DataArray<Option<TileInfo>>,
        view_update: &mut DataArray<Option<Option<TileInfo>>>,
    ) -> bool {
        // Perform moves
        for (index, tile_info) in moves
            .into_iter()
            .enumerate()
            .filter_map(|(index, tile_info)| tile_info.map(|tile_info| (index, tile_info)))
        {
            view_update[index] = Some(Some(tile_info.clone()));
            self.tile_info[index] = Some(tile_info);
        }

        // Sync tiles and tile_info
        // done means whether all tiles have completed their movement
        let mut done = true;
        for (index, tile) in self.tile_info.iter().enumerate() {
            self.tiles[index] = tile.is_some();
            if let Some(tile) = tile {
                if !tile.tick_velocity.is_zero() {
                    done = false;
                }
            }
        }

        done
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MoveInfo {
    Unknown,
    Impossible,
    Recursive,
    Collision {
        direction: Option<TileMoveDirection>,
        friction_coef: f32,
    },
    Possible,
}

pub struct ChunkCalculation {
    checked: DataArray<bool>,
    moves_from: DataArray<bool>,
    moves: DataArray<Option<usize>>,
    collision: DataArray<Option<TileMoveDirection>>,
    pub cross_moves: DataArray<Option<Tile>>,
    pub moves_to: DataArray<bool>,
    pub collisions: Vec<(usize, usize)>,
    pub cross_collisions: Vec<(usize, Tile)>,
    update_tiles: Vec<usize>,
    unknown: DataArray<bool>,
    pub dependencies: DataArray<Option<Tile>>,
}
