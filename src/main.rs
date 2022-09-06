use geng::prelude::*;

const FIXED_DELTA_TIME: f64 = 1.0 / 30.0;

mod game;
mod model;
mod render;

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();

    let geng = Geng::new_with(geng::ContextOptions {
        fixed_delta_time: FIXED_DELTA_TIME,
        ..default()
    });

    let game = game::Game::new(&geng);

    geng::run(&geng, game)
}
