use bevy::{math::vec4, prelude::*};

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_mod_picking::prelude::*;

use bevy_egui::EguiPlugin;

// use hexx::shapes;
use hexx::*;

use rand::prelude::SliceRandom;
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
    BedrockElevation, DebugWeatherBundle, DistancesFromVolcano, ElevationBundle, Evaporation,
    HexCoordinates, Humidity, HumidityReceived, HumiditySent, IncomingOverflow,
    Neighbours, Overflow, OverflowReceived, PendingHumidityRedistribution, Precipitation,
    Temperature,
};

use ui::{terrain_callback, terrain_details, SelectedTile};
use weather_systems::{
    apply_humidity_redistribution, apply_vulcanism, apply_water_overflow,
    calculate_neighbour_heights_system, evaporation_system, morph_terrain_system,
    precipitation_system, redistribute_humidity_system, redistribute_overflow_system,
    update_terrain_assets,
};
use world::{TileTypeGenerator, WorldAttributes};

// number of epochs to run when pressing enter
pub const EPOCHS_ON_ENTER: u8 = 10;

/// Describes the orientation and tile size of a hexagon grid.
pub fn pointy_layout(hex_size: f32) -> HexLayout {
    HexLayout {
        orientation: HexOrientation::Pointy,
        hex_size: Vec2::splat(hex_size),
        ..default()
    }
}

///////////////////////////////// Debug Resources /////////////////////////////////////////
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
        .add_plugins(CameraControllerPlugin)
        .add_plugins(EguiPlugin)
        .add_systems(
            PreStartup,
            (setup_camera, play_tunes, load_tile_assets, setup_grid),
        )
        .add_state::<GameStates>()
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, start_epoch)
        .add_systems(Update, make_pickable)
        .add_systems(Update, terrain_details)
        // initial weather phase
        .add_systems(OnEnter(GameStates::EpochStart), benchmark::start_benchmark)
        .add_systems(OnEnter(GameStates::EpochStart), precipitation_system)
        .add_systems(OnEnter(GameStates::EpochStart), evaporation_system)
        .add_systems(
            OnEnter(GameStates::EpochStart),
            calculate_neighbour_heights_system,
        )
        // calculate neighbour effects
        .add_systems(OnExit(GameStates::EpochStart), redistribute_humidity_system)
        .add_systems(OnExit(GameStates::EpochStart), redistribute_overflow_system)
        // apply effects on neighbours
        .add_systems(OnEnter(GameStates::EpochRunning), apply_water_overflow)
        .add_systems(
            OnEnter(GameStates::EpochRunning),
            apply_humidity_redistribution,
        )
        .add_systems(OnEnter(GameStates::EpochRunning), apply_vulcanism)
        // update terrain assets and map
        .add_systems(OnExit(GameStates::EpochRunning), morph_terrain_system)
        .add_systems(OnEnter(GameStates::EpochFinish), update_terrain_assets)
        .add_systems(OnExit(GameStates::EpochFinish), finish_epoch)
        .add_systems(OnExit(GameStates::EpochFinish), benchmark::end_benchmark)
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

// Move the epoch forward on space bar press
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

/// Makes everything in the scene with a mesh pickable
fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<Pickable>)>,
) {
    for entity in meshes.iter() {
        commands
            .entity(entity)
            .insert((PickableBundle::default(), HIGHLIGHT_TINT.clone()));
    }
}

/// Used to tint the mesh instead of simply replacing the mesh's material with a single color. See
/// `tinted_highlight` for more details.
const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.5, -0.3, 0.9, 0.8), // hovered is blue
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, -0.4, 0.8, 0.8), // pressed is a different blue
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, 0.8, -0.4, 0.0), // selected is green
        ..matl.to_owned()
    })),
};

/// Hex grid setup
fn setup_grid(asset_server: Res<AssetServer>, mut commands: Commands) {
    let world = WorldAttributes::load();

    // use hexx lib to generate hexagon shaped map of hexagons
    let all_hexes: Vec<Hex> =
        hexx::shapes::hexagon(Hex::ZERO, world.map.map_radius as u32).collect();

    let mut rng = rand::thread_rng();
    let volcano_hexes: Vec<Hex> = all_hexes
        .choose_multiple(&mut rng, world.elevation.vulcanism as usize)
        .cloned()
        .collect();

    // generate altitude and derive temperature from that
    let altitude_map =
        map_generation::generate_altitude_map(&world.elevation, &all_hexes, &volcano_hexes);
    let temperature_map = map_generation::generate_temperature_map(
        &world.temperature,
        world.map.map_radius,
        &altitude_map,
    );

    let mut hex_to_entity = HashMap::new();

    let tile_assets = terrain::TileAssets::new(&asset_server);

    // Spawn tiles
    for hex in all_hexes.clone() {
        let altitude = *altitude_map.get(&hex).unwrap();
        let temperature = *temperature_map.get(&hex).unwrap();

        // spawn tile based on altitude and temperature
        let tile_type = world.spawn_tile(hex.y as f32, altitude, temperature);
        let scene = tile_assets.get_scene_handle(tile_type).unwrap();
        let pos = pointy_layout(world.map.hex_size).hex_to_world_pos(hex);
        let amount_below_sea_level = (world.elevation.sea_level - altitude).max(0.0);

        // create terrain entity
        let id = commands
            .spawn((
                SceneBundle {
                    transform: Transform::from_xyz(pos.x, 0.0, pos.y).with_scale(Vec3::splat(2.0)),
                    scene: scene.clone(),
                    ..default()
                },
                On::<Pointer<Click>>::run(terrain_callback),
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

    let distances_to_volcanoes =
        map_generation::get_distances_from_volcanos(&world.elevation, &volcano_hexes);

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

        match distances_to_volcanoes.get(&hex) {
            Some(distances) => {
                commands.entity(entity_id).insert(DistancesFromVolcano(distances.to_vec()));
            }
            None => continue,
        }
    }

    // TODO: hex_to_entity isn't currently used
    commands.insert_resource(HexToEntity(hex_to_entity.clone()));
    commands.insert_resource(tile_assets);

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
#[derive(Component)]
struct MyMusic;

fn play_tunes(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("music/ganymede_orbiter.wav"),
            settings: PlaybackSettings::default(),
        },
        MyMusic,
    ));
}
