mod gpu_data;
mod phase_item;
mod state;
mod vpull;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::{app::App, diagnostic::LogDiagnosticsPlugin, window::WindowDescriptor};
use vpull::VpullPlugin;

use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "doug_renderer".into(),
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        // .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(VpullPlugin)
        .add_plugin(PanCamPlugin::default())
        .add_startup_system(setup)
        // .add_system(camera_controller)
        .run();
}

// Ultimately, Doug converts ints into f32s
#[derive(Clone, Copy, Default, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

// Rect in the format Doug uses
#[derive(Clone, Copy, Default, Component, Debug)]
pub struct DRect {
    pub p0: Point,
    pub p1: Point,
    pub layer: f32,
    pub color: u32,
}

pub struct LayerRects {
    pub rects: Vec<DRect>,
    //pub color: Color,
    pub index: u8,
}

#[derive(Clone, Component, Default, Debug)]
pub struct BatchedQuads {
    pub data: Vec<DRect>,
    pub extracted: bool,
    pub prepared: bool,
}

fn setup(mut commands: Commands) {
    // let mut camera = OrthographicCameraBundle::new_2d();
    // // camera.orthographic_projection.scale = 75.0;
    // // camera.transform = Transform::from_translation(50.0 * Vec3::Z).looking_at(Vec3::ZERO, Vec3::Y);
    // commands.spawn_bundle(camera).insert(PanCam::default());
    // // .insert(CameraController::default());

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PanCam::default());

    let mut batched_rects = BatchedQuads::default();
    let rects = overlapping_rects(5);
    batched_rects.data = rects;
    commands.spawn_bundle((batched_rects,));
}

fn overlapping_rects(n: u32) -> Vec<DRect> {
    let scale = 10.0;
    let translate = 10.0;
    (0..n)
        .map(|ix| {
            let i = ix as f32;
            DRect {
                p0: Point {
                    x: i * scale - translate,
                    y: i * scale - translate,
                },
                p1: Point {
                    x: (1.25 * i + 1.0) * scale - translate,
                    y: (1.25 * i + 1.0) * scale - translate,
                },
                layer: i,
                color: ix % 5,
            }
        })
        .collect()
}
