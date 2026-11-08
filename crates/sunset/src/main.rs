//! Sunset application: windowing, input, presentation, and rendering adapter
//! over the authoritative `sunset-core` world model (PRD 4.1).

mod camera;
mod hud;
mod renderer;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

use sunset_core::fixture;
use sunset_core::world::World as GameWorld;

/// The authoritative world, held as an opaque resource behind the adapter.
#[derive(Resource)]
pub struct CoreWorld(pub GameWorld);

/// Selectable development fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fixture {
    BuildingYard,
}

fn main() {
    let fixture = parse_args(std::env::args().skip(1));

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Sunset — M0".into(),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .insert_resource(CoreWorld(build_fixture(fixture)))
        .add_systems(
            Startup,
            (
                camera::spawn_camera,
                renderer::spawn_chunk_meshes,
                hud::spawn_hud,
            ),
        )
        .add_systems(Update, (camera::flycam, hud::update_hud))
        .run();
}

fn build_fixture(fixture: Fixture) -> GameWorld {
    match fixture {
        Fixture::BuildingYard => fixture::building_yard(),
    }
}

/// Minimal command-line parsing for the documented launch convention:
/// `cargo run --release -p sunset -- --dev-fixture building-yard`.
fn parse_args(mut args: impl Iterator<Item = String>) -> Fixture {
    let mut fixture = Fixture::BuildingYard;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dev-fixture" => match args.next().as_deref() {
                Some("building-yard") => fixture = Fixture::BuildingYard,
                Some(other) => eprintln!("unknown fixture '{other}'; using building-yard"),
                None => eprintln!("--dev-fixture requires a name; using building-yard"),
            },
            other => eprintln!("ignoring unknown argument '{other}'"),
        }
    }
    fixture
}
