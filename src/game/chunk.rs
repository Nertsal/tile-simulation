use std::collections::HashMap;

use macroquad::prelude::{ivec2, uvec2, IVec2, UVec2};

use crate::constants::{CHUNK_SIZE, CHUNK_SIZE_X, CHUNK_SIZE_Y};

use super::tile::{Tile, TileInfo};

pub type Dependencies = HashMap<Tile, MoveInfo>;

pub type DataArray<T> = Vec<T>;

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
    cant_move: DataArray<bool>,
}

impl Chunk {
    pub fn empty(chunk_pos: IVec2) -> Self {
        Self {
            chunk_pos,
            tiles: data_array(false),
            tile_info: default_data_array(),
            need_update: data_array(false),
            cant_move: data_array(false),
        }
    }

    pub fn tiles(&self) -> impl Iterator<Item = (usize, &Option<TileInfo>)> {
        self.tile_info.iter().enumerate()
    }

    pub fn set_tile(&mut self, index: usize, tile_info: Option<TileInfo>) -> Vec<Tile> {
        self.need_update[index] = tile_info.is_some();
        self.tiles[index] = tile_info.is_some();
        self.tile_info[index] = tile_info;
        self.cant_move[index] = false;
        self.queue_updates_around(index, 1)
    }

    pub fn queue_update(&mut self, index: usize) {
        self.need_update[index] = true;
        self.cant_move[index] = false;
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
            moves_to: default_data_array(),
            update_tiles: {
                let mut update_tiles = Vec::new();
                for index in 0..self.need_update.len() {
                    if self.need_update[index] {
                        if self.tiles[index] {
                            update_tiles.push(index);
                        } else {
                            self.need_update[index] = false;
                        }
                    }
                }
                update_tiles
            },
            unknown: data_array(false),
            dependencies: default_data_array(),
            view_update: default_data_array(),
        };

