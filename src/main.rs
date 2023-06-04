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
const ELEVATION: u16 = 15;
// Vulcanism -> Spawns mountains
const VOLCANISM: u16 = 3;
// Elevation where sea level is
const SEA_LEVEL: u16 = 5;
// Planet Age
const PLANET_AGE: u128 = 0;
// Base Temperature
const BASE_TEMPERATURE: u16 = 20;

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

fn generate_altitude_map(all_hexes: Vec<Hex>) -> HashMap<Hex, u16> {
    // TODO: should we create a wrappable hex-map?
    // let wrappable_map = hexx::hex_map::HexMap::new(MAP_RADIUS).with_center(Hex::ZERO);

    // Randly pick VOLCANISM number of hexes from the map
    let mut rng = rand::thread_rng();
    let mut shuffled_hexes = all_hexes.clone();
    shuffled_hexes.shuffle(&mut rng);
    let volcano_hexes: HashSet<_> = shuffled_hexes
        .into_iter()
        .take(VOLCANISM as usize)
        .collect();

    // Create a hashmap of hex grid to altitude. All volcanoes start at 1, everything else 0
    let mut altitude_map: HashMap<Hex, u16> = all_hexes
        .into_iter()
        .map(|hex| match volcano_hexes.contains(&hex) {
            true => (hex, 1),
            false => (hex, 0),
        })
        .collect();

    // Loop through all hexes, increasing the height of the volcanism points, and surrounding hexes
    // The loop should finish once a volcano has reached ELEVATION
    let mut max_height = 1;
    while max_height < ELEVATION {
        volcano_hexes.iter().for_each(|hex| {
            let neighbours = hex.ring(1);

            // Increase the height of the volcano
            let mut height = altitude_map.get(&hex).unwrap().clone();
            height += 1;
            altitude_map.insert(*hex, height);

            // Update max height
            if height > max_height {
                max_height = height;
            }

            // Increase the height of the neighbours by 1
            neighbours.into_iter().for_each(|neighbour| {
                // TODO: add randomness
                let mut height = altitude_map.get(&neighbour).expect("Failed to find neighbour for hex").clone();
                height += 1;
                altitude_map.insert(neighbour, height);
            });

            // TODO: increase 2nd degree neighbours height with lower odds
        });
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
    let altitude_map = generate_altitude_map(all_hexes.clone());

    // Spawn tiles
    let entities = all_hexes
        .into_iter()
        .map(|hex| {
            println!("Hex: {:?}", hex);
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
                        transform: Transform::from_xyz(pos.x, altitude as f32, pos.y)
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
