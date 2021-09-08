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

        // if this tile's behaviour is unknown,
        // then movement is unknown
        if calculation.unknown[update_index] {
            return MoveInfo::Unknown;
        }

        // If we've alredy checked this tile (implying it can't move)
        // then movement is not allowed
        let checked = calculation.checked[update_index];
        calculation.checked[update_index] = true;
        if checked {
            self.cant_move[update_index] = true;
            return MoveInfo::Impossible;
        }

        // Check for possible moves
        for direction in self.tile_info[update_index]
            .as_ref()
            .unwrap()
            .movement_directions()
        {
            // Check if target is inside the current chunk
            match self.shift_position(update_index, direction) {
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
                        MoveInfo::Impossible => {}
                        MoveInfo::Possible => {
                            // Register the move
                            let tile_info =
                                std::mem::take(self.tile_info.get_mut(update_index).unwrap())
                                    .unwrap();
                            calculation.moves[update_index] = Some(target_index);
                            calculation.moves_from[update_index] = true;
                            calculation.moves_to[target_index] = Some(tile_info.clone());

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
                    // Outside of the current chunk -> check if it's been calculated, else queue calculation
                    match dependencies.entry(tile).or_insert(MoveInfo::Unknown) {
                        MoveInfo::Unknown => {
                            calculation.unknown[update_index] = true;
                            return MoveInfo::Unknown;
                        }
                        MoveInfo::Impossible => {}
                        MoveInfo::Possible => {
                            // Register the move
                            let tile_info =
                                std::mem::take(self.tile_info.get_mut(update_index).unwrap())
                                    .unwrap();
                            cross_moves.insert(tile, tile_info);
                            calculation.moves_from[update_index] = true;

                            // Update view
                            if calculation.moves_to[update_index].is_none() {
                                calculation.view_update[update_index] = Some(None);
                            }

                            // Update nearby lazy tiles
                            self.update_tiles_around(update_index, 1, calculation, extra_updates);
                            return MoveInfo::Possible;
                        }
                    }
                }
            }
        }

        // There are no possible moves
        // Set this tile into lazy mode
        self.cant_move[update_index] = true;
        self.need_update[update_index] = false;
        MoveInfo::Impossible
    }

    fn queue_updates_around(&mut self, index: usize, distance: i32) -> Vec<Tile> {
        let mut extra_updates = Vec::new();
        // Queue updates in a square around a given tile
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                let shift = ivec2(dx, dy);
                match self.shift_position(index, shift) {
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
                match self.shift_position(index, shift) {
                    Ok(index) => {
                        // Tile is inside the chunk
                        if self.tiles[index]
                            && !self.need_update[index]
                            && !calculation.checked[index]
                        {
                            calculation.update_tiles.push(index);
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

    pub fn shift_position(&self, tile_index: usize, shift: IVec2) -> Result<usize, Tile> {
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
                chunk_pos: self.chunk_pos + ivec2(chunk_shift_x, chunk_shift_y),
                index,
            })
        }
    }

    pub fn movement(&mut self, moves: DataArray<Option<TileInfo>>) {
        for (index, tile_info) in moves
            .into_iter()
            .enumerate()
            .filter_map(|(index, tile_info)| tile_info.map(|tile_info| (index, tile_info)))
        {
            self.tile_info[index] = Some(tile_info);
        }

        for (index, tile) in self.tile_info.iter().enumerate() {
            self.tiles[index] = tile.is_some();
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MoveInfo {
    Unknown,
    Impossible,
    Possible,
}

pub struct ChunkCalculation {
    checked: DataArray<bool>,
    moves_from: DataArray<bool>,
    moves: DataArray<Option<usize>>,
    pub moves_to: DataArray<Option<TileInfo>>,
    update_tiles: Vec<usize>,
    unknown: DataArray<bool>,
    pub view_update: DataArray<Option<Option<TileInfo>>>,
}