        (calculation, HashMap::new())
    }

    pub fn calculation_cycle(
        &mut self,
        calculation: &mut ChunkCalculation,
        dependencies: &mut Dependencies,
        updates: Option<DataArray<bool>>,
        cross_moves: Option<DataArray<Option<TileInfo>>>,
    ) -> (Vec<Option<MoveInfo>>, Vec<Tile>, HashMap<Tile, TileInfo>) {
        // Register extra updates
        if let Some(updates) = updates {
            for update_index in updates
                .into_iter()
                .enumerate()
                .filter_map(|(index, update)| if update { Some(index) } else { None })
            {
                calculation.update_tiles.push(update_index);
                calculation.checked[update_index] = false;
                self.cant_move[update_index] = false;
            }
        }

        // Register cross-chunk moves
        if let Some(cross_moves) = cross_moves {
            for (move_to, tile_info) in cross_moves
                .into_iter()
                .enumerate()
                .filter_map(|(index, tile_info)| tile_info.map(|tile_info| (index, tile_info)))
            {
                calculation.view_update[move_to] = Some(Some(tile_info.clone()));
                calculation.moves_to[move_to] = Some(tile_info);
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
            self.cant_move[unknown_tile] = false;
        }

        // Calculate tiles
        let mut chunk_updates = data_array(None);
        let mut extra_updates = Vec::new();
        let mut cross_moves = HashMap::new();
        while !calculation.update_tiles.is_empty() {
            let update_index = calculation.update_tiles.remove(0);
            let move_info = self.calculate_tile(
                update_index,
                calculation,
                &mut extra_updates,
                &mut cross_moves,
                dependencies,
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
        cross_moves: &mut HashMap<Tile, TileInfo>,
        dependencies: &mut Dependencies,
    ) -> MoveInfo {
        // If this tile couldn't move last frame
        // or another tile is going to move here,
        // then movement is not allowed
        if self.cant_move[update_index] || calculation.moves_to[update_index].is_some() {
            return MoveInfo::Impossible;
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
        // then movement is not allowed
        let checked = calculation.checked[update_index];
        calculation.checked[update_index] = true;
        if checked {
            return MoveInfo::Impossible;
        }

        // Check for possible moves
        let tile = self.tile_info[update_index].as_mut().unwrap();
        for direction in tile.tick_velocity.directions() {
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
                    ) {
                        MoveInfo::Unknown => {
                            calculation.unknown[update_index] = true;
                            return MoveInfo::Unknown;
                        }
                        MoveInfo::Impossible => {
                            // Reduce velocity in that direction
                            let tile = self.tile_info[update_index].as_mut().unwrap();
                            let direction = direction.direction_f32().normalize();
                            tile.velocity -= direction * tile.velocity.velocity.dot(direction);
                        }
                        MoveInfo::Recursive => unreachable!(),
                        MoveInfo::Possible => {
                            // Remove dependency
                            calculation.dependencies[update_index] = None;

                            // Register the move
                            let mut tile_info =
                                std::mem::take(self.tile_info.get_mut(update_index).unwrap())
                                    .unwrap();
                            tile_info.tick_velocity -= direction;
                            calculation.moves[update_index] = Some(target_index);
                            calculation.moves_from[update_index] = true;
                            calculation.moves_to[target_index] = Some(tile_info.clone());
                            self.cant_move[update_index] = false;

                            // Update view
                            calculation.view_update[target_index] = Some(Some(tile_info));
                            if calculation.moves_to[update_index].is_none() {
                                calculation.view_update[update_index] = Some(None);
                            }

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
                            MoveInfo::Impossible => {
                                let tile = self.tile_info[update_index].as_mut().unwrap();
                                let direction = direction.direction_f32().normalize();
                                tile.velocity -= direction * tile.velocity.velocity.dot(direction);
                            }
                            MoveInfo::Recursive => {
                                // Reset dependency
                                if is_current_dependency {
                                    *dependency = MoveInfo::Unknown;
                                }
                            }
                            MoveInfo::Possible => {
                                // Register the move
                                let mut tile_info =
                                    std::mem::take(self.tile_info.get_mut(update_index).unwrap())
                                        .unwrap();
                                tile_info.tick_velocity -= direction;
                                cross_moves.insert(tile, tile_info);
                                calculation.moves_from[update_index] = true;

                                // Update view
                                if calculation.moves_to[update_index].is_none() {
                                    calculation.view_update[update_index] = Some(None);
                                }

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
        }

        // There are no possible moves or velocity is zero
        let tile = self.tile_info[update_index].as_mut().unwrap();
        // Lazy if tile attempted to move but failed
        let need_update = tile.tick_velocity.is_zero() || !tile.velocity.is_zero();
        tile.tick_velocity = IVec2::ZERO.into();

        self.cant_move[update_index] = !need_update;
        self.need_update[update_index] = need_update;
        if !need_update {
            tile.lazy();
        }
        MoveInfo::Impossible
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
                            calculation.update_tiles.push(index);
                            calculation.checked[index] = false;
                            self.cant_move[index] = false;
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

    pub fn movement(&mut self, moves: DataArray<Option<TileInfo>>) -> bool {
        // Perform moves
        for (index, tile_info) in moves
            .into_iter()
            .enumerate()
            .filter_map(|(index, tile_info)| tile_info.map(|tile_info| (index, tile_info)))
        {
            self.tile_info[index] = Some(tile_info);
        }

        // Sync tiles and tile_info
        // done means whether all tiles have moved the distance they had to
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
    Possible,
}

pub struct ChunkCalculation {
    checked: DataArray<bool>,
    moves_from: DataArray<bool>,
    moves: DataArray<Option<usize>>,
    pub moves_to: DataArray<Option<TileInfo>>,
    update_tiles: Vec<usize>,
    unknown: DataArray<bool>,
    pub dependencies: DataArray<Option<Tile>>,
    pub view_update: DataArray<Option<Option<TileInfo>>>,
}
