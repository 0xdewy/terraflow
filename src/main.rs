use bevy::prelude::*;

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_mod_picking::prelude::*;

use hexx::shapes;
use hexx::*;

use rand::prelude::SliceRandom;
use std::collections::HashMap;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 0.1,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(CameraControllerPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_grid)
        .add_system(bevy::window::close_on_esc)
        .run();
}

////////////////////// GLOBAL ATTRIBUTES /////////////////
// TODO: should be stored in a resource and used to generate the map
const ELEVATION: f32 = 10.0;
// Vulcanism -> Spawns mountains
const VOLCANISM: f32 = 3.0;
// How many rings to traverse from the volcano to spawn mountains
const MOUNTAIN_SPREAD: u32 = MAP_RADIUS * 90 / 100;
// How much to increase the elevation by
const ELEVATION_INCREMENT: f32 = 0.5;
// Elevation where sea level is
const SEA_LEVEL: f32 = 3.0;
// Planet Age
const _PLANET_AGE: u128 = 0;
// Base Temperature
const _BASE_TEMPERATURE: f32 = 20.0;

////////////////////// GRID //////////////////////

/// World size of the hexagons (outer radius)
const HEX_SIZE: Vec2 = Vec2::splat(2.0);
/// Map radius
const MAP_RADIUS: u32 = 40;

// TODO: store tile type + innate attributes instead of the material
#[derive(Resource)]
struct Map {
    entities: HashMap<Hex, (Entity)>,
}

/// Hex grid setup
fn setup_grid(asset_server: Res<AssetServer>, mut commands: Commands) {
    let layout = HexLayout {
        orientation: HexOrientation::pointy(),
        hex_size: HEX_SIZE,
        ..default()
    };

    let tile_assets = load_tile_assets(&asset_server);

    let all_hexes: Vec<Hex> = shapes::hexagon(Hex::ZERO, MAP_RADIUS).collect();
    let altitude_map = generate_altitude_map(&all_hexes);

    // Spawn tiles
    let entities = all_hexes
        .into_iter()
        .map(|hex| {
            // Random generation
            let rand_num = rand::random::<u8>() % 4;

            // Hex position and altitude
            let pos = layout.hex_to_world_pos(hex);
            let mut altitude = altitude_map.get(&hex).unwrap().clone();

            // let scene_handle;
            // TODO: loading as scenes break the pickable plugin
            // if altitude <= SEA_LEVEL {
            //     scene_handle = tile_assets.ocean.clone();
            //     altitude = SEA_LEVEL;
            // } else if altitude >= ELEVATION * 0.8 {
            //     scene_handle = tile_assets.mountain.clone();
            // } else if altitude >= ELEVATION * 0.6 {
            //     scene_handle = tile_assets.hills.clone();
            // } else {
            //     scene_handle = tile_assets.grass.clone();
            // }


            let mesh_handle;
            let material_handle;

            if altitude <= SEA_LEVEL {
                mesh_handle = tile_assets.ocean_mesh.clone();
                material_handle = tile_assets.ocean_material.clone();
                altitude = SEA_LEVEL;
            } else if altitude >= ELEVATION * 0.8 {
                mesh_handle = tile_assets.mountain_mesh.clone();
                material_handle = tile_assets.mountain_material.clone();
            } else if altitude >= ELEVATION * 0.6 {
                mesh_handle = tile_assets.hills_mesh.clone();
                material_handle = tile_assets.hills_material.clone();
            } else {
                mesh_handle = tile_assets.grass_mesh.clone();
                material_handle = tile_assets.grass_material.clone();
            }

            let id = commands
                .spawn((
                    PbrBundle {
                        transform: Transform::from_xyz(pos.x, altitude, pos.y)
                            .with_scale(Vec3::splat(2.0)),
                        mesh: mesh_handle,
                        material: material_handle,
                        ..default()
                    },
                    // SceneBundle {
                    //     transform: Transform::from_xyz(pos.x, altitude, pos.y)
                    //         .with_scale(Vec3::splat(2.0)),
                    //     scene: scene_handle.clone(),
                    //     ..Default::default()
                    // },
                    PickableBundle::default(), // <- Makes the mesh pickable.
                    RaycastPickTarget::default(), // <- Needed for the raycast backend.
                ))
                .id();

            (hex, id)
        })
        .collect();

    commands.insert_resource(Map { entities });
}

