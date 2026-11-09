//! In-game HUD: hotbar, crosshair, and frame timing / build identity.

use std::collections::VecDeque;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::text::FontSize;

use sunset_core::block::{block_def, BlockState, AIR};

use crate::Session;

/// Number of recent frame durations kept for average / 0.1%-low FPS.
const FRAME_WINDOW: usize = 1000;

/// Rolling window of frame durations used to report average and 0.1%-low FPS.
#[derive(Resource)]
pub struct FrameStats {
    /// Recent frame durations in seconds, oldest first.
    times: VecDeque<f32>,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            times: VecDeque::with_capacity(FRAME_WINDOW),
        }
    }
}

impl FrameStats {
    fn record(&mut self, dt: f32) {
        if dt > 0.0 && dt.is_finite() {
            self.times.push_back(dt);
            if self.times.len() > FRAME_WINDOW {
                self.times.pop_front();
            }
        }
    }

    /// Average FPS over the window (frames divided by total time).
    pub(crate) fn average_fps(&self) -> Option<f32> {
        let n = self.times.len();
        if n == 0 {
            return None;
        }
        let total: f32 = self.times.iter().sum();
        if total <= 0.0 {
            return None;
        }
        Some(n as f32 / total)
    }

    /// 0.1%-low FPS: `1 / (99.9th-percentile frame time)`.
    pub(crate) fn low_01_fps(&self) -> Option<f32> {
        let n = self.times.len();
        if n == 0 {
            return None;
        }
        let mut sorted: Vec<f32> = self.times.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((n as f32 * 0.999).ceil() as usize)
            .saturating_sub(1)
            .min(n - 1);
        let dt = sorted[idx];
        if dt <= 0.0 {
            return None;
        }
        Some(1.0 / dt)
    }
}

/// Log average and 0.1%-low FPS to stdout once per second, for benchmarking
/// (M5). Mirrors the on-screen HUD values so the frame rate can be captured
/// from a headless run's stdout.
pub fn log_frame_stats(stats: Res<FrameStats>, time: Res<Time>, mut acc: Local<f32>) {
    *acc += time.delta_secs();
    if *acc >= 1.0 {
        *acc = 0.0;
        let avg = stats
            .average_fps()
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "n/a".into());
        let low = stats
            .low_01_fps()
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "n/a".into());
        info!("FPS_STATS avg={avg} 0.1%low={low}");
    }
}

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
    time: Res<Time>,
    mut stats: ResMut<FrameStats>,
    diagnostics: Res<DiagnosticsStore>,
    session: Res<Session>,
    mut text_query: Query<&mut TextSpan, With<HudText>>,
    mut slot_query: Query<(&HotbarSlot, &mut BorderColor)>,
    mut frame: Local<u32>,
    mut cached_low: Local<String>,
) {
    stats.record(time.delta_secs());

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .map(|v| format!("{v:.1}"))
        .unwrap_or_else(|| "n/a".into());
    let avg = stats
        .average_fps()
        .map(|v| format!("{v:.1}"))
        .unwrap_or_else(|| "n/a".into());
    // The 0.1%-low sorts the whole 1000-frame window, so recompute it only
    // every 16 frames and cache the result.
    *frame = frame.wrapping_add(1);
    if *frame == 1 || *frame % 16 == 0 {
        *cached_low = stats
            .low_01_fps()
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "n/a".into());
    }
    let low = &*cached_low;

    for mut span in &mut text_query {
        **span = format!(
            "sunset v{} (M1)\nFPS: {} | avg: {} | 0.1% low: {}\nEsc: cursor  |  F5: save  |  F11: fullscreen",
            env!("CARGO_PKG_VERSION"),
            fps,
            avg,
            low
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_stats_average_and_low() {
        let mut s = FrameStats::default();
        // 100 frames at a steady 10ms each (100 FPS).
        for _ in 0..100 {
            s.record(0.010);
        }
        assert!((s.average_fps().unwrap() - 100.0).abs() < 0.001);
        assert!((s.low_01_fps().unwrap() - 100.0).abs() < 0.001);

        // A single slower frame (50 FPS) should barely dent the average but
        // shows up as the 0.1% low.
        s.record(0.020);
        let avg = s.average_fps().unwrap();
        assert!((avg - 99.0).abs() < 1.0, "avg = {avg}");
        assert!(
            (s.low_01_fps().unwrap() - 50.0).abs() < 0.001,
            "low = {}",
            s.low_01_fps().unwrap()
        );
    }
}
