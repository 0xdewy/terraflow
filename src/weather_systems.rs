use core::panic;

use bevy::prelude::*;

use bevy::transform::commands;
use bevy_mod_picking::prelude::*;

use crate::components::{
    ElevationBundle, HexCoordinates, HigherNeighbours, Humidity, IncomingOverflow, LowerNeighbours,
    Neighbours, OutgoingOverflow, PendingHumidityRedistribution, Temperature, TileTypeChanged,
};
use crate::terrain::{TileAssets, TileType, WeatherEffects};
use crate::utils::RandomSelection;
use crate::world::{
    EcosystemAttributes, ElevationAttributes, ErosionAttributes, MapAttributes,
    TemperatureAttributes,
};
use crate::{pointy_layout, DebugWeatherBundle, Epochs, GameStates};

// TODO: move this to a config file
pub const SIGMOID_STEEPNESS: f32 = 1.0;

// TODO: move to utils file
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

///////////////////////////////// Terrain Changes /////////////////////////////////////////
/// Gives this entity a new tiletype if weather conditions are met

pub fn morph_terrain_system(
    mut debug: ResMut<Epochs>,
    mut commands: Commands,
    mut query: Query<(Entity, &ElevationBundle, &Humidity, &mut TileType)>,
    elevation_attributes: Res<ElevationAttributes>,
    ecosystem_attributes: Res<EcosystemAttributes>,
) {
    debug.fn_order.push("morph_terrain_system".to_string());
    for (entity, elevation, humidity, mut tile_type) in query.iter_mut() {
        // humidity effects
        let mut tile_probabilities = humidity.apply_weather(&tile_type);
        tile_probabilities.extend(
            (
                &elevation.water,
                &elevation.soil,
                &ecosystem_attributes.terrain_change_sensitivity,
            )
                .apply_weather(&tile_type),
        );
        tile_probabilities
            .extend((&elevation.bedrock, &*elevation_attributes).apply_weather(&tile_type));

        let new_tile = tile_probabilities.pick_random();
        if new_tile != *tile_type {
            *tile_type = new_tile;
            commands.entity(entity).insert(TileTypeChanged);
        }
    }
}

pub fn update_terrain_assets(
    mut debug: ResMut<Epochs>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &TileType,
        &HexCoordinates,
        &ElevationBundle,
        &mut Transform,
        &mut Handle<Mesh>,
        &mut Handle<StandardMaterial>,
        &TileTypeChanged,
    )>,
    tile_assets: Res<TileAssets>,
    mut next_state: ResMut<NextState<GameStates>>,
    elevation_attributes: Res<ElevationAttributes>,
    map_attributes: Res<MapAttributes>,
) {
    debug.fn_order.push("update_terrain_assets".to_string());
    for (
        entity,
        tile_type,
        hex,
        altitude,
        mut transform,
        mut mesh_handle,
        mut material_handle,
        _,
    ) in query.iter_mut()
    {
        let world_pos = pointy_layout(map_attributes.hex_size).hex_to_world_pos(hex.0);

        let total_height = altitude.bedrock.value
            + ((altitude.soil.value + altitude.water.value)
                * elevation_attributes.soil_and_water_height_display_factor);

        let (new_mesh_handle, new_material_handle) = tile_assets.get_mesh_and_material(tile_type);

        // remove picking components before the update
        commands.entity(entity).remove::<PickableBundle>();
        commands.entity(entity).remove::<RaycastPickTarget>();

        // update entity with new mesh, material and transform
        *transform = Transform::from_xyz(world_pos.x, total_height, world_pos.y)
            .with_scale(Vec3::splat(2.0));
        *mesh_handle = new_mesh_handle;
        *material_handle = new_material_handle;

        // add back picking components after the update
        commands.entity(entity).insert(PickableBundle::default());
        commands.entity(entity).insert(RaycastPickTarget::default());

        commands.entity(entity).remove::<TileTypeChanged>();
    }

    // Finish epoch
    next_state.set(GameStates::Waiting);
}

/////////////////////////////////Weather Systems//////////////////////////////////////////////////

pub fn evaporation_system(
    mut debug: ResMut<Epochs>,
    mut query: Query<(
        &mut ElevationBundle,
        &mut Humidity,
        &mut DebugWeatherBundle,
        &Temperature,
        &TileType,
    )>,
    ecosystem: Res<EcosystemAttributes>,
    temperature_attributes: Res<TemperatureAttributes>,
) {
    debug.fn_order.push("evaporation_system".to_string());
    for (mut elevation, mut humidity, mut weather, temperature, tile_type) in query.iter_mut() {
        // Normalize temperature to be between 0 and 1
        let normalized_temperature = (temperature.value / temperature_attributes.base_temperature)
            .max(0.0)
            .min(1.0);

        // Ocean tiles are set to produce more evaporation to supply the planet with humidity
        let tile_factor = match tile_type {
            TileType::Ocean => 4.0,
            _ => 1.0,
        };

        // Calculate evaporation
        weather.evaporation.value = (normalized_temperature
            * elevation.water.value
            * ecosystem.evaporation_factor
            * tile_factor)
            .max(0.0);

        assert!(weather.evaporation.value >= 0.0);
        humidity.value += weather.evaporation.value;

        let water_lost_to_evaporation = match tile_type {
            TileType::Ocean => 0.0,
            _ => weather.evaporation.value,
        };
        elevation.water.value = elevation.water.value - water_lost_to_evaporation.max(0.0);
    }
}