/////////////////////////////// TileAssets ///////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct SceneTileAssets {
    pub desert: Handle<Scene>,
    pub dirt: Handle<Scene>,
    pub forest: Handle<Scene>,
    pub grass: Handle<Scene>,
    pub hills: Handle<Scene>,
    pub ice: Handle<Scene>,
    pub jungle: Handle<Scene>,
    pub mountain: Handle<Scene>,
    pub ocean: Handle<Scene>,
    pub plains: Handle<Scene>,
    pub rocky: Handle<Scene>,
    pub swamp: Handle<Scene>,
    pub waste: Handle<Scene>,
    pub water: Handle<Scene>,
}
pub fn load_tile_assets_scene(asset_server: &Res<AssetServer>) -> SceneTileAssets {
    SceneTileAssets {
        desert: asset_server.load("tiles/Desert.gltf#Scene0"),
        dirt: asset_server.load("tiles/Dirt.gltf#Scene0"),
        forest: asset_server.load("tiles/Forest.gltf#Scene0"),
        grass: asset_server.load("tiles/Grass.gltf#Scene0"),
        hills: asset_server.load("tiles/Hills.gltf#Scene0"),
        ice: asset_server.load("tiles/Ice.gltf#Scene0"),
        jungle: asset_server.load("tiles/Jungle.gltf#Scene0"),
        mountain: asset_server.load("tiles/Mountain.gltf#Scene0"),
        ocean: asset_server.load("tiles/Ocean.gltf#Scene0"),
        plains: asset_server.load("tiles/Plains.gltf#Scene0"),
        rocky: asset_server.load("tiles/Rocky.gltf#Scene0"),
        swamp: asset_server.load("tiles/Swamp.gltf#Scene0"),
        waste: asset_server.load("tiles/Waste.gltf#Scene0"),
        water: asset_server.load("tiles/Water.gltf#Scene0"),
    }
}

pub struct TileAssets {
    pub desert_mesh: Handle<Mesh>,
    pub desert_material: Handle<StandardMaterial>,

    pub dirt_mesh: Handle<Mesh>,
    pub dirt_material: Handle<StandardMaterial>,

    pub forest_mesh: Handle<Mesh>,
    pub forest_material: Handle<StandardMaterial>,

    pub grass_mesh: Handle<Mesh>,
    pub grass_material: Handle<StandardMaterial>,

    pub hills_mesh: Handle<Mesh>,
    pub hills_material: Handle<StandardMaterial>,

    pub ice_mesh: Handle<Mesh>,
    pub ice_material: Handle<StandardMaterial>,

    pub jungle_mesh: Handle<Mesh>,
    pub jungle_material: Handle<StandardMaterial>,

    pub mountain_mesh: Handle<Mesh>,
    pub mountain_material: Handle<StandardMaterial>,

    pub ocean_mesh: Handle<Mesh>,
    pub ocean_material: Handle<StandardMaterial>,

    pub plains_mesh: Handle<Mesh>,
    pub plains_material: Handle<StandardMaterial>,

    pub rocky_mesh: Handle<Mesh>,
    pub rocky_material: Handle<StandardMaterial>,

    pub swamp_mesh: Handle<Mesh>,
    pub swamp_material: Handle<StandardMaterial>,

    pub waste_mesh: Handle<Mesh>,
    pub waste_material: Handle<StandardMaterial>,

    pub water_mesh: Handle<Mesh>,
    pub water_material: Handle<StandardMaterial>,
}

