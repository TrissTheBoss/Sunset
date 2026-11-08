//! Sunset application: windowing, input, presentation, and rendering adapter
//! over the authoritative `sunset-core` simulation (PRD 4.1).

mod hud;
mod mcassets;
mod player;
mod renderer;

use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use sunset_core::fixture;
use sunset_core::intent::PlayerIntent;
use sunset_core::player::Player;
use sunset_core::simulation::Simulation;
use sunset_core::world::World as GameWorld;

/// Where the development save lives (a versioned `.sunset` file).
const SAVE_PATH: &str = "saves/building-yard.sunset";

/// The in-process session boundary holding the authoritative simulation.
#[derive(Resource)]
pub struct Session(pub Simulation);

/// The current frame's client intent, consumed by the fixed-tick simulation.
#[derive(Resource, Default)]
pub struct Intent(pub PlayerIntent);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Sunset — M1".into(),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .insert_resource(Time::<Fixed>::from_hz(20.0))
        .insert_resource(Intent::default())
        .insert_resource(renderer::Target::default())
        .add_systems(PreStartup, (setup_session, setup_assets))
        .add_systems(
            Startup,
            (
                player::spawn_camera,
                renderer::spawn_chunk_meshes,
                hud::spawn_hud,
            ),
        )
        .add_systems(FixedUpdate, simulation_step)
        .add_systems(
            Update,
            (
                player::input_system,
                player::camera_system,
                renderer::outline_system,
                renderer::remesh_system,
                hud::update_hud,
                save_system,
            ),
        )
        .run();
}

/// Load the save, or generate the authored fixture on first launch.
fn setup_session(mut commands: Commands) {
    let (world, player) = load_or_generate();
    commands.insert_resource(Session(Simulation::new(world, player)));
}

/// Load Minecraft block textures from a local install into a texture atlas,
/// if available. Falls back to authored flat colors otherwise.
fn setup_assets(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let Some(dir) = mcassets::find_minecraft_dir() else {
        warn!("No .minecraft directory found; using authored colors");
        return;
    };
    match mcassets::load_assets(&dir) {
        Ok(loaded) => {
            let (w, h) = (loaded.atlas.width(), loaded.atlas.height());
            let image = Image::new(
                Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                loaded.atlas.into_raw(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );
            let handle = images.add(image);
            commands.insert_resource(renderer::BlockAtlas {
                face_textures: loaded.face_textures,
                uv: loaded.uv,
                handle,
            });
            info!("Loaded Minecraft block textures from {}", dir.display());
        }
        Err(e) => warn!("Could not load Minecraft assets ({e}); using authored colors"),
    }
}

fn load_or_generate() -> (GameWorld, Player) {
    let path = Path::new(SAVE_PATH);
    if let Ok(json) = std::fs::read_to_string(path) {
        match sunset_core::save::from_json(&json) {
            Ok((world, player)) => {
                info!("Loaded save from {}", path.display());
                return (world, player);
            }
            Err(e) => warn!("Could not load save ({}); regenerating fixture", e),
        }
    }

    let world = fixture::building_yard();
    let player = Player::new(
        Vec3::new(8.0, 5.5, 16.0),
        0.5,   // yaw
        -0.25, // pitch
    );
    (world, player)
}

/// Apply one tick of client intent to the authoritative simulation.
fn simulation_step(mut session: ResMut<Session>, mut intent: ResMut<Intent>, time: Res<Time>) {
    let dt = time.delta_secs();
    // Discrete actions are consumed exactly once (so catch-up steps don't
    // repeat them); continuous movement state persists across steps.
    let block_action = intent.0.block_action.take();
    let select_slot = intent.0.select_slot.take();
    let step = PlayerIntent {
        move_axis: intent.0.move_axis,
        jump: intent.0.jump,
        block_action,
        select_slot,
    };
    session.0.step(&step, dt);
}

/// Save the current world and player state on `F5`.
fn save_system(keys: Res<ButtonInput<KeyCode>>, session: Res<Session>) {
    if keys.just_pressed(KeyCode::F5) {
        let json = sunset_core::save::to_json(&session.0.world, &session.0.player);
        match sunset_core::save::write_atomic(Path::new(SAVE_PATH), &json) {
            Ok(()) => info!("Saved to {}", SAVE_PATH),
            Err(e) => error!("Save failed: {}", e),
        }
    }
}
