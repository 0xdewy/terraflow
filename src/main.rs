use bevy::prelude::*;
use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_mod_picking::prelude::*;

use hexx::shapes;
use hexx::*;

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
        .run();
}

////////////////////// GRID //////////////////////
///
/// World size of the hexagons (outer radius)
const HEX_SIZE: Vec2 = Vec2::splat(2.0);
/// Map radius
const MAP_RADIUS: u32 = 20;

#[derive(Resource)]
struct Map {
    entities: HashMap<Hex, (Entity, Handle<StandardMaterial>)>,
}

/// Hex grid setup
fn setup_grid(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let layout = HexLayout {
        orientation: HexOrientation::pointy(),
        hex_size: HEX_SIZE,
        ..default()
    };

    // Meshes
    let mountain_handle: Handle<Mesh> =
        asset_server.load("mountain/Mountain.gltf#Mesh0/Primitive0");
    let waste_handle = asset_server.load("12 Wastes.gltf#Mesh0/Primitive0");
    let hills_handle = asset_server.load("04_Hills.gltf#Mesh0/Primitive0");


    // TODO: load assets as resource and use scenes
    // let gltf_handle = asset_server.load("mountain/Mountain.gltf#Scene0");

    // Spawn tiles
    let entities = shapes::hexagon(Hex::ZERO, MAP_RADIUS)
        .map(|hex| {
            // Random generation
            let rand_num = rand::random::<u8>() % 4;

            // Hex position
            let pos = layout.hex_to_world_pos(hex);
            let mut altitude = 0f32;

            // Setup meshes/material
            let mesh_handle;
            let material_handle;

            // Use random number to choose what tile to create
            if rand_num == 0 {
                altitude = 0f32;
                mesh_handle = mountain_handle.clone();
                material_handle = asset_server.load("mountain/Mountain.gltf#Material0");
            } else if rand_num == 1 {
                mesh_handle = waste_handle.clone();
                material_handle = asset_server.load("waste/Waste.gltf#Material0");
            } else {
                altitude = 0f32;
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