pub fn load_tile_assets(asset_server: &Res<AssetServer>) -> TileAssets {
    TileAssets {
        desert_mesh: asset_server.load("tiles/Desert.gltf#Mesh0/Primitive0"),
        desert_material: asset_server.load("tiles/Desert.gltf#Material0"),

        dirt_mesh: asset_server.load("tiles/Dirt.gltf#Mesh0/Primitive0"),
        dirt_material: asset_server.load("tiles/Dirt.gltf#Material0"),

        forest_mesh: asset_server.load("tiles/Forest.gltf#Mesh0/Primitive0"),
        forest_material: asset_server.load("tiles/Forest.gltf#Material0"),

        grass_mesh: asset_server.load("tiles/Grass.gltf#Mesh0/Primitive0"),
        grass_material: asset_server.load("tiles/Grass.gltf#Material0"),

        hills_mesh: asset_server.load("tiles/Hills.gltf#Mesh0/Primitive0"),
        hills_material: asset_server.load("tiles/Hills.gltf#Material0"),

        ice_mesh: asset_server.load("tiles/Ice.gltf#Mesh0/Primitive0"),
        ice_material: asset_server.load("tiles/Ice.gltf#Material0"),

        jungle_mesh: asset_server.load("tiles/Jungle.gltf#Mesh0/Primitive0"),
        jungle_material: asset_server.load("tiles/Jungle.gltf#Material0"),

        mountain_mesh: asset_server.load("tiles/Mountain.gltf#Mesh0/Primitive0"),
        mountain_material: asset_server.load("tiles/Mountain.gltf#Material0"),

        ocean_mesh: asset_server.load("tiles/Ocean.gltf#Mesh0/Primitive0"),
        ocean_material: asset_server.load("tiles/Ocean.gltf#Material0"),

        plains_mesh: asset_server.load("tiles/Plains.gltf#Mesh0/Primitive0"),
        plains_material: asset_server.load("tiles/Plains.gltf#Material0"),

        rocky_mesh: asset_server.load("tiles/Rocky.gltf#Mesh0/Primitive0"),
        rocky_material: asset_server.load("tiles/Rocky.gltf#Material0"),

        swamp_mesh: asset_server.load("tiles/Swamp.gltf#Mesh0/Primitive0"),
        swamp_material: asset_server.load("tiles/Swamp.gltf#Material0"),

        waste_mesh: asset_server.load("tiles/Waste.gltf#Mesh0/Primitive0"),
        waste_material: asset_server.load("tiles/Waste.gltf#Material0"),

        water_mesh: asset_server.load("tiles/Water.gltf#Mesh0/Primitive0"),
        water_material: asset_server.load("tiles/Water.gltf#Material0"),
    }
}

/////////////////////////////// Altitude ///////////////////////////////////////////////////////

// The base probability is 1 when distance is 0 (at the volcano)
// The probability decreases linearly as the distance increases, down to 0 at the furthest point
fn increment_height(current_height: &mut f32, distance: u32) {
    let distance_f32 = distance as f32;
    let probability = 1.0 - (distance_f32 / MOUNTAIN_SPREAD as f32);

    if rand::random::<f32>() < probability {
        *current_height += ELEVATION_INCREMENT;
    }
}

fn generate_altitude_map(all_hexes: &Vec<Hex>) -> HashMap<Hex, f32> {
    let mut rng = rand::thread_rng();

    let mut altitude_map: HashMap<Hex, f32> = all_hexes.iter().map(|hex| (*hex, 0.0)).collect();

    // Randomly pick hexes to start elevation gains from
    let volcano_hexes: Vec<Hex> = all_hexes
        .choose_multiple(&mut rng, VOLCANISM as usize)
        .cloned()
        .collect();

    // Set volcano hexes' initial altitude to ELEVATION_INCREMENT
    for hex in &volcano_hexes {
        altitude_map.insert(*hex, ELEVATION_INCREMENT);
    }

    let mut max_height = 0.0;
    while max_height < ELEVATION {
        for hex in &volcano_hexes {
            // Update the volcano height
            increment_height(altitude_map.get_mut(hex).unwrap(), 0);
            max_height = max_height.max(altitude_map[hex] as f32);

            // Update the neighbors' heights based on their distance to the volcano
            for rings_traversed in 1..=MOUNTAIN_SPREAD as u32 {
                for neighbour in hex.ring(rings_traversed) {
                    if let Some(height) = altitude_map.get_mut(&neighbour) {
                        increment_height(height, rings_traversed);
                        max_height = max_height.max(*height as f32);
                    }
                }
            }
        }
    }
    altitude_map
}

////////////////////// CAMERA MOVEMENT //////////////////////
///
/// /// 3D Orthogrpahic camera setup
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
