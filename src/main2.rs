use bevy::prelude::*;

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_mod_picking::prelude::*;

use bevy_egui::{
    egui::{self, ScrollArea},
    EguiContexts, EguiPlugin,
};

use hexx::shapes;
use hexx::*;

use std::collections::HashMap;

// Local modules
mod terrain;
mod tiles;
mod utils;
mod world;

use terrain::{Terrain, TerrainMap};
use world::{TileTypeGenerator, WorldAttributes};

#[derive(Resource)]
pub struct TurnTimer(Timer);

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 0.1,
            ..default()
        })
        .insert_resource(TurnTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_plugins(DefaultPlugins)
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_plugin(CameraControllerPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_grid)
        .add_startup_system(play_tunes)
        .add_system(ui_example)
        .add_system(run_epoch)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn play_tunes(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let music = asset_server.load("music/galactic_steps.wav");
    audio.play(music);
}

// Info box
// TODO: learn how to show Pickable::Click events here
fn ui_example(mut egui_contexts: EguiContexts) {
    egui::Window::new("Terraflow").show(egui_contexts.ctx_mut(), |ui| {
        ScrollArea::both().auto_shrink([true; 2]).show(ui, |ui| {
            ui.heading("TODO: show Terrain attributes here");
        });
    });
}

// fn run_epoch(
//     time: Res<Time>,
//     mut timer: ResMut<TurnTimer>,
//     mut terrain: ResMut<TerrainMap>,
// ) {
//     if timer.0.tick(time.delta()).just_finished() {
//         terrain.epoch();
//         // TODO: save asset materials and meshes in TerrainMap
//         // TODO: take assets as parameter
//         // TODO: update entity with new mesh and material if terrain type changes
//     }
// }

/// Hex grid setup
fn setup_grid(asset_server: Res<AssetServer>, mut commands: Commands) {
    let layout = HexLayout {
        orientation: HexOrientation::pointy(),
        hex_size: world::HEX_SIZE,
        ..default()
    };

    let world = WorldAttributes::new();

    // load all gltf files from assets folder
    let tile_assets = tiles::TileAssets::new(&asset_server);

    // use hexx lib to generate hexagon shaped map of hexagons
    let all_hexes: Vec<Hex> = shapes::hexagon(Hex::ZERO, world::MAP_RADIUS).collect();

    // generate altitude and derive temperature from that
    let altitude_map = world.altitude.generate_altitude_map(&all_hexes);
    let temperature_map = world.temperature.generate_temperature_map(&altitude_map);

    // Spawn tiles
    let map: HashMap<Hex, Terrain> = all_hexes
        .into_iter()
        .map(|hex| {
            let altitude = altitude_map.get(&hex).unwrap().clone();
            let temperature = temperature_map.get(&hex).unwrap().clone();

            // spawn tile based on altitude and temperature
            let tile_type = world.spawn_tile(hex.y as f32, altitude, temperature);
            let (mesh_handle, material_handle) = tile_assets.mesh_and_material(&tile_type);

            // hex -> world position
            let pos = layout.hex_to_world_pos(hex);

            // create terrain entity
            let id = commands
                .spawn((
                    PbrBundle {
                        transform: Transform::from_xyz(pos.x, altitude, pos.y)
                            .with_scale(Vec3::splat(2.0)),
                        mesh: mesh_handle,
                        material: material_handle,
                        ..default()
                    },
                    PickableBundle::default(), // <- Makes the mesh pickable.
                    RaycastPickTarget::default(), // <- Needed for the raycast backend.
                    OnPointer::<Click>::run_callback(terrain_callback),
                ))
                .id();

            (hex, Terrain::new(id, hex, tile_type, altitude, temperature))
        })
        .collect();

    let entity_map = map
        .iter()
        .map(|(hex, terrain)| (terrain.entity, *hex))
        .collect();

    commands.insert_resource(TerrainMap {
        map,
        entity_map,
        world_attributes: world,
    });
}

fn terrain_callback(
    // The first parameter is always the `ListenedEvent`, passed in by the event listening system.
    In(event): In<ListenedEvent<Click>>,
    terrain: Res<TerrainMap>,
) -> Bubble {
    println!("Clicked terrain");
    println!(
        "Terrain: {:?}",
        terrain.entity_map.get(&event.target).unwrap()
    );

    let hex = terrain.entity_map.get(&event.target).unwrap();
    println!("Terrain Attributes: {:?}", terrain.map.get(&hex).unwrap());
    Bubble::Up
}

////////////////////// CAMERA MOVEMENT //////////////////////

// 3D Orthogrpahic camera setup
fn setup_camera(mut commands: Commands) {
    let transform = Transform::from_xyz(0.0, 60.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Camera3dBundle {
            transform,
            ..default()
        },
        RaycastPickCamera::default(), // <- Enable picking for this camera
        CameraController {
            orbit_mode: true,
            walk_speed: 50.0,
            run_speed: 100.0,
            ..default()
        },
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        transform,
        ..default()
    });
}
