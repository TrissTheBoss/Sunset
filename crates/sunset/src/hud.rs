//! In-game HUD: hotbar, crosshair, and frame timing / build identity.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::text::FontSize;

use sunset_core::block::{block_def, BlockState, AIR};

use crate::Session;

/// Marker for the HUD text span updated each frame.
#[derive(Component)]
pub(crate) struct HudText;

/// Marker for a hotbar slot (index 0..9).
#[derive(Component)]
pub(crate) struct HotbarSlot(usize);

pub fn spawn_hud(mut commands: Commands, session: Res<Session>) {
    // Frame timing + build identity (top-left).
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

    // Crosshair (center).
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Text::new("+"),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    // Hotbar (bottom center).
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            padding: UiRect::bottom(Val::Px(16.0)),
            ..default()
        })
        .with_children(|p| {
            for i in 0..9 {
                let state = session.0.player.hotbar[i];
                let selected = i == session.0.player.selected_slot;
                p.spawn((
                    Node {
                        width: Val::Px(44.0),
                        height: Val::Px(44.0),
                        margin: UiRect::horizontal(Val::Px(2.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(slot_color(state)),
                    BorderColor::all(if selected {
                        Color::WHITE
                    } else {
                        Color::srgb(0.2, 0.2, 0.2)
                    }),
                    HotbarSlot(i),
                ));
            }
        });
}

pub fn update_hud(
    diagnostics: Res<DiagnosticsStore>,
    session: Res<Session>,
    mut text_query: Query<&mut TextSpan, With<HudText>>,
    mut slot_query: Query<(&HotbarSlot, &mut BorderColor)>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .map(|v| format!("{v:.1}"))
        .unwrap_or_else(|| "n/a".into());

    for mut span in &mut text_query {
        **span = format!(
            "sunset v{} (M1)\nFPS: {}\nEsc: release cursor  |  F5: save",
            env!("CARGO_PKG_VERSION"),
            fps
        );
    }

    for (slot, mut border) in &mut slot_query {
        *border = BorderColor::all(if slot.0 == session.0.player.selected_slot {
            Color::WHITE
        } else {
            Color::srgb(0.2, 0.2, 0.2)
        });
    }
}

fn slot_color(state: BlockState) -> Color {
    if state == AIR {
        Color::srgba(0.0, 0.0, 0.0, 0.3)
    } else {
        let d = block_def(state);
        Color::srgb(
            d.color[0] as f32 / 255.0,
            d.color[1] as f32 / 255.0,
            d.color[2] as f32 / 255.0,
        )
    }
}
