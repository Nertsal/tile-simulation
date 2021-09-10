use macroquad::{
    camera::{set_camera, Camera2D},
    prelude::{
        draw_rectangle_lines, draw_texture, ivec2, mouse_position, screen_height, screen_width,
        vec2, Color, FilterMode, IVec2, Image, Texture2D, Vec2, BLACK, BLUE, WHITE, YELLOW,
    },
};

use crate::{
    constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y},
    update_view::UpdateView,
};

use super::tile::TileInfo;

pub struct Renderer {
    game_camera: Camera2D,
    image: Image,
    texture: Texture2D,
}

impl Renderer {
    pub fn new() -> Self {
        let image = Image::gen_image_color(screen_width() as u16, screen_height() as u16, BLACK);
        Self {
            game_camera: Camera2D {
                offset: vec2(0.0, -1.0),
                zoom: vec2(0.01, 0.01 * screen_width() / screen_height()),
                ..Default::default()
            },
            texture: {
                let texture = Texture2D::from_image(&image);
                texture.set_filter(FilterMode::Nearest);
                texture
            },
            image,
        }
    }

    pub fn mouse_world_pos(&self) -> Vec2 {
        let pos = mouse_position();
        let pos = vec2(pos.0, pos.1);
        self.game_camera.screen_to_world(pos)
    }

    pub fn update(&mut self, _delta_time: f32) {}

    pub fn draw(&mut self, view: UpdateView) {
        set_camera(&self.game_camera);
        self.draw_game(view);
        self.draw_chunks();
    }

    fn draw_game(&mut self, view: UpdateView) {
        // let offset = self.game_camera.world_to_screen(vec2(0.0, 0.0));
        // let offset = ivec2(offset.x as i32, offset.y as i32);
        let offset = ivec2(self.image.width as i32 / 2, 0);
        for (pos, tile) in view.into_tiles() {
            let pos = pos + offset;
            if pos.x >= 0
                && pos.x < self.image.width as i32
                && pos.y >= 0
                && pos.y < self.image.height as i32
            {
                match tile {
                    None => self.image.set_pixel(pos.x as u32, pos.y as u32, BLACK),
                    Some(tile_info) => {
                        let color = tile_color(tile_info);
                        self.image.set_pixel(pos.x as u32, pos.y as u32, color);
                    }
                }
            }
        }

        self.texture.update(&self.image);
        draw_texture(self.texture, -offset.x as f32, -offset.y as f32, WHITE);
    }

    fn draw_chunks(&self) {
        const CHUNKS: i32 = 1;
        for x in -CHUNKS..=CHUNKS {
            for y in 0..=CHUNKS * 2 {
                let pos = ivec2(x, y);
                self.draw_chunk(pos);
            }
        }
    }

    fn draw_chunk(&self, chunk_pos: IVec2) {
        draw_rectangle_lines(
            (chunk_pos.x as f32) * CHUNK_SIZE_X as f32,
            (chunk_pos.y as f32) * CHUNK_SIZE_Y as f32,
            CHUNK_SIZE_X as f32,
            CHUNK_SIZE_Y as f32,
            0.1,
            WHITE,
        )
    }
}

fn tile_color(tile_info: TileInfo) -> Color {
    match tile_info {
        TileInfo::Barrier => WHITE,
        TileInfo::Sand => YELLOW,
        TileInfo::Water { .. } => BLUE,
    }
}
