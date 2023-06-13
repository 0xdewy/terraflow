use bevy::prelude::*;
use bevy::utils::Duration;

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_mod_picking::prelude::*;

use bevy_egui::{
    egui::{self, ScrollArea},
    EguiContexts, EguiPlugin,
};

use hexx::shapes;
use hexx::*;

use std::collections::HashMap;

mod tiles;
mod utils;
mod world;

use tiles::{TileType, WeatherEffects};
use utils::RandomSelection;
use world::{TileTypeGenerator, WorldAttributes};

//////////////////////////////////////////////// Constants ///////////////////////////////////////////////////
/// The size of the hexagons in the world map (outer radius). The Vec2::splat(2.0) command creates a
/// new Vec2 where both elements are 2.0. This size will be applied to all hexagons in the world map.
pub const HEX_SIZE: Vec2 = Vec2::splat(2.0);

/// The radius of the world map, measured in hexagons. This value determines how large the playable
/// area will be. A larger radius means a larger world map.
pub const MAP_RADIUS: u32 = 50;

/// The erosion factor determines how quickly terrain is eroded by water and wind. A higher value
/// means faster erosion.
pub const EROSION_FACTOR: f32 = 0.05;

/// The erosion scale is a scalar factor that is used to adjust the scale of the erosion effect.
pub const EROSION_SCALE: f32 = 0.1;

/// The precipitation factor affects how much rainfall occurs. A higher value means more rainfall.
pub const PRECIPITATION_FACTOR: f32 = 0.03;

/// The humidity factor determines the overall humidity in the game world. A higher value results in
/// a more humid environment.
pub const HUMIDITY_FACTOR: f32 = 0.03;

/// The humidity travel factor influences how fast humidity moves across the game world. Higher
/// values mean that humidity can travel farther from its original source.
pub const HUMIDITY_TRAVEL_FACTOR: f32 = 0.1;

/// The evaporation factor determines how quickly water evaporates. Higher values lead to faster
/// evaporation.
pub const EVAPORATION_FACTOR: f32 = 0.03;

/// The highest possible elevation in the game world. This value represents the maximum height that
/// terrain can reach.
pub const HIGHEST_ELEVATION: f32 = 10.0;

/// The vulcanism factor determines the number of volcano spawn points
/// Surrounding tiles form a general slope around the spawn point
pub const VULCANISM: f32 = 6.0;

/// The spread of mountain terrain across the game world, as a percentage of the total map radius.
/// A larger spread means that volcano spawn points will spread further across the map
pub const MOUNTAIN_SPREAD: f32 = (MAP_RADIUS * 60 / 100) as f32;

/// The increment by which elevation is adjusted. This constant is used when creating the initial
/// terrain and during terrain modification processes such as erosion and vulcanism.
pub const ELEVATION_INCREMENT: f32 = 0.1;

/// The sea level of the game world. Terrain with an elevation lower than this value will be
/// underwater, while terrain with a higher elevation will be above water.
pub const SEA_LEVEL: f32 = 1.0;

/// Describes the orientation and tile size of a hexagon grid.
pub fn pointy_layout() -> HexLayout {
    HexLayout {
        orientation: HexOrientation::pointy(),
        hex_size: HEX_SIZE,
        ..default()
    }
}

#[derive(Resource)]
pub struct EpochTimer(Timer);

#[derive(Debug, Clone, Component)]
pub struct HexCoordinates(Hex);

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
pub struct Temperature {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Neighbours {
    pub ids: Vec<Entity>,
}

#[derive(Debug, Clone, Component)]
pub struct HigherNeighbours {
    pub ids: Vec<(Entity, f32)>,
}

#[derive(Debug, Clone, Component)]
pub struct LowerNeighbours {
    pub ids: Vec<(Entity, f32)>,
}

///////////////////////////////// Terrain Morphing /////////////////////////////////////////
/// Morphs the terrain depending on current weather state

fn morph_terrain_system(
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut WaterElevation,
        &mut SoilElevation,
        &mut BedrockElevation,
        &Humidity,
        &mut TileType,
    )>,
) {
    for (
        _entity,
        mut transform,
        mut water_level,
        mut soil_level,
        mut bedrock_level,
        humidity,
        mut tile_type,
    ) in query.iter_mut()
    {
        let tile_probabilities = &humidity.apply_weather(&tile_type);
        let new_tile = tile_probabilities.pick_random();
        *tile_type = new_tile;
    }
}