pub fn precipitation_system(
    mut debug: ResMut<Epochs>,
    mut query: Query<(
        &Humidity,
        &mut DebugWeatherBundle,
        &TileType,
        &mut ElevationBundle,
    )>,
    ecosystem_terrain: Res<EcosystemAttributes>,
) {
    debug.fn_order.push("precipitation_system".to_string());
    for (humidity, mut weather, tile_type, mut water_level) in query.iter_mut() {
        let tile_factor = match tile_type {
            TileType::Mountain => 0.9,
            TileType::Hills | TileType::Rocky => 0.3,
            TileType::Jungle | TileType::Swamp => 0.2,
            TileType::Ocean => 0.1,
            _ => 0.1,
        };

        let factor = sigmoid(SIGMOID_STEEPNESS * (humidity.value - 1.0));
        let precipitation_increment =
            factor * humidity.value * tile_factor * ecosystem_terrain.precipitation_factor;

        weather.precipitation.value = precipitation_increment;

        // water_level.water.value += tile_type.handle_precipitation(precipitation_increment);
        water_level.water.value += match tile_type {
            TileType::Ocean => 0.0,
            _ => precipitation_increment,
        }
    }

    assert!(query.iter().len() > 0);
}

///////////////////////////////// Terrain Analysis systems /////////////////////////////////////////

pub fn calculate_neighbour_heights_system(
    mut commands: Commands,
    mut debug: ResMut<Epochs>,
    mut query: Query<(Entity, &ElevationBundle, &Neighbours)>,
    neighbour_query: Query<&ElevationBundle>,
    mut next_game_state: ResMut<NextState<GameStates>>,
) {
    debug
        .fn_order
        .push("calculate_neighbour_heights_system".to_string());

    for (entity, elevation, neighbours) in query.iter_mut() {
        let this_entity_height =
            elevation.bedrock.value + elevation.water.value + elevation.soil.value;

        // Reset the lists of lower and higher neighbours
        let mut lower_neighbours = LowerNeighbours { ids: Vec::new() };
        let mut higher_neighbours = HigherNeighbours { ids: Vec::new() };

        for neighbour_id in &neighbours.ids {
            if let Ok(neighbour_elevation) = neighbour_query.get(*neighbour_id) {
                let neighbour_height = neighbour_elevation.bedrock.value
                    + neighbour_elevation.soil.value
                    + neighbour_elevation.water.value;

                // Include equal level into higher_neighbours
                if neighbour_height < this_entity_height {
                    lower_neighbours.ids.push((*neighbour_id, neighbour_height));
                } else {
                    higher_neighbours
                        .ids
                        .push((*neighbour_id, neighbour_height));
                }
            }
        }

        if higher_neighbours.ids.len() > 0 {
            commands.entity(entity).insert(higher_neighbours);
        }

        if lower_neighbours.ids.len() > 0 {
            commands.entity(entity).insert(lower_neighbours);
        }
    }

    next_game_state.set(GameStates::EpochRunning);
}

///////////////////////////////// Humidity systems /////////////////////////////////////////
///

pub fn redistribute_humidity_system(
    mut debug: ResMut<Epochs>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Humidity,
        &mut DebugWeatherBundle,
        &TileType,
        &HigherNeighbours,
    )>,
    mut incoming_humidity_query: Query<&mut PendingHumidityRedistribution>,
    ecosystem: Res<EcosystemAttributes>,
) {
    debug
        .fn_order
        .push("redistribute_humidity_system".to_string());

    for (_entity, mut humidity, mut weather, tile_type, higher_neighbours) in query.iter_mut() {
        let num_higher_neighbours = higher_neighbours.ids.len() as f32;
        let factor = sigmoid(SIGMOID_STEEPNESS * (humidity.value - 1.0));
        let humidity_to_escape = humidity.value * factor * ecosystem.humidity_escape_factor;
        assert!(humidity_to_escape >= 0.0);

        for &(neighbour_id, _neighbour_height) in &higher_neighbours.ids {
            let proportion = 1.0 / num_higher_neighbours;
            let humidity_for_neighbour = humidity_to_escape * proportion;

            if humidity_for_neighbour < 0.0 {
                continue;
            }

            incoming_humidity_query
                .get_mut(neighbour_id)
                .ok()
                .map(|mut incoming_humidity| {
                    incoming_humidity.value += humidity_for_neighbour;
                });
        }
        humidity.value = (humidity.value - humidity_to_escape).max(0.0);
        weather.humidity_sent.value = humidity_to_escape;
        weather.humidity_received.value = 0.0;
    }
    debug
        .fn_order
        .push("finished redistribute_humidity_system".to_string());
}

