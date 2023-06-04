use bevy::prelude::*;
use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_mod_picking::prelude::*;

use hexx::shapes;
use hexx::*;

use rand::prelude::SliceRandom;
use std::collections::{HashMap, HashSet};

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
        .run();
}

////////////////////// GLOBAL ATTRIBUTES /////////////////
// TODO: should be stored in a resource and used to generate the map
const ELEVATION: f32 = 10.0;
// Vulcanism -> Spawns mountains
const VOLCANISM: f32 = 3.0;
// How many rings to traverse from the volcano to spawn mountains
const MOUNTAIN_SPREAD: u32 = MAP_RADIUS * 60 / 100;
// How much to increase the elevation by
const ELEVATION_INCREMENT: f32 = 0.5;
// Elevation where sea level is
const SEA_LEVEL: f32 = 5.0;
// Planet Age
const PLANET_AGE: u128 = 0;
// Base Temperature
const BASE_TEMPERATURE: f32 = 20.0;

////////////////////// GRID //////////////////////

/// World size of the hexagons (outer radius)
const HEX_SIZE: Vec2 = Vec2::splat(2.0);
/// Map radius
const MAP_RADIUS: u32 = 20;

// TODO: store tile type + innate attributes instead of the material
#[derive(Resource)]
struct Map {
    entities: HashMap<Hex, (Entity, Handle<StandardMaterial>)>,
}

// To avoid having a fixed probability, we can make it a function of distance
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

/// Hex grid setup
fn setup_grid(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let layout = HexLayout {
        orientation: HexOrientation::pointy(),
        hex_size: HEX_SIZE,
        ..default()
    };

    // Load assets
    let mountain_handle: Handle<Mesh> =
        asset_server.load("mountain/Mountain.gltf#Mesh0/Primitive0");
    let waste_handle = asset_server.load("12 Wastes.gltf#Mesh0/Primitive0");
    let hills_handle = asset_server.load("04_Hills.gltf#Mesh0/Primitive0");

    // TODO: load assets as resource and use scenes
    // let gltf_handle = asset_server.load("mountain/Mountain.gltf#Scene0");

    let all_hexes: Vec<Hex> = shapes::hexagon(Hex::ZERO, MAP_RADIUS).collect();
    let altitude_map = generate_altitude_map(&all_hexes);

    // Spawn tiles
    let entities = all_hexes
        .into_iter()
        .map(|hex| {
            // Random generation
            let rand_num = rand::random::<u8>() % 4;

            // Hex position
            let pos = layout.hex_to_world_pos(hex);
            let mut altitude = altitude_map.get(&hex).unwrap().clone();

            // Setup meshes/material handles
            let mesh_handle;
            let material_handle;

            // Use random number to choose what tile to create
            // TODO: determine tile type from altitude, temperature, humidity
            if rand_num == 0 {
                mesh_handle = mountain_handle.clone();
                material_handle = asset_server.load("mountain/Mountain.gltf#Material0");
            } else if rand_num == 1 {
                mesh_handle = waste_handle.clone();
                material_handle = asset_server.load("waste/Waste.gltf#Material0");
            } else {
                mesh_handle = hills_handle.clone();
                material_handle = materials.add(StandardMaterial {
                    base_color: Color::DARK_GREEN,
                    metallic: 0.5,
                    ..Default::default()
                })
            }

            let id = commands
                .spawn((
                    PbrBundle {
                        transform: Transform::from_xyz(pos.x, altitude, pos.y)
                            .with_scale(Vec3::splat(2.0)),
                        mesh: mesh_handle.clone(),
                        material: material_handle.clone(),
                        ..default()
                    },
                    // SceneBundle {
                    //     scene: gltf_handle.clone(),
                    //     transform: Transform::from_xyz(pos.x, altitude, pos.y)
                    //         .with_scale(Vec3::splat(1.71)),
                    //     ..Default::default()
                    // },
                    PickableBundle::default(), // <- Makes the mesh pickable.
                    RaycastPickTarget::default(), // <- Needed for the raycast backend.
                ))
                .id();

            (hex, (id, material_handle.clone()))
        })
        .collect();

    commands.insert_resource(Map { entities });
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
            ..default()
        },
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        transform,
        ..default()
    });
}