fn update_terrain_assets(
    query: Query<(Entity, &TileType, &HexCoordinates, &BedrockElevation)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let tile_assets = tiles::TileAssets::new(&asset_server);

    println!("Updating terrain mesh");
    for (entity, tile_type, hex, altitude) in query.iter() {
        let world_pos = pointy_layout().hex_to_world_pos(hex.0);

        let (mesh_handle, material_handle) = tile_assets.mesh_and_material(&tile_type);
        // update entity with new mesh and material
        commands.entity(entity).insert(PbrBundle {
            transform: Transform::from_xyz(world_pos.x, altitude.value, world_pos.y)
                .with_scale(Vec3::splat(2.0)),
            mesh: mesh_handle,
            material: material_handle,
            ..default()
        });
    }
}

/////////////////////////////////Weather Systems//////////////////////////////////////////////////

#[derive(Default, Resource)]
struct GroundWaterUpdates(HashMap<Entity, f32>);

fn precipitation_system(mut query: Query<(&Humidity, &mut Precipitation, &TileType)>) {
    for (humidity, mut precipitation, tile_type) in query.iter_mut() {
        // Calculate precipitation
        precipitation.value +=
            humidity.value * PRECIPITATION_FACTOR * tile_type.precipitation_factor();
        println!("Precipitation: {}", precipitation.value);
    }
}

fn evaporation_system(mut query: Query<(&WaterElevation, &mut Humidity, &Temperature)>) {
    for (water_elevation, mut humidity, temperature) in query.iter_mut() {
        // Calculate evaporation
        let evaporation = temperature.value.max(0.0) * water_elevation.value * EVAPORATION_FACTOR;
        println!("Evaporation: {}", evaporation);
        // Update humidity
        humidity.value = (humidity.value - evaporation).max(0.0);
    }
}

fn calculate_overflow_system(
    mut query: Query<(
        Entity,
        &SoilElevation,
        &WaterElevation,
        &mut Overflow,
        &TileType,
    )>,
) {
    for (_entity, soil_elevation, water_elevation, mut overflow, tiletype) in query.iter_mut() {
        overflow.value += tiletype.overflow_amount(water_elevation.value, soil_elevation.value);
    }
}

fn calculate_neighbour_heights_system(
    mut query: Query<(
        Entity,
        &BedrockElevation,
        &Neighbours,
        &WaterElevation,
        &SoilElevation,
        &mut LowerNeighbours,
        &mut HigherNeighbours,
    )>,
    neighbour_query: Query<(&BedrockElevation, &WaterElevation, &SoilElevation)>,
) {
    for (
        _entity,
        elevation,
        neighbours,
        water_level,
        soil_level,
        mut lower_neighbours,
        mut higher_neighbours,
    ) in query.iter_mut()
    {
        let this_entity_height = elevation.value + water_level.value + soil_level.value;

        // Reset the lists of lower and higher neighbours
        lower_neighbours.ids.clear();
        higher_neighbours.ids.clear();

        for neighbour_id in &neighbours.ids {
            if let Ok((neighbour_elevation, neighbour_water, neighbour_soil)) =
                neighbour_query.get(*neighbour_id)
            {
                let neighbour_height =
                    neighbour_elevation.value + neighbour_water.value + neighbour_soil.value;

                // Add the neighbour to the appropriate list
                if neighbour_height < this_entity_height {
                    lower_neighbours.ids.push((*neighbour_id, neighbour_height));
                } else if neighbour_height > this_entity_height {
                    higher_neighbours
                        .ids
                        .push((*neighbour_id, neighbour_height));
                }
            }
        }
    }
}

fn redistribute_overflow_system(
    mut ground_water_updates: ResMut<GroundWaterUpdates>,
    mut query: Query<(
        Entity,
        &mut Overflow,
        &mut WaterElevation,
        &mut SoilElevation,
        &mut BedrockElevation,
        &LowerNeighbours,
    )>,
) {
    for (_entity, mut overflow, mut water_level, soil_level, bedrock_level, lower_neighbours) in
        query.iter_mut()
    {
        let this_entity_height = bedrock_level.value + water_level.value + soil_level.value;

        // Get the total altitude difference with lower neighbours
        let total_difference: f32 = lower_neighbours
            .ids
            .iter()
            .map(|(_, neighbour_height)| (this_entity_height - neighbour_height).max(0.0))
            .sum();

        // If there are no lower neighbours, add the overflow to the water level
        if total_difference == 0.0 {
            water_level.value += overflow.value;
            overflow.value = 0.0;
            continue;
        }

        // Calculate the overflow for each neighbour
        let num_lower_neighbours = lower_neighbours.ids.len() as f32;

        assert!(num_lower_neighbours > 0.0);

        for &(neighbour_id, neighbour_height) in &lower_neighbours.ids {
            let difference = (this_entity_height - neighbour_height).max(0.0);
            let proportion = difference / total_difference;
            let overflow_for_neighbour = overflow.value * proportion;

            *ground_water_updates.0.entry(neighbour_id).or_insert(0.0) += overflow_for_neighbour;

            overflow.value -= overflow_for_neighbour;
            water_level.value -= overflow_for_neighbour;
        }
    }
}

