mod gpu_data;
mod phase_item;
mod state;
mod vpull;

use bevy::prelude::*;
use bevy::{app::App, diagnostic::LogDiagnosticsPlugin, window::WindowDescriptor};
use vpull::VpullPlugin;

use bevy_pancam::{PanCam, PanCamPlugin};
use rand::Rng;

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
    pub stroke_width: f32,
    pub color: u32,
}

fn random_point<R: Rng + ?Sized>(rng: &mut R, min: Point, max: Point) -> Point {
    Point {
        x: rng.gen_range(min.x..max.x),
        y: rng.gen_range(min.y..max.y),
    }
}

impl DRect {
    pub fn random<R: Rng + ?Sized>(rng: &mut R, min: Point, max: Point) -> Self {
        DRect {
            p0: random_point(rng, min, max),
            p1: random_point(rng, min, max),
            stroke_width: 0.1,
            color: rng.gen_range(0..5),
        }
    }

    pub fn randomly_placed<R: Rng + ?Sized>(rng: &mut R, min: Point, max: Point) -> Self {
        let p0 = random_point(rng, min, max);
        DRect {
            p0,
            p1: Point {
                x: p0.x + 1.0,
                y: p0.y + 1.0,
            },
            stroke_width: 0.1,
            color: rng.gen_range(0..5),
        }
    }
}

pub struct LayerRects {
    pub rects: Vec<DRect>,
    pub index: u8,
}

#[derive(Clone, Component, Default, Debug)]
pub struct BatchedQuads {
    pub data: Vec<DRect>,
    pub extracted: bool,
    pub prepared: bool,
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PanCam::default());

    let mut batched_rects = BatchedQuads::default();
    let rects = ordered_rects(false);
    batched_rects.data = rects;
    commands.spawn_bundle((batched_rects,));
}

#[allow(dead_code)]
fn ordered_rects(reverse: bool) -> Vec<DRect> {
    let mut rects = vec![
        DRect {
            p0: Point { x: 0.0, y: 0.0 },
            p1: Point { x: 100.0, y: 100.0 },
            stroke_width: 1.0,
            color: 0,
        },
        DRect {
            p0: Point { x: 50.0, y: 50.0 },
            p1: Point { x: 150.0, y: 150.0 },
            stroke_width: 1.0,
            color: 2,
        },
    ];
    if reverse {
        rects.reverse();
    }
    rects
}

#[allow(dead_code)]
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
                stroke_width: 1.0 * i,
                color: ix % 5,
            }
        })
        .collect()
}

#[allow(dead_code)]
fn many_random_rects(n: usize) -> Vec<DRect> {
    let mut result = Vec::with_capacity(n);

    let min = Point {
        x: -300.0,
        y: -300.0,
    };
    let max = Point { x: 300.0, y: 300.0 };

    let mut rng = rand::thread_rng();
    for _ in 0..n {
        result.push(DRect::random(&mut rng, min, max))
    }

    result
}

#[allow(dead_code)]
fn many_small_random_rects(n: usize) -> Vec<DRect> {
    let mut result = Vec::with_capacity(n);

    let min = Point {
        x: -300.0,
        y: -300.0,
    };
    let max = Point { x: 300.0, y: 300.0 };

    let mut rng = rand::thread_rng();
    for _ in 0..n {
        result.push(DRect::randomly_placed(&mut rng, min, max))
    }

    result
}
