mod game;
mod story;
mod ui;

use macroquad::prelude::*;
use ui::App;

fn application_icon() -> Option<macroquad::miniquad::conf::Icon> {
    // A packaged macOS app has a multi-resolution ICNS; retain that full-quality icon.
    #[cfg(target_os = "macos")]
    if std::env::current_exe()
        .ok()
        .and_then(|p| p.parent()?.parent().map(std::path::Path::to_path_buf))
        .is_some_and(|contents| contents.join("Resources/AppIcon.icns").is_file())
    {
        return None;
    }
    Some(macroquad::miniquad::conf::Icon {
        small: *include_bytes!("../assets/branding/app-icon-16.rgba"),
        medium: *include_bytes!("../assets/branding/app-icon-32.rgba"),
        big: *include_bytes!("../assets/branding/app-icon-64.rgba"),
    })
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Cardfight!! Vanguard — Limit Break / Break Ride".into(),
        window_width: 1440,
        window_height: 960,
        window_resizable: true,
        high_dpi: true,
        icon: application_icon(),
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