pub fn apply_humidity_redistribution(
    mut debug: ResMut<Epochs>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Humidity,
        &mut DebugWeatherBundle,
        &mut PendingHumidityRedistribution,
    )>,
    mut next_state: ResMut<NextState<GameStates>>,
) {
    debug
        .fn_order
        .push("apply_humidity_redistribution".to_string());
    for (entity, mut humidity, mut weather, mut redistribution) in query.iter_mut() {
        humidity.value += redistribution.value;

        weather.humidity_received.value = redistribution.value;
        redistribution.value = 0.0;
    }

    next_state.set(GameStates::EpochFinish);
    debug
        .fn_order
        .push("finished apply_humidity_redistribution".to_string());
}

// takes in the overflow and creates a component for each lower neighbour, containing their share of the overflow
pub fn redistribute_overflow_system(
    mut debug: ResMut<Epochs>,
    mut query: Query<(
        Entity,
        &mut ElevationBundle,
        &LowerNeighbours,
        &TileType,
        &mut DebugWeatherBundle,
    )>,
    mut incoming_overflow_query: Query<&mut IncomingOverflow>,
    erosion_attributes: Res<ErosionAttributes>,
) {
    debug
        .fn_order
        .push("redistribute_overflow_system".to_string());

    // Create a component for each lower neighbour, containing their share of the overflow
    for (entity, mut elevation, lower_neighbours, tiletype, mut weather) in query.iter_mut() {
        // if there is an altitude difference, but no lower neighbours, something is wrong
        assert!(lower_neighbours.ids.len() > 0);

        let overflow_factor = sigmoid(SIGMOID_STEEPNESS * (elevation.water.value - 1.0));
        // Nothing is lower than oceans so they don't overflow
        let water_overflow = match tiletype {
            TileType::Ocean => 0.0,
            _ => {
                (elevation.water.value - elevation.soil.value).max(0.0)
                    * erosion_attributes.overflow_factor
                    * overflow_factor
            }
        };

        if water_overflow <= 0.0 {
            continue;
        }

        // erosion is influenced by soil,water, + general erosion factor
        let normalized_overflow = weather.overflow.water / 1.0;
        let normalized_soil = elevation.soil.value / 1.0;
        let erosion_factor =
            (normalized_soil * normalized_overflow * erosion_attributes.erosion_factor).min(1.0);
        // this reduces the bedrock level so it needs to be 0-1 normalized to the current bedrock level
        let soil_overflow = erosion_factor * elevation.bedrock.value;

        let lowest_neighbour = crate::utils::get_lowest_neighbour(lower_neighbours);
        assert!(lowest_neighbour != entity);
        println!("lowest_neighbour: {:?}", lowest_neighbour);

        incoming_overflow_query
            .get_mut(lowest_neighbour)
            .ok()
            .map(|mut incoming_overflow| {
                incoming_overflow.water += water_overflow;
                incoming_overflow.soil += soil_overflow;
                println!(
                    "neighbour: {:?}, is receiving overflow: {:?} from: {:?}",
                    lowest_neighbour, water_overflow, entity
                );
            });

        // TODO: does soil need to be reduced also? this should probably be split amongs bedrock and soil
        // assert!(overflow.soil <= elevation.soil.value);
        // elevation.soil.value = (elevation.soil.value - overflow.soil).max(0.0);
        assert!(elevation.bedrock.value >= soil_overflow);
        elevation.bedrock.value = (elevation.bedrock.value - soil_overflow).max(0.0);
        elevation.water.value = (elevation.water.value - water_overflow).max(0.0);

        weather.overflow.water = water_overflow;
        weather.overflow.soil = soil_overflow;
    }
}

// Groundwater overflow is applied to this neighbours water level
pub fn apply_water_overflow(
    mut debug: ResMut<Epochs>,
    mut query: Query<(
        Entity,
        &mut ElevationBundle,
        &mut IncomingOverflow,
        &TileType,
        &mut DebugWeatherBundle,
    )>,
) {
    debug.fn_order.push("apply_water_overflow".to_string());

    for (entity, mut elevation, mut incoming_overflow, tile_type, mut weather) in query.iter_mut() {
        if incoming_overflow.water == 0.0 {
            incoming_overflow.soil = 0.0;
            continue;
        }

        elevation.water.value += match tile_type {
            TileType::Ocean => 0.0,
            _ => incoming_overflow.water,
        };

        elevation.soil.value += match tile_type {
            TileType::Ocean => 0.0,
            _ => incoming_overflow.soil,
        };

        weather.overflow_received.soil = incoming_overflow.soil;
        weather.overflow_received.water = incoming_overflow.water;

        incoming_overflow.water = 0.0;
        incoming_overflow.soil = 0.0;
    }
}
