use bevy::prelude::*;

use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_mod_picking::prelude::*;

use hexx::shapes;
use hexx::*;

use std::collections::HashMap;

mod tiles;
mod world;

use world::{AltitudeGenerator, TemperatureGenerator, TileTypeGenerator, WorldAttributes};

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

// TODO: store tile type + innate attributes instead of the material
#[derive(Resource)]
struct Map {
    entities: HashMap<Hex, Entity>,
}

// pub struct Terrain {
//     pub tile_type: TileType,
//     pub altitude: f32,
//     pub temperature: f32,
// }

// pub struct TerrainMap {
//     pub map: HashMap<Hex, Terrain>,
// }

/// Hex grid setup
fn setup_grid(asset_server: Res<AssetServer>, mut commands: Commands) {
    let layout = HexLayout {
        orientation: HexOrientation::pointy(),
        hex_size: world::HEX_SIZE,
        ..default()
    };

    let world = WorldAttributes::default();

    let tile_assets = tiles::TileAssets::new(&asset_server);

    let all_hexes: Vec<Hex> = shapes::hexagon(Hex::ZERO, world::MAP_RADIUS).collect();

    let altitude_map = world.altitude.generate_altitude_map(&all_hexes);
    let temperature_map = world.temperature.generate_temperature_map(&altitude_map);

    // Spawn tiles
    let entities = all_hexes
        .into_iter()
        .map(|hex| {
            // Hex position and altitude
            let pos = layout.hex_to_world_pos(hex);
            let altitude = altitude_map.get(&hex).unwrap().clone();
            let temperature = temperature_map.get(&hex).unwrap().clone();
            println!(
                "latitude {:?}: altitude: {}, temperature: {}",
                hex.y, altitude, temperature
            );
            let tile_type = world.from_geography(hex.y as f32, altitude, temperature);
            let (mesh_handle, material_handle) = tile_assets.mesh_and_material(&tile_type);

            let id = commands
                .spawn((
                    PbrBundle {
                        transform: Transform::from_xyz(pos.x, altitude, pos.y)
                            .with_scale(Vec3::splat(2.0)),
                        mesh: mesh_handle,
                        material: material_handle,
                        ..default()
                    },
                    PickableBundle::default(), // <- Makes the mesh pickable.
                    RaycastPickTarget::default(), // <- Needed for the raycast backend.
                ))
                .id();

            (hex, id)
        })
        .collect();

    commands.insert_resource(Map { entities });
}

// Epoch
// plot evaporation rates
// probability that humidity becomes rainfall, otherwise it moves on (mountains force rainfall)
//  ---> where does it move on? is this also random?

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
