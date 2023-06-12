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

// use terrain::{run_epoch, Neighbors, Terrain};
use tiles::TileType;
use utils::RandomSelection;
use world::{TileTypeGenerator, WorldAttributes};

#[derive(Resource)]
pub struct EpochTimer(Timer);

#[derive(Debug, Clone, Component)]
pub struct Elevation {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Humidity {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct GroundWater {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Overflow {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Erosion {
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

// The system to handle evaporation
fn evaporation_system(
    mut query: Query<(&Elevation, &mut Humidity, &mut GroundWater, &Evaporation)>,
) {
    for (elevation, mut humidity, mut ground_water, evaporation) in query.iter_mut() {
        // Calculate evaporation here
        // ...
    }
}

// The system to handle precipitation
fn precipitation_system(
    mut query: Query<(&Elevation, &mut Humidity, &mut GroundWater, &Precipitation)>,
) {
    for (elevation, mut humidity, mut ground_water, precipitation) in query.iter_mut() {
        // Calculate precipitation here
        // ...
    }
}

// The system to handle water overflow
fn overflow_system(
    mut query: Query<(&Elevation, &mut GroundWater, &mut Overflow)>,
    hex_to_entity: Res<HexToEntity>,
) {
    for (elevation, mut ground_water, mut overflow) in query.iter_mut() {
        // Calculate water overflow here
        // ...
    }
}

// The system to handle erosion
fn erosion_system(
    mut query: Query<(&Elevation, &mut Overflow, &mut Erosion)>,
    hex_to_entity: Res<HexToEntity>,
) {
    for (elevation, mut overflow, mut erosion) in query.iter_mut() {
        // Calculate erosion here
        // ...
    }
}

#[derive(Default, Resource)]
struct GroundWaterUpdates(HashMap<Entity, f32>);

fn calculate_redistribute_overflow_system(
    mut ground_water_updates: ResMut<GroundWaterUpdates>,
    query: Query<(Entity, &Elevation, &Neighbours, &GroundWater, &Overflow)>,
) {
    ground_water_updates.0.clear();

    for (entity, elevation, neighbours, ground_water, overflow) in query.iter() {
        // Get the total altitude difference with neighbours
        let total_difference: f32 = neighbours
            .ids
            .iter()
            .filter_map(|neighbour_id| query.get(*neighbour_id).ok())
            .map(|(_, neighbour_elevation, _, _, _)| {
                (neighbour_elevation.value - elevation.value).max(0.0)
            })
            .sum();

        // Calculate the overflow for each neighbour
        for neighbour_id in &neighbours.ids {
            if let Ok((_, neighbour_elevation, _, _, _)) = query.get(*neighbour_id) {
                let difference = (neighbour_elevation.value - elevation.value).max(0.0);
                println!("Neighbour heigh difference {:?}", difference);
                let proportion = if total_difference == 0.0 {
                    0.0
                } else {
                    difference / total_difference
                };
                let overflow_for_neighbour = overflow.value * proportion;
                println!("overflow for neighbour: {:?}", overflow_for_neighbour);
                *ground_water_updates.0.entry(*neighbour_id).or_insert(0.0) +=
                    overflow_for_neighbour;
            }
        }
    }
}

fn apply_redistribute_overflow_system(
    ground_water_updates: Res<GroundWaterUpdates>,
    mut query: Query<(&mut GroundWater, &mut Overflow)>,
) {
    for (entity, additional_ground_water) in &ground_water_updates.0 {
        if let Ok((mut ground_water, mut overflow)) = query.get_mut(*entity) {
            // println!("Applying ground water overflow: {:?}", &additional_ground_water);
            ground_water.value += *additional_ground_water;
            overflow.value = 0.0;
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
        .add_system(overflow_system) // New system
        .add_system(calculate_redistribute_overflow_system)
        .add_system(apply_redistribute_overflow_system)
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
                Elevation { value: altitude },
                GroundWater { value: tile_type.default_ground_water() },
                Overflow { value: 0.0 },
                Neighbours { ids: vec![] },
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
    query: Query<(Entity, &GroundWater)>,
) -> Bubble {
    println!("Clicked terrain");

    // Get the entity and its terrain
    for (entity, ground_water) in query.iter() {
        if entity == event.target {
            println!("Terrain: \n groundwater {:?}", ground_water);
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

// Define components for the different parts of a Terrain
#[derive(Debug, Clone, Component)]
pub struct Terrain {
    pub coordinates: Hex,
    pub tile_type: TileType,
    pub altitude: f32,
    pub temperature: f32,
    pub pollution: f32,
    pub ground_water: f32,
    pub humidity: f32,
    pub soil: f32,
}

impl Terrain {
    pub fn new(coordinates: Hex, tile_type: TileType, altitude: f32, temperature: f32) -> Self {
        Self {
            coordinates,
            tile_type: tile_type.clone(),
            altitude,
            temperature,
            pollution: tile_type.default_pollution(),
            ground_water: tile_type.default_ground_water(),
            humidity: tile_type.default_humidity(),
            soil: tile_type.default_soil(),
        }
    }

    pub fn fertility(&self) -> f32 {
        return self.soil + self.humidity - self.pollution;
    }

    // TODO: make temperature a dynamic attribute
    pub fn evaporation(&self) -> f32 {
        // Each tile could have a default value?
        return self.ground_water * self.temperature * EVAPORATION_FACTOR;
    }

    // TODO: tiletype should probably influence the overflow level?
    pub fn overflow_level(&self) -> f32 {
        return self.soil;
    }

    // TODO: mountains will run out of soil, should volcanoes add soil?
    // TODO: save volcano points and produce soil from them + raise elevation
    pub fn erosion_rate(&self, overflow: f32) -> f32 {
        // will return 0 if there is no overflow
        return EROSION_FACTOR * (1.0 - self.soil) * overflow;
    }

    pub fn apply_erosion(&self, overflow: f32) -> f32 {
        let mut erosion = self.erosion_rate(overflow);
        let erosion_effect_on_soil = self.soil.min(erosion);
        erosion -= erosion_effect_on_soil;
        erosion
    }

    pub fn update_deposits(&mut self, overflow: f32, erosion: f32) {
        self.ground_water += overflow;
        self.soil += erosion;
    }

    pub fn precipitation(&self) -> f32 {
        if self.tile_type.precipitation_factor().pick_random() {
            return self.humidity * PRECIPITATION_FACTOR;
        }

        0.0
    }
}
