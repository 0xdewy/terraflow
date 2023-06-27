use bevy::prelude::*;

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_mod_picking::prelude::*;

use bevy_egui::EguiPlugin;

// use hexx::shapes;
use hexx::*;

use std::collections::HashMap;

mod benchmark;
mod components;
mod map_generation;
mod terrain;
mod ui;
mod utils;
mod weather_systems;
mod world;

use components::{
    BedrockElevation, DebugWeatherBundle, ElevationBundle, Evaporation, HexCoordinates,
    HigherNeighbours, Humidity, HumidityReceived, HumiditySent, IncomingOverflow, Neighbours,
    Overflow, OverflowReceived, PendingHumidityRedistribution, Precipitation, Temperature,
};
use ui::{terrain_callback, terrain_details, SelectedTile};
use weather_systems::{
    apply_humidity_redistribution, apply_water_overflow, calculate_neighbour_heights_system,
    evaporation_system, morph_terrain_system, precipitation_system, redistribute_humidity_system,
    redistribute_overflow_system, update_terrain_assets,
};
use world::{TileTypeGenerator, WorldAttributes};

pub const EPOCHS_ON_ENTER: u8 = 10;

/// Describes the orientation and tile size of a hexagon grid.
pub fn pointy_layout(hex_size: f32) -> HexLayout {
    HexLayout {
        orientation: HexOrientation::pointy(),
        hex_size: Vec2::splat(hex_size),
        ..default()
    }
}

///////////////////////////////// Resources /////////////////////////////////////////
///

#[derive(Debug, Clone, Resource, Default)]
pub struct Epochs {
    epochs: u16,
    fn_order: Vec<String>,
    epochs_to_run: u16,
}

#[derive(Debug, Resource)]
pub struct HexToEntity(HashMap<Hex, Entity>);

////////////////////////////////////////// App /////////////////////////////////////////

// Replace the u8 with unit (), since we don't need to keep track of the number of executions anymore
#[derive(Clone, Eq, PartialEq, Debug, Hash, States, Default)]
pub enum GameStates {
    Waiting,
    #[default]
    EpochStart,
    EpochRunning,
    EpochFinish,
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 0.1,
            ..default()
        })
        .insert_resource(Epochs::default())
        .insert_resource(SelectedTile::default())
        .insert_resource(benchmark::BenchmarkResource::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_plugin(CameraControllerPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(play_tunes)
        .add_startup_system(load_tile_assets)
        .add_startup_system(setup_grid.after(load_tile_assets))
        .add_system(terrain_details)
        .add_system(bevy::window::close_on_esc)
        .add_state::<GameStates>()
        .add_system(start_epoch)
        // initial weather phase
        .add_system(benchmark::start_benchmark.in_schedule(OnEnter(GameStates::EpochStart)))
        .add_system(precipitation_system.in_schedule(OnEnter(GameStates::EpochStart)))
        .add_system(evaporation_system.in_schedule(OnEnter(GameStates::EpochStart)))
        // .add_system(calculate_overflow_system.in_schedule(OnEnter(GameStates::EpochStart)))
        .add_system(calculate_neighbour_heights_system.in_schedule(OnEnter(GameStates::EpochStart)))
        // calculate neighbour effects
        .add_system(redistribute_humidity_system.in_schedule(OnExit(GameStates::EpochStart)))
        .add_system(redistribute_overflow_system.in_schedule(OnExit(GameStates::EpochStart)))
        // apply effects on neighbours
        .add_system(apply_water_overflow.in_schedule(OnEnter(GameStates::EpochRunning)))
        .add_system(apply_humidity_redistribution.in_schedule(OnEnter(GameStates::EpochRunning)))
        // update terrain assets and map
        .add_system(morph_terrain_system.in_schedule(OnExit(GameStates::EpochRunning)))
        .add_system(update_terrain_assets.in_schedule(OnEnter(GameStates::EpochFinish)))
        .add_system(finish_epoch.in_schedule(OnExit(GameStates::EpochFinish)))
        .add_system(benchmark::end_benchmark.in_schedule(OnExit(GameStates::EpochFinish)))
        .run();
}

fn finish_epoch(mut epochs: ResMut<Epochs>, mut next_state: ResMut<NextState<GameStates>>) {
    if epochs.epochs_to_run > 0 {
        println!("{}", *epochs);
        epochs.epochs_to_run -= 1;
        epochs.epochs += 1;
        next_state.set(GameStates::EpochStart);
    }
}

// Move the epoch forward on keystroke
fn start_epoch(
    mut epochs: ResMut<Epochs>,
    keypress: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<GameStates>>,
) {
    if keypress.just_pressed(KeyCode::Space) {
        println!("=== Epoch: {} ===\n", epochs.epochs);
        epochs.epochs += 1;
        for epoch in epochs.fn_order.iter() {
            println!(" ---> {}", epoch);
        }

        next_state.set(GameStates::EpochStart);
        epochs.fn_order.clear();
        epochs.epochs_to_run = 0;
    }

    if keypress.just_pressed(KeyCode::Return) {
        println!("Running {} Epochs", EPOCHS_ON_ENTER);

        next_state.set(GameStates::EpochStart);
        epochs.fn_order.clear();
        epochs.epochs_to_run = EPOCHS_ON_ENTER.into();
    }
}

fn load_tile_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let tile_assets = terrain::TileAssets::new(&asset_server);
    commands.insert_resource(tile_assets);
}