fn apply_water_overflow(
    mut ground_water_updates: ResMut<GroundWaterUpdates>,
    mut query: Query<&mut WaterElevation>,
) {
    let mut entities_to_remove = Vec::new();

    for (entity, additional_ground_water) in &ground_water_updates.0 {
        if let Ok(mut ground_water) = query.get_mut(*entity) {
            println!(
                "Applying ground water overflow: {:?}",
                &additional_ground_water
            );
            ground_water.value += additional_ground_water;
            entities_to_remove.push(*entity);
        }
    }

    for entity in entities_to_remove {
        ground_water_updates.0.remove(&entity);
    }
}

////////////////////////////////////////// App /////////////////////////////////////////

#[derive(Debug, Resource)]
pub struct HexToEntity(HashMap<Hex, Entity>);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum GroundWaterSystemSet {
    EpochStart,
    LocalWeather,
    TerrainAnalysis,
    RedistributeOverflow,
    ApplyWaterOverflow,
    TerrainMetamorphis,
    VisualUpdate,
}

fn main() {
    let duration = Duration::from_secs(1); // Desired delay in seconds

    App::new()
        .insert_resource(AmbientLight {
            brightness: 0.1,
            ..default()
        })
        .insert_resource(FixedTime::new_from_secs(1.0))
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
        .add_system(
            epoch_system
                .in_set(GroundWaterSystemSet::EpochStart)
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .add_systems(
            (
                precipitation_system,
                evaporation_system,
                calculate_overflow_system,
            )
                .in_set(GroundWaterSystemSet::LocalWeather)
                .after(GroundWaterSystemSet::EpochStart),
        )
        .add_system(
            calculate_neighbour_heights_system
                .in_set(GroundWaterSystemSet::TerrainAnalysis)
                .after(GroundWaterSystemSet::LocalWeather),
        )
        .add_system(
            redistribute_overflow_system
                .in_set(GroundWaterSystemSet::RedistributeOverflow)
                .after(GroundWaterSystemSet::TerrainAnalysis),
        )
        .add_system(
            apply_water_overflow
                .in_set(GroundWaterSystemSet::ApplyWaterOverflow)
                .after(GroundWaterSystemSet::RedistributeOverflow),
        )
        .add_system(
            morph_terrain_system
                .in_set(GroundWaterSystemSet::TerrainMetamorphis)
                .after(GroundWaterSystemSet::ApplyWaterOverflow),
        )
        .add_system(
            update_terrain_assets
                .in_set(GroundWaterSystemSet::VisualUpdate)
                .after(GroundWaterSystemSet::TerrainMetamorphis),
        )
        .run();
}

fn epoch_system() {
    println!("Epoch");
}

/// Hex grid setup
fn setup_grid(asset_server: Res<AssetServer>, mut commands: Commands) {
    let world = WorldAttributes::new();
    // load all gltf files from assets folder
    let tile_assets = tiles::TileAssets::new(&asset_server);
    // use hexx lib to generate hexagon shaped map of hexagons
    let all_hexes: Vec<Hex> = shapes::hexagon(Hex::ZERO, MAP_RADIUS).collect();

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
        let pos = pointy_layout().hex_to_world_pos(hex);

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
                Temperature { value: temperature },
                Overflow { value: 0.0 }, // TODO: should this be calculated before epoch?
                HexCoordinates { 0: hex },
                Neighbours { ids: vec![] }, // populate once all entities are spawned
                LowerNeighbours { ids: vec![] }, // populate once weather has run
                HigherNeighbours { ids: vec![] }, // populate once weather has run
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
    let music = asset_server.load("music/intersteller_dreams.wav");
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
