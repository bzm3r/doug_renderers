mod compute;
mod state;
mod vpull;

use bevy::prelude::*;
use bevy::{
    app::App,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::OrthographicCameraBundle,
    window::WindowDescriptor,
};
use vpull::VertexPullRendererPlugin;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "doug_renderer".into(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .add_plugin(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(VertexPullRendererPlugin)
        .add_startup_system(setup)
        .run();
}

// Ultimately, Doug converts ints into f32s
struct Point {
    x: f32,
    y: f32,
}
// Rect in the format Doug uses
struct Rect {
    p0: Point,
    p1: Point,
}

impl Rect {
    fn new_a() -> Rect {
        Rect {
            p0: Point { x: 100.0, y: 100.0 },
            p1: Point { x: 200.0, y: 200.0 },
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
