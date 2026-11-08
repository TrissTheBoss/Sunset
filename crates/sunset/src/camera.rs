//! M0 debug fly camera: mouse look plus free movement, with cursor capture
//! that can be released and re-acquired (PRD 3.1 acceptance 2).

use bevy::input::mouse::{AccumulatedMouseMotion, MouseButton};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

/// Look sensitivity in radians per pixel of mouse motion.
const SENSITIVITY: f32 = 0.0025;
/// Free-camera movement speed in world units per second.
const MOVE_SPEED: f32 = 10.0;

/// Camera look state (yaw/pitch), kept separate from the visual transform.
#[derive(Resource, Default)]
pub struct FlyCam {
    yaw: f32,
    pitch: f32,
    initialized: bool,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        IsDefaultUiCamera,
        Transform::from_xyz(8.0, 8.0, 16.0).looking_at(Vec3::new(0.0, 5.0, 0.0), Vec3::Y),
        FlyCam::default(),
    ));
}

pub fn flycam(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    mouse: Res<AccumulatedMouseMotion>,
    mut cursor: Single<&mut CursorOptions>,
    mut cam: Single<(&mut Transform, &mut FlyCam), With<Camera3d>>,
    mut captured: Local<bool>,
) {
    // Cursor capture toggle: Esc releases, left click re-captures.
    if keys.just_pressed(KeyCode::Escape) && *captured {
        *captured = false;
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    } else if mouse_btn.just_pressed(MouseButton::Left) && !*captured {
        *captured = true;
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }

    // Initialize yaw/pitch from the starting transform exactly once.
    if !cam.1.initialized {
        let (yaw, pitch, _roll) = cam.0.rotation.to_euler(EulerRot::YXZ);
        cam.1.yaw = yaw;
        cam.1.pitch = pitch;
        cam.1.initialized = true;
    }

    // Mouse look while captured.
    if *captured {
        let half = std::f32::consts::FRAC_PI_2;
        cam.1.pitch = (cam.1.pitch - mouse.delta.y * SENSITIVITY).clamp(-half + 0.001, half - 0.001);
        cam.1.yaw -= mouse.delta.x * SENSITIVITY;
        cam.0.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, cam.1.yaw, cam.1.pitch);
    }

    // Free movement (WASD horizontal, Space/Shift vertical).
    let forward = cam.0.forward();
    let right = cam.0.right();
    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        dir += Vec3::new(forward.x, 0.0, forward.z);
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= Vec3::new(forward.x, 0.0, forward.z);
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += Vec3::new(right.x, 0.0, right.z);
    }
    if keys.pressed(KeyCode::KeyA) {
        dir -= Vec3::new(right.x, 0.0, right.z);
    }
    if keys.pressed(KeyCode::Space) {
        dir += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        dir -= Vec3::Y;
    }

    if dir != Vec3::ZERO {
        cam.0.translation += dir.normalize_or_zero() * MOVE_SPEED * time.delta_secs();
    }
}
