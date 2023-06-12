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

use world::{TileTypeGenerator, WorldAttributes};

#[derive(Resource)]
pub struct EpochTimer(Timer);

#[derive(Debug, Clone, Component)]
pub struct BedrockElevation {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct SoilElevation {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct WaterElevation {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Humidity {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Overflow {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Precipitation {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Evaporation {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Neighbours {
    pub ids: Vec<Entity>,
}

/////////////////////////////////Weather Systems//////////////////////////////////////////////////

#[derive(Default, Resource)]
struct GroundWaterUpdates(HashMap<Entity, f32>);

fn precipitation_system(mut query: Query<(&Humidity, &mut Precipitation)>) {
    for (humidity, mut precipitation) in query.iter_mut() {
        // Calculate precipitation
        precipitation.value = humidity.value * PRECIPITATION_FACTOR;
    }
}

fn evaporation_system(mut query: Query<(&WaterElevation, &mut Humidity)>) {
    for (water_elevation, mut humidity) in query.iter_mut() {
        // Calculate evaporation
        let evaporation = water_elevation.value * EVAPORATION_FACTOR;
        // Update humidity
        humidity.value = (humidity.value - evaporation).max(0.0);
    }
}

fn overflow_system(
    mut ground_water_updates: ResMut<GroundWaterUpdates>,
    mut query: Query<(Entity, &SoilElevation, &WaterElevation, &mut Overflow)>,
) {
    for (entity, soil_elevation, water_elevation, mut overflow) in query.iter_mut() {
        // Calculate overflow - overflow.value is the height of the water above the soil
        overflow.value += (water_elevation.value - soil_elevation.value).max(0.0);
        // Update overflow in ground water updates
        *ground_water_updates.0.entry(entity).or_insert(0.0) += overflow.value;
    }
}

fn redistribute_overflow_system(
    mut ground_water_updates: ResMut<GroundWaterUpdates>,
    mut query: Query<(
        Entity,
        &BedrockElevation,
        &Neighbours,
        &mut Overflow,
        &mut WaterElevation,
    )>,
    neighbour_query: Query<(&BedrockElevation,)>,
) {
    for (_entity, elevation, neighbours, mut overflow, mut water_level) in query.iter_mut() {
        // Get the total altitude difference with neighbours
        let total_difference: f32 = neighbours
            .ids
            .iter()
            .filter_map(|neighbour_id| neighbour_query.get(*neighbour_id).ok())
            .map(|(neighbour_elevation,)| (neighbour_elevation.value - elevation.value).max(0.0))
            .sum();


        // If there are no lower neighbours, add the overflow to the water level
        if total_difference == 0.0 {
            water_level.value += overflow.value;
            overflow.value = 0.0;
            continue;
        }

        // Calculate the overflow for each neighbour
        let num_neighbours = neighbours.ids.len() as f32;
        for neighbour_id in &neighbours.ids {
            if let Ok((neighbour_elevation,)) = neighbour_query.get(*neighbour_id) {
                let difference = (neighbour_elevation.value - elevation.value).max(0.0);
                let proportion = difference / total_difference;
                let overflow_for_neighbour = overflow.value * proportion;
                *ground_water_updates.0.entry(*neighbour_id).or_insert(0.0) +=
                    overflow_for_neighbour;

                overflow.value -= overflow_for_neighbour;
            }
        }

        // If there's any remaining overflow due to rounding errors, distribute it evenly
        if overflow.value > 0.0 {
            let overflow_per_neighbour = overflow.value / num_neighbours;
            for neighbour_id in &neighbours.ids {
                *ground_water_updates.0.entry(*neighbour_id).or_insert(0.0) +=
                    overflow_per_neighbour;
            }
            overflow.value = 0.0;
        }
    }
}

fn apply_water_overflow(
    ground_water_updates: Res<GroundWaterUpdates>,
    mut query: Query<&mut WaterElevation>,
) {
    for (entity, additional_ground_water) in &ground_water_updates.0 {
        if let Ok(mut ground_water) = query.get_mut(*entity) {
            // println!(
            //     "Applying ground water overflow: {:?}",
            //     &additional_ground_water
            // );
            ground_water.value += *additional_ground_water;
        }
    }
}

////////////////////////////////////////// App /////////////////////////////////////////

#[derive(Debug, Resource)]
pub struct HexToEntity(HashMap<Hex, Entity>);

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 0.1,
            ..default()
        })
        .insert_resource(EpochTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(GroundWaterUpdates::default())
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
        .add_system(bevy::window::close_on_esc)
        .add_system(precipitation_system)
        .add_system(evaporation_system)
        .add_system(overflow_system.after((evaporation_system)))
        .add_system(
            redistribute_overflow_system
                .after(overflow_system)
                .before(apply_water_overflow),
        )
        .add_system(apply_water_overflow)
        .run();
}

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

    let mut hex_to_entity = HashMap::new();

    // Spawn tiles
    for hex in all_hexes.clone() {
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
                PickableBundle::default(),    // <- Makes the mesh pickable.
                RaycastPickTarget::default(), // <- Needed for the raycast backend.
                OnPointer::<Click>::run_callback(terrain_callback),
                // Terrain::new(hex, tile_type, altitude, temperature),
                BedrockElevation { value: altitude },
                WaterElevation {
                    value: tile_type.default_ground_water(),
                },
                SoilElevation {
                    value: tile_type.default_soil(),
                },
                Humidity {
                    value: tile_type.default_humidity(),
                },
                Overflow { value: 0.0 }, // TODO: should this be calculated before epoch?
                Neighbours { ids: vec![] }, // populate once all entities are spawned
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

    commands.insert_resource(HexToEntity(hex_to_entity.clone()));
}

fn terrain_callback(
    // The first parameter is always the `ListenedEvent`, passed in by the event listening system.
    In(event): In<ListenedEvent<Click>>,
    query: Query<(
        Entity,
        &WaterElevation,
        &SoilElevation,
        &Overflow,
        &Humidity,
    )>,
) -> Bubble {
    println!("Clicked terrain");

    // Get the entity and its terrain
    for (entity, water_level, soil_level, overflow, humidity) in query.iter() {
        if entity == event.target {
            println!(
                "Entity: {:?}, water: {:?}, soil: {:?}, overflow: {:?}, humidity: {:?}",
                entity, water_level, soil_level, overflow, humidity
            );
            println!("\n\n");
            break;
        }
    }

    Bubble::Up
}
//////////////////////////////Music//////////////////////////////
fn play_tunes(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let music = asset_server.load("music/galactic_steps.wav");
    audio.play(music);
}

//////////////////////////////InfoBox//////////////////////////////
// TODO: learn how to show Pickable::Click events here
fn ui_example(mut egui_contexts: EguiContexts) {
    egui::Window::new("Terraflow").show(egui_contexts.ctx_mut(), |ui| {
        ScrollArea::both().auto_shrink([true; 2]).show(ui, |ui| {
            ui.heading("TODO: show Terrain attributes here");
        });
    });
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

////////////////////////////// Terrain ///////////////////////////////////////////

// TODO: add as resource
pub const EROSION_FACTOR: f32 = 0.05;
pub const EROSION_SCALE: f32 = 0.1;
pub const PRECIPITATION_FACTOR: f32 = 0.03;
pub const HUMIDITY_FACTOR: f32 = 0.03;
pub const HUMIDITY_TRAVEL_FACTOR: f32 = 0.1;
pub const EVAPORATION_FACTOR: f32 = 0.03;
