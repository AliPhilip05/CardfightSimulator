mod game;
mod story;
mod ui;

use macroquad::prelude::*;
use ui::App;

fn window_conf() -> Conf {
    Conf {
        window_title: "Cardfight!! Vanguard — Limit Break / Break Ride".into(),
        window_width: 1440,
        window_height: 960,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::load().await;
    loop {
        app.draw();
        next_frame().await;
    }
}
