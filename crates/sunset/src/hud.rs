//! Minimal in-game HUD: build identity and frame timing (PRD 3.1 acceptance 4).

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::text::FontSize;

/// Marker for the HUD text span updated each frame.
#[derive(Component)]
pub(crate) struct HudText;

pub fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .with_child((TextSpan::default(), HudText));
}

pub fn update_hud(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<HudText>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .map(|v| format!("{v:.1}"))
        .unwrap_or_else(|| "n/a".into());

    let text = format!(
        "{} v{} (M0)\nFPS: {}\nEsc: release cursor   Click: capture",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        fps
    );

    for mut span in &mut query {
        **span = text.clone();
    }
}
