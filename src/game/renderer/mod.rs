use macroquad::{
    camera::{set_camera, Camera2D},
    prelude::{
        draw_texture, ivec2, screen_height, screen_width, vec2, FilterMode, Image, Texture2D,
        BLACK, WHITE,
    },
};

use crate::update_view::UpdateView;

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

    pub fn update(&mut self, _delta_time: f32) {}

    pub fn draw(&mut self, view: UpdateView) {
        set_camera(&self.game_camera);
        self.draw_game(view);
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
                    Some(tile_info) => match tile_info {
                        TileInfo::Sand => {
                            let color = WHITE; // tile_color(&tile);
                            self.image.set_pixel(pos.x as u32, pos.y as u32, color);
                        }
                    },
                }
            }
        }

        self.texture.update(&self.image);
        draw_texture(self.texture, -offset.x as f32, -offset.y as f32, WHITE);
    }
}