/// Hex grid setup
fn setup_grid(asset_server: Res<AssetServer>, mut commands: Commands) {
    let world = WorldAttributes::load();

    // load all gltf files from assets folder
    let tile_assets = terrain::TileAssets::new(&asset_server);

    // use hexx lib to generate hexagon shaped map of hexagons
    let all_hexes: Vec<Hex> =
        hexx::shapes::hexagon(Hex::ZERO, world.map.map_radius as u32).collect();

    // generate altitude and derive temperature from that
    let altitude_map = map_generation::generate_altitude_map(&world.elevation, &all_hexes);
    let temperature_map = map_generation::generate_temperature_map(
        &world.temperature,
        world.map.map_radius,
        &altitude_map,
    );

    let mut hex_to_entity = HashMap::new();

    // Spawn tiles
    for hex in all_hexes.clone() {
        let altitude = *altitude_map.get(&hex).unwrap();
        let temperature = *temperature_map.get(&hex).unwrap();

        // spawn tile based on altitude and temperature
        let tile_type = world.spawn_tile(hex.y as f32, altitude, temperature);
        let (mesh_handle, material_handle) = tile_assets.get_mesh_and_material(&tile_type);

        // hex -> world position
        let pos = pointy_layout(world.map.hex_size).hex_to_world_pos(hex);

        let amount_below_sea_level = (world.elevation.sea_level - altitude).max(0.0);

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
                PickableBundle::default(),    // <- Makes the mesh pickable.
                RaycastPickTarget::default(), // <- Needed for the raycast backend.
                OnPointer::<Click>::run_callback(terrain_callback),
                ElevationBundle::from(tile_type, altitude, amount_below_sea_level),
                Humidity::from(tile_type),
                Temperature { value: temperature },
                DebugWeatherBundle {
                    evaporation: Evaporation { value: 0.0 },
                    precipitation: Precipitation { value: 0.0 },
                    overflow: Overflow {
                        water: 0.0,
                        soil: 0.0,
                    },
                    overflow_received: OverflowReceived {
                        water: 0.0,
                        soil: 0.0,
                    },
                    humidity_received: HumidityReceived { value: 0.0 },
                    humidity_sent: HumiditySent { value: 0.0 },
                },
                HexCoordinates(hex),
                Neighbours { ids: vec![] }, // populate once all entities are spawned
                PendingHumidityRedistribution { value: 0.0 },
                IncomingOverflow {
                    water: 0.0,
                    soil: 0.0,
                },
                tile_type,
            ))
            .id();

        hex_to_entity.insert(hex, id);
    }

    // Populate `Neighbours` component for each entity
    for hex in all_hexes {
        let entity_id = hex_to_entity[&hex];
        let neighbour_hexes = hex.ring(1);
        let neighbour_ids = neighbour_hexes
            .into_iter()
            .filter_map(|neighbour_hex| hex_to_entity.get(&neighbour_hex))
            .cloned()
            .collect::<Vec<Entity>>();

        commands
            .entity(entity_id)
            .insert(Neighbours { ids: neighbour_ids });
    }

    // TODO: hex_to_entity isn't currently used
    commands.insert_resource(HexToEntity(hex_to_entity.clone()));

    // World Attributes
    commands.insert_resource(world.elevation); // ElevationAttributes
    commands.insert_resource(world.erosion); // ErosionAttributes
    commands.insert_resource(world.ecosystem); // EcosystemAttributes
    commands.insert_resource(world.temperature); // TemperatureAttributes
    commands.insert_resource(world.map); // MapAttributes
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

//////////////////////////////Music//////////////////////////////
fn play_tunes(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let music = asset_server.load("music/ganymede_orbiter.wav");
    audio.play(music);
}
