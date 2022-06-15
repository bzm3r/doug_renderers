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

    let scale = 10.0;
    let translate = 10.0;
    let p1_scale = 10.0;
    let rects: Vec<DRect> = (0..5)
        .map(|ix| {
            let ix = ix as f32;
            DRect {
                p0: Point {
                    x: ix * scale - translate,
                    y: ix * scale - translate,
                },
                p1: Point {
                    x: p1_scale * ((1.25 * ix + 1.0) * scale) - translate,
                    y: (1.25 * ix + 1.0) * scale - translate,
                },
                layer: ix,
                color: (ix as u32) % 5,
            }
        })
        .collect();
    // let rects: Vec<DRect> = vec![DRect {
    //     p0: Point { x: -10.0, y: -10.0 },
    //     p1: Point { x: 10.0, y: 10.0 },
    //     z: 0.0,
    // }];
    batched_rects.data = rects;
    commands.spawn_bundle((batched_rects,));
}

// #[derive(Component)]
// struct CameraController {
//     pub enabled: bool,
//     pub initialized: bool,
//     pub sensitivity: f32,
//     pub key_forward: KeyCode,
//     pub key_back: KeyCode,
//     pub key_left: KeyCode,
//     pub key_right: KeyCode,
//     pub key_up: KeyCode,
//     pub key_down: KeyCode,
//     pub key_run: KeyCode,
//     pub key_enable_mouse: MouseButton,
//     pub walk_speed: f32,
//     pub run_speed: f32,
//     pub friction: f32,
//     pub pitch: f32,
//     pub yaw: f32,
//     pub velocity: Vec3,
// }

// impl Default for CameraController {
//     fn default() -> Self {
//         Self {
//             enabled: true,
//             initialized: false,
//             sensitivity: 0.5,
//             key_forward: KeyCode::W,
//             key_back: KeyCode::S,
//             key_left: KeyCode::A,
//             key_right: KeyCode::D,
//             key_up: KeyCode::E,
//             key_down: KeyCode::Q,
//             key_run: KeyCode::LShift,
//             key_enable_mouse: MouseButton::Left,
//             walk_speed: 2.0,
//             run_speed: 6.0,
//             friction: 0.5,
//             pitch: 0.0,
//             yaw: 0.0,
//             velocity: Vec3::ZERO,
//         }
//     }
// }

// fn camera_controller(
//     time: Res<Time>,
//     mut mouse_events: EventReader<MouseMotion>,
//     mouse_button_input: Res<Input<MouseButton>>,
//     key_input: Res<Input<KeyCode>>,
//     mut query: Query<
//         (
//             &mut Transform,
//             &mut OrthographicProjection,
//             &mut CameraController,
//         ),
//         With<Camera>,
//     >,
// ) {
//     let dt = time.delta_seconds();

//     if let Ok((mut transform, mut ortho_projection, mut options)) = query.get_single_mut() {
//         if !options.initialized {
//             let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
//             options.yaw = yaw;
//             options.pitch = pitch;
//             options.initialized = true;
//         }
//         if !options.enabled {
//             return;
//         }

//         // Handle key input
//         let mut axis_input = Vec3::ZERO;
//         if key_input.pressed(options.key_forward) {
//             axis_input.z += 1.0;
//         }
//         if key_input.pressed(options.key_back) {
//             axis_input.z -= 1.0;
//         }
//         if key_input.pressed(options.key_right) {
//             axis_input.x += 1.0;
//         }
//         if key_input.pressed(options.key_left) {
//             axis_input.x -= 1.0;
//         }
//         if key_input.pressed(options.key_up) {
//             axis_input.y += 1.0;
//         }
//         if key_input.pressed(options.key_down) {
//             axis_input.y -= 1.0;
//         }

//         // Apply movement update
//         if axis_input != Vec3::ZERO {
//             let max_speed = if key_input.pressed(options.key_run) {
//                 options.run_speed
//             } else {
//                 options.walk_speed
//             };
//             options.velocity = axis_input.normalize() * max_speed;
//         } else {
//             let friction = options.friction.clamp(0.0, 1.0);
//             options.velocity *= 1.0 - friction;
//             if options.velocity.length_squared() < 1e-6 {
//                 options.velocity = Vec3::ZERO;
//             }
//         }
//         let forward = 1.0;
//         let right = transform.right();
//         transform.translation += options.velocity.x * dt * right
//             + options.velocity.y * dt * Vec3::Y
//             + options.velocity.z * dt * forward;
//         ortho_projection.scale =
//             (ortho_projection.scale + options.velocity.z * dt * forward).max(0.0);

//         // Handle mouse input
//         let mut mouse_delta = Vec2::ZERO;
//         if mouse_button_input.pressed(options.key_enable_mouse) {
//             for mouse_event in mouse_events.iter() {
//                 mouse_delta += mouse_event.delta;
//             }
//         }

//         if mouse_delta != Vec2::ZERO {
//             // Apply look update
//             let (pitch, yaw) = (
//                 (options.pitch - mouse_delta.y * 0.5 * options.sensitivity * dt).clamp(
//                     -0.99 * std::f32::consts::FRAC_PI_2,
//                     0.99 * std::f32::consts::FRAC_PI_2,
//                 ),
//                 options.yaw - mouse_delta.x * options.sensitivity * dt,
//             );
//             transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
//             options.pitch = pitch;
//             options.yaw = yaw;
//         }
//     }
// }
