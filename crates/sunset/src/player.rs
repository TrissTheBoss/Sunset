//! Client-side player controller: input → intent, look rotation (applied at
//! render rate for smoothness), and the first-person camera derived from the
//! authoritative player state with interpolation (PRD 4.3).

use bevy::input::mouse::{AccumulatedMouseMotion, MouseButton};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

use sunset_core::intent::BlockAction;
use sunset_core::physics;
use sunset_core::player::{PLAYER_EYE_HEIGHT, PLAYER_HEIGHT};

use crate::renderer::{Target, TargetHit};
use crate::{Intent, Session};

/// Look sensitivity in radians per pixel of mouse motion.
const SENSITIVITY: f32 = 0.0025;
/// Block interaction reach distance.
const REACH: f32 = 5.0;

/// Marker for the first-person camera entity.
#[derive(Component)]
pub struct PlayerCamera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        IsDefaultUiCamera,
        Transform::default(),
        PlayerCamera,
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    mouse: Res<AccumulatedMouseMotion>,
    mut cursor: Single<&mut CursorOptions>,
    mut session: ResMut<Session>,
    mut intent: ResMut<Intent>,
    mut target: ResMut<Target>,
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

    // Look (render-rate, applied to authoritative rotation for smoothness).
    if *captured {
        let half = std::f32::consts::FRAC_PI_2;
        session.0.player.pitch = (session.0.player.pitch - mouse.delta.y * SENSITIVITY)
            .clamp(-half + 0.001, half - 0.001);
        session.0.player.yaw -= mouse.delta.x * SENSITIVITY;
    }

    // Movement axis (WASD → player-space Vec2).
    let mut move_axis = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        move_axis.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_axis.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_axis.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_axis.x -= 1.0;
    }
    intent.0.move_axis = move_axis.normalize_or_zero();
    intent.0.jump = keys.pressed(KeyCode::Space);

    // Raycast target (from authoritative eye + look direction).
    let eye = session.0.player.eye_pos();
    let rot = Quat::from_euler(
        EulerRot::ZYX,
        0.0,
        session.0.player.yaw,
        session.0.player.pitch,
    );
    let forward = rot * Vec3::NEG_Z;
    target.0 = physics::raycast(&session.0.world, eye, forward, REACH)
        .map(|(pos, face)| TargetHit { pos, face });

    // Block actions (only while captured).
    if *captured {
        if let Some(hit) = target.0 {
            if mouse_btn.pressed(MouseButton::Left) {
                intent.0.block_action = Some(BlockAction::Break(hit.pos));
            } else if mouse_btn.pressed(MouseButton::Right) {
                intent.0.block_action = Some(BlockAction::Place([
                    hit.pos[0] + hit.face[0],
                    hit.pos[1] + hit.face[1],
                    hit.pos[2] + hit.face[2],
                ]));
            }
        }
    }

    // Slot selection: number keys 1..9.
    for i in 0..9 {
        let key = match i {
            0 => KeyCode::Digit1,
            1 => KeyCode::Digit2,
            2 => KeyCode::Digit3,
            3 => KeyCode::Digit4,
            4 => KeyCode::Digit5,
            5 => KeyCode::Digit6,
            6 => KeyCode::Digit7,
            7 => KeyCode::Digit8,
            _ => KeyCode::Digit9,
        };
        if keys.just_pressed(key) {
            intent.0.select_slot = Some(i);
        }
    }
}

pub fn camera_system(
    session: Res<Session>,
    time: Res<Time<Fixed>>,
    mut query: Query<&mut Transform, With<PlayerCamera>>,
) {
    // Interpolate the authoritative position between the previous and current
    // fixed-tick states for smooth rendering.
    let alpha = time.overstep_fraction();
    let pos = session.0.prev_pos.lerp(session.0.player.pos, alpha);
    // `pos` is the collision-box center, so the eye sits `eye_height` above the
    // feet (matching `Player::eye_pos`, which the raycast already uses).
    let eye = pos + Vec3::new(0.0, PLAYER_EYE_HEIGHT - PLAYER_HEIGHT * 0.5, 0.0);
    let rot = Quat::from_euler(
        EulerRot::ZYX,
        0.0,
        session.0.player.yaw,
        session.0.player.pitch,
    );
    for mut tf in &mut query {
        tf.translation = eye;
        tf.rotation = rot;
    }
}
