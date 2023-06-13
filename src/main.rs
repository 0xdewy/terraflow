// use bevy::utils::Duration;

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

mod tiles;
mod utils;
mod world;

use tiles::{TileAssets, TileType, WeatherEffects};
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
pub const HUMIDITY_TRAVEL_FACTOR: f32 = 0.5;

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

/// Lower this value to make terrain more sensitive to change
pub const TERRAIN_CHANGE_SENSITIVITY: f32 = 0.1;

/// Describes the orientation and tile size of a hexagon grid.
pub fn pointy_layout() -> HexLayout {
    HexLayout {
        orientation: HexOrientation::pointy(),
        hex_size: HEX_SIZE,
        ..default()
    }
}

///////////////////////////////// Resources /////////////////////////////////////////

#[derive(Debug, Clone, Resource, Default)]
pub struct Epochs {
    epochs: u16,
    times_called: HashMap<Entity, u16>,
}

#[derive(Debug, Resource)]
pub struct HexToEntity(HashMap<Hex, Entity>);

///////////////////////////////// Components /////////////////////////////////////////

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

impl From<TileType> for SoilElevation {
    fn from(tile_type: TileType) -> SoilElevation {
        match tile_type {
            TileType::Grass
            | TileType::Hills
            | TileType::Forest
            | TileType::Jungle
            | TileType::Swamp => SoilElevation { value: 1.0 },
            TileType::Desert | TileType::Waste => SoilElevation { value: 0.3 },
            TileType::Rocky | TileType::Dirt => SoilElevation { value: 0.1 },
            TileType::Ocean | TileType::Water => SoilElevation { value: 0.0 },
            TileType::Mountain | TileType::Ice => SoilElevation { value: 0.0 },
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct WaterElevation {
    pub value: f32,
}

impl From<f32> for WaterElevation {
    fn from(value: f32) -> Self {
        WaterElevation { value }
    }
}

impl From<TileType> for WaterElevation {
    fn from(tile_type: TileType) -> Self {
        match tile_type {
            TileType::Ocean | TileType::Water | TileType::Swamp => 1.0.into(),
            TileType::Ice
            | TileType::Grass
            | TileType::Hills
            | TileType::Forest
            | TileType::Jungle => 0.7.into(),
            TileType::Dirt | TileType::Rocky => 0.5.into(),
            TileType::Mountain | TileType::Desert | TileType::Waste => 0.2.into(),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Humidity {
    pub value: f32,
}

impl From<f32> for Humidity {
    fn from(value: f32) -> Self {
        Humidity { value }
    }
}

impl From<TileType> for Humidity {
    fn from(tile_type: TileType) -> Humidity {
        match tile_type {
            TileType::Ocean | TileType::Water | TileType::Swamp | TileType::Jungle => 1.0.into(),
            TileType::Ice | TileType::Grass | TileType::Hills | TileType::Forest => 0.7.into(),
            TileType::Dirt | TileType::Rocky | TileType::Mountain | TileType::Desert => 0.5.into(),
            TileType::Waste => 0.2.into(),
        }
    }
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

///////////////////////////////// Terrain Changes /////////////////////////////////////////
/// TODO: should this attach a component to the entity if the tile changes? or signal event?
/// Gives this entity a new tiletype if weather conditions are met
fn morph_terrain_system(mut query: Query<(Entity, &ElevationBundle, &Humidity, &mut TileType)>) {
    for (_entity, elevation, humidity, mut tile_type) in query.iter_mut() {
        let mut tile_probabilities = humidity.apply_weather(&tile_type);

        tile_probabilities.extend((&elevation.water, &elevation.soil).apply_weather(&tile_type));

        let new_tile = tile_probabilities.pick_random();
        *tile_type = new_tile;
    }
}

fn update_terrain_assets(
    mut query: Query<(
        Entity,
        &TileType,
        &HexCoordinates,
        &ElevationBundle,
        &mut Transform,
        &mut Handle<Mesh>,
        &mut Handle<StandardMaterial>,
    )>,
    tile_assets: Res<TileAssets>,
) {
    for (_entity, tile_type, hex, altitude, mut transform, mut mesh_handle, mut material_handle) in
        query.iter_mut()
    {
        let world_pos = pointy_layout().hex_to_world_pos(hex.0);

        // TODO: water height is getting too high
        let total_height = altitude.bedrock.value + altitude.soil.value + altitude.water.value;

        let (new_mesh_handle, new_material_handle) = tile_assets.get_mesh_and_material(tile_type);

        // TODO: pickable is being lost when this gets updated
        // update entity with new mesh, material and transform
        *transform = Transform::from_xyz(world_pos.x, total_height, world_pos.y)
            .with_scale(Vec3::splat(2.0));
        *mesh_handle = new_mesh_handle;
        *material_handle = new_material_handle;
    }
}

/////////////////////////////////Weather Systems//////////////////////////////////////////////////

fn precipitation_system(
    mut query: Query<(
        &Humidity,
        &mut Precipitation,
        &TileType,
        &mut ElevationBundle,
    )>,
) {
    for (humidity, mut precipitation, tile_type, mut water_level) in query.iter_mut() {
        // Calculate precipitation
        precipitation.value +=
            humidity.value * PRECIPITATION_FACTOR * tile_type.precipitation_factor();

        water_level.water.value += precipitation.value;
    }

    assert!(query.iter().len() > 0);
}

// TODO: positive temperature component?
fn evaporation_system(mut query: Query<(&mut ElevationBundle, &mut Humidity, &Temperature)>) {
    for (mut elevation, mut humidity, temperature) in query.iter_mut() {
        // Calculate evaporation
        let evaporation = temperature.value.max(0.0) * elevation.water.value * EVAPORATION_FACTOR;
        // println!("evaporation: {}", evaporation);
        // Update humidity
        humidity.value += evaporation;
        elevation.water.value -= evaporation;
    }
}

///////////////////////////////// Terrain Analysis systems /////////////////////////////////////////

fn calculate_neighbour_heights_system(
    mut query: Query<(
        Entity,
        &ElevationBundle,
        &Neighbours,
        &mut LowerNeighbours,
        &mut HigherNeighbours,
    )>,
    neighbour_query: Query<&ElevationBundle>,
) {
    for (_entity, elevation, neighbours, mut lower_neighbours, mut higher_neighbours) in
        query.iter_mut()
    {
        let this_entity_height =
            elevation.bedrock.value + elevation.water.value + elevation.soil.value;

        // Reset the lists of lower and higher neighbours
        lower_neighbours.ids.clear();
        higher_neighbours.ids.clear();

        for neighbour_id in &neighbours.ids {
            if let Ok(neighbour_elevation) = neighbour_query.get(*neighbour_id) {
                let neighbour_height = neighbour_elevation.bedrock.value
                    + elevation.water.value
                    + neighbour_elevation.soil.value;

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

///////////////////////////////// Humidity systems /////////////////////////////////////////

#[derive(Debug, Clone, Component)]
pub struct PendingHumidityRedistribution {
    amount: f32,
}

fn redistribute_humidity_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Humidity, &TileType, &HigherNeighbours)>,
) {
    for (_entity, mut humidity, _tile_type, higher_neighbours) in query.iter_mut() {
        let humidity_to_escape = humidity.value * HUMIDITY_FACTOR;
        let num_higher_neighbours = higher_neighbours.ids.len() as f32;

        for &(neighbour_id, _neighbour_height) in &higher_neighbours.ids {
            let proportion = 1.0 / num_higher_neighbours;
            let humidity_for_neighbour = humidity_to_escape * proportion;

            if humidity_for_neighbour > 0.0 {
                // Add a PendingHumidityRedistribution component to the neighbour
                commands
                    .entity(neighbour_id)
                    .insert(PendingHumidityRedistribution {
                        amount: humidity_for_neighbour,
                    });
            }
        }
        humidity.value -= humidity_to_escape;
    }
}

fn apply_humidity_redistribution(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Humidity, &PendingHumidityRedistribution)>,
) {
    for (entity, mut humidity, redistribution) in query.iter_mut() {
        humidity.value += redistribution.amount;
        // Remove the PendingHumidityRedistribution component
        commands
            .entity(entity)
            .remove::<PendingHumidityRedistribution>();
    }
}

///////////////////////////////// Overflow systems /////////////////////////////////////////
#[derive(Debug, Clone, Component)]
struct IncomingOverflow {
    water: f32,
    soil: f32,
}

fn calculate_overflow_system(
    mut query: Query<(Entity, &ElevationBundle, &mut Overflow, &TileType)>,
) {
    for (_entity, elevation, mut overflow, tiletype) in query.iter_mut() {
        overflow.value += tiletype.overflow_amount(elevation.water.value, elevation.soil.value);
    }
}

// takes in the overflow and creates a component for each lower neighbour, containing their share of the overflow
fn redistribute_overflow_system(
    mut query: Query<(
        Entity,
        &mut Overflow,
        &mut ElevationBundle,
        &LowerNeighbours,
    )>,
    mut commands: Commands,
    mut incoming_overflow_query: Query<&mut IncomingOverflow>,
) {
    for (_entity, mut overflow, mut elevation, lower_neighbours) in query.iter_mut() {
        let this_entity_height =
            elevation.bedrock.value + elevation.water.value + elevation.soil.value;

        // Get the total altitude difference with lower neighbours
        let total_difference: f32 = lower_neighbours
            .ids
            .iter()
            .map(|(_, neighbour_height)| (this_entity_height - neighbour_height).max(0.0))
            .sum();

        // If there are no lower neighbours, add the overflow to the water level
        if total_difference == 0.0 {
            elevation.water.value += overflow.value;
            overflow.value = 0.0;
            continue;
        }

        let num_lower_neighbours = lower_neighbours.ids.len() as f32;

        assert!(num_lower_neighbours > 0.0);

        // Calculate the overflow for each neighbour
        for &(neighbour_id, neighbour_height) in &lower_neighbours.ids {
            let difference = (this_entity_height - neighbour_height).max(0.0);
            let proportion = difference / total_difference;
            let water_overflow_for_neighbour = overflow.value * proportion;
            let soil_overflow_for_neighbour =
                water_overflow_for_neighbour * elevation.soil.value * EROSION_FACTOR;

            if let Ok(mut incoming_overflow) = incoming_overflow_query.get_mut(neighbour_id) {
                incoming_overflow.water += water_overflow_for_neighbour;
                incoming_overflow.soil += soil_overflow_for_neighbour;
            } else {
                commands.entity(neighbour_id).insert(IncomingOverflow {
                    water: water_overflow_for_neighbour,
                    soil: soil_overflow_for_neighbour,
                });
            }
        }

        elevation.water.value -= overflow.value;
        elevation.soil.value -= overflow.value * elevation.soil.value * EROSION_FACTOR;
        overflow.value = 0.0;
    }
}

// Groundwater overflow is applied to this neighbours water level
// TODO: add soil or make seperate function for soil
fn apply_water_overflow(
    mut query: Query<(Entity, &mut ElevationBundle, &IncomingOverflow), Added<IncomingOverflow>>,
    mut commands: Commands,
) {
    for (entity, mut elevation, incoming_overflow) in query.iter_mut() {
        elevation.water.value += incoming_overflow.water;
        elevation.soil.value += incoming_overflow.soil;

        commands.entity(entity).remove::<IncomingOverflow>();
    }
}

////////////////////////////////////////// App /////////////////////////////////////////

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum GroundWaterSystemSet {
    LoadingAssets,
    GeneratingGrid,
    EpochStart,
    LocalWeather,
    TerrainAnalysis,
    RedistributeOverflow,
    ApplyWaterOverflow,
    TerrainMetamorphis,
    VisualUpdate,
}

/*

   app spawning and scheduling

   TODO: an epoch timer should run and trigger a chain of events
   problem is that the other systems are not waiting for the epoch
   i assume the .after() only works on the initial run

    -- right now it is running all systems on a fixed schedule, which seems very inefficient and probably has race conditions

   solution --> run local weather on timer, and have each function create a new component, which will trigger the next function in the chain for that entity
   this may cause some race conditions, but it may be tolerable, and may be considered a bit of randomness?
        --> would have to check that there is no overlap in the systems that are running on the same entity, which seems impossible
*/
fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 0.1,
            ..default()
        })
        .insert_resource(FixedTime::new_from_secs(0.5))
        .insert_resource(Epochs::default())
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
        .add_system(ui_example)
        .add_system(bevy::window::close_on_esc)
        // beginning of epoch
        .add_system(
            epoch_system
                .in_set(GroundWaterSystemSet::EpochStart)
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        // initial local weather systems should run first
        .add_systems(
            (
                precipitation_system,
                evaporation_system,
                calculate_overflow_system,
            )
                .in_set(GroundWaterSystemSet::LocalWeather)
                .after(GroundWaterSystemSet::EpochStart)
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        // terrain analysis systems should run after local weather to consider water levels
        .add_system(
            calculate_neighbour_heights_system
                .in_set(GroundWaterSystemSet::TerrainAnalysis)
                .after(GroundWaterSystemSet::LocalWeather)
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        // now that we know the heights of neighbours we can apply the water/humidity overflows
        .add_systems(
            (redistribute_overflow_system, redistribute_humidity_system)
                .in_set(GroundWaterSystemSet::RedistributeOverflow)
                .after(GroundWaterSystemSet::TerrainAnalysis), // .in_schedule(CoreSchedule::FixedUpdate),
        )
        // apply the overflow to the neighbours state
        // TODO: check the sequence of events if the schedule is not given
        .add_systems(
            (apply_water_overflow, apply_humidity_redistribution)
                .in_set(GroundWaterSystemSet::ApplyWaterOverflow)
                .after(GroundWaterSystemSet::RedistributeOverflow), // .in_schedule(CoreSchedule::FixedUpdate),
        )
        // change the tile type based on the weather conditions
        // update temperature based on new altitude
        .add_system(
            morph_terrain_system
                .in_set(GroundWaterSystemSet::TerrainMetamorphis)
                .after(GroundWaterSystemSet::ApplyWaterOverflow), // .in_schedule(CoreSchedule::FixedUpdate),
        )
        // update tile meshes and materials
        .add_system(
            update_terrain_assets
                .in_set(GroundWaterSystemSet::VisualUpdate)
                .after(GroundWaterSystemSet::TerrainMetamorphis), // .in_schedule(CoreSchedule::FixedUpdate),
        )
        .run();
}

fn epoch_system(
    query: Query<(
        &TileType,
        &Humidity,
        &Precipitation,
        &Evaporation,
        &Temperature,
        &ElevationBundle,
    )>,
    mut epochs: ResMut<Epochs>,
) {
    println!("=== Epoch: {} ===\n", epochs.epochs);
    epochs.epochs += 1;

    for (tile_type, humidity, precipitation, evaporation, temperature, elevation) in query.iter() {
        if tile_type != &TileType::Ice {
            println!(
                "Tile Type: {:?}\n\
             Humidity: {:?}\n\
             Precipitation: {:?}\n\
             Evaporation: {:?}\n\
             Temperature: {:?}\n\
             Elevation: {:?}\n\
             ------------------",
                tile_type, humidity, precipitation, evaporation, temperature, elevation
            );
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct ElevationBundle {
    bedrock: BedrockElevation,
    soil: SoilElevation,
    water: WaterElevation,
}

impl From<(TileType, BedrockElevation)> for ElevationBundle {
    fn from(tile_type: (TileType, BedrockElevation)) -> Self {
        Self {
            bedrock: tile_type.1.into(),
            soil: tile_type.0.into(),
            water: tile_type.0.into(),
        }
    }
}

fn load_tile_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let tile_assets = tiles::TileAssets::new(&asset_server);
    commands.insert_resource(tile_assets);
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
        let altitude = *altitude_map.get(&hex).unwrap();
        let temperature = *temperature_map.get(&hex).unwrap();

        // spawn tile based on altitude and temperature
        let tile_type = world.spawn_tile(hex.y as f32, altitude, temperature);
        let (mesh_handle, material_handle) = tile_assets.get_mesh_and_material(&tile_type);

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
                ElevationBundle::from((tile_type, BedrockElevation { value: altitude })),
                Humidity::from(tile_type),
                Temperature { value: temperature },
                Evaporation { value: 0.0 },
                Precipitation { value: 0.0 },
                Overflow { value: 0.0 }, // TODO: should this be calculated before epoch?
                HexCoordinates(hex),
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
    query: Query<(Entity, &ElevationBundle, &Overflow, &Humidity)>,
) -> Bubble {
    // Get the entity and its terrain
    for (entity, elevation, overflow, humidity) in query.iter() {
        if entity == event.target {
            println!(
                "Entity: {:?}, water: {:?}, soil: {:?}, overflow: {:?}, humidity: {:?}",
                entity, elevation.water, elevation.soil, overflow, humidity
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
