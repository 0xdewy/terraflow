use bevy::prelude::*;

use bevy_mod_picking::prelude::*;

use crate::components::{
    ElevationBundle, Evaporation, HexCoordinates, HigherNeighbours, Humidity, IncomingOverflow,
    LowerNeighbours, Neighbours, Overflow, PendingHumidityRedistribution, Precipitation,
    Temperature, TileTypeChanged,
};
use crate::tiles::{TileAssets, TileType, WeatherEffects};
use crate::utils::RandomSelection;
use crate::world::{
    EcosystemAttributes, ElevationAttributes, ErosionAttributes, MapAttributes,
    TemperatureAttributes,
};
use crate::{pointy_layout, Epochs, GameStates};

// TODO: move this to a config file
pub const SIGMOID_STEEPNESS: f32 = 2.0;

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
    next_state.set(GameStates::Loading);
}

/////////////////////////////////Weather Systems//////////////////////////////////////////////////

pub fn evaporation_system(
    mut debug: ResMut<Epochs>,
    mut query: Query<(
        &mut ElevationBundle,
        &mut Humidity,
        &mut Evaporation,
        &Temperature,
        &TileType,
    )>,
    ecosystem: Res<EcosystemAttributes>,
    temperature_attributes: Res<TemperatureAttributes>,
) {
    debug.fn_order.push("evaporation_system".to_string());
    for (mut elevation, mut humidity, mut evaporation, temperature, tile_type) in query.iter_mut() {
        // Normalize temperature to be between 0 and 1
        let normalized_temperature = (temperature.value / temperature_attributes.base_temperature)
            .max(0.0)
            .min(1.0);

        // Calculate evaporation
        let factor = sigmoid(SIGMOID_STEEPNESS * (elevation.water.value - 1.0));
        evaporation.value = (normalized_temperature
            * elevation.water.value
            * factor
            * ecosystem.evaporation_factor)
            .max(0.0);

        // Verify evaporation is non-negative
        assert!(evaporation.value >= 0.0);
        // Update humidity and water level
        humidity.value += evaporation.value;
        elevation.water.value =
            (elevation.water.value - tile_type.handle_evaporation(evaporation.value)).max(0.0);
    }
}

pub fn precipitation_system(
    mut debug: ResMut<Epochs>,
    mut query: Query<(
        &Humidity,
        &mut Precipitation,
        &TileType,
        &mut ElevationBundle,
    )>,
    ecosystem_terrain: Res<EcosystemAttributes>,
) {
    debug.fn_order.push("precipitation_system".to_string());
    for (humidity, mut precipitation, tile_type, mut water_level) in query.iter_mut() {
        precipitation.value = 0.0;

        let factor = sigmoid(SIGMOID_STEEPNESS * (humidity.value - 1.0));
        let precipitation_increment = factor
            * humidity.value
            * tile_type.precipitation_factor(ecosystem_terrain.precipitation_factor);
        // println!(
        //     "precipitation: {}, humidity: {}",
        //     precipitation_increment, humidity.value
        // );

        precipitation.value = precipitation_increment;

        water_level.water.value += tile_type.handle_precipitation(precipitation_increment);
    }

    assert!(query.iter().len() > 0);
}

///////////////////////////////// Terrain Analysis systems /////////////////////////////////////////
///

pub fn calculate_neighbour_heights_system(
    mut debug: ResMut<Epochs>,
    mut query: Query<(
        Entity,
        &ElevationBundle,
        &Neighbours,
        &mut LowerNeighbours,
        &mut HigherNeighbours,
    )>,
    neighbour_query: Query<&ElevationBundle>,
    mut next_game_state: ResMut<NextState<GameStates>>,
) {
    debug
        .fn_order
        .push("calculate_neighbour_heights_system".to_string());
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

    next_game_state.set(GameStates::EpochRunning);
}

///////////////////////////////// Humidity systems /////////////////////////////////////////
///

pub fn redistribute_humidity_system(
    mut debug: ResMut<Epochs>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Humidity, &TileType, &HigherNeighbours)>,
    mut incoming_humidity_query: Query<&mut PendingHumidityRedistribution>,
) {
    debug
        .fn_order
        .push("redistribute_humidity_system".to_string());
    for (_entity, mut humidity, _tile_type, higher_neighbours) in query.iter_mut() {
        let factor = sigmoid(SIGMOID_STEEPNESS * (humidity.value - 1.0));

        let humidity_to_escape = humidity.value * factor;

        if humidity_to_escape <= 0.0 {
            continue;
        }

        let num_higher_neighbours = higher_neighbours.ids.len() as f32;

        for &(neighbour_id, _neighbour_height) in &higher_neighbours.ids {
            let proportion = 1.0 / num_higher_neighbours;
            let humidity_for_neighbour = humidity_to_escape * proportion;

            if humidity_for_neighbour > 0.0 {
                // Check if the neighbour already has a PendingHumidityRedistribution
                if let Ok(mut incoming_humidity) = incoming_humidity_query.get_mut(neighbour_id) {
                    incoming_humidity.amount += humidity_for_neighbour;
                } else {
                    // If not, add a PendingHumidityRedistribution component to the neighbour
                    commands
                        .entity(neighbour_id)
                        .insert(PendingHumidityRedistribution {
                            amount: humidity_for_neighbour,
                        });
                }
            }
        }
        humidity.value = (humidity.value - humidity_to_escape).max(0.0);
    }
}

pub fn apply_humidity_redistribution(
    mut debug: ResMut<Epochs>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Humidity, &PendingHumidityRedistribution)>,
    mut next_state: ResMut<NextState<GameStates>>,
) {
    debug
        .fn_order
        .push("apply_humidity_redistribution".to_string());
    for (entity, mut humidity, redistribution) in query.iter_mut() {
        // println!(
        //     "entity: {:?}, humidity {}, redistribution: {:?}",
        //     entity, humidity.value, redistribution.amount
        // );
        humidity.value += redistribution.amount;
        // Remove the PendingHumidityRedistribution component
        commands
            .entity(entity)
            .remove::<PendingHumidityRedistribution>();
    }

    next_state.set(GameStates::EpochFinish);
}

///////////////////////////////// Overflow systems /////////////////////////////////////////
pub fn calculate_overflow_system(
    mut debug: ResMut<Epochs>,
    mut query: Query<(Entity, &ElevationBundle, &mut Overflow, &TileType)>,
    erosion_attributes: Res<ErosionAttributes>,
) {
    debug.fn_order.push("calculate_overflow_system".to_string());
    for (_entity, elevation, mut overflow, tiletype) in query.iter_mut() {
        overflow.water = tiletype.overflow_amount(elevation.water.value, elevation.soil.value);

        overflow.soil = overflow.water * elevation.soil.value * erosion_attributes.erosion_factor;
        // println!(
        //     "water level {} overflow.value: {}, tiletype: {:?}",
        //     elevation.water.value, overflow.value, tiletype
        // );
    }
}

// takes in the overflow and creates a component for each lower neighbour, containing their share of the overflow
pub fn redistribute_overflow_system(
    mut debug: ResMut<Epochs>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Overflow,
        &mut ElevationBundle,
        &LowerNeighbours,
    )>,
    mut incoming_overflow_query: Query<&mut IncomingOverflow>,
    erosion_attributes: Res<ErosionAttributes>,
) {
    debug
        .fn_order
        .push("redistribute_overflow_system".to_string());
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
            continue;
        }

        let num_lower_neighbours = lower_neighbours.ids.len() as f32;
        // if there is an altitude difference, but no lower neighbours, something is wrong
        assert!(num_lower_neighbours > 0.0);

        // Calculate the overflow for each neighbour
        for &(neighbour_id, neighbour_height) in &lower_neighbours.ids {
            let difference = (this_entity_height - neighbour_height).max(0.0);
            let proportion = difference / total_difference;
            let water_overflow_for_neighbour = overflow.water * proportion;
            let soil_overflow_for_neighbour = water_overflow_for_neighbour
                * elevation.soil.value
                * erosion_attributes.erosion_factor;

            if let Ok(mut incoming_overflow) = incoming_overflow_query.get_mut(neighbour_id) {
                // If there is an existing IncomingOverflow, add to it
                incoming_overflow.water += water_overflow_for_neighbour;
                incoming_overflow.soil += soil_overflow_for_neighbour;
            } else {
                // If there is not an existing IncomingOverflow, create one
                commands.entity(neighbour_id).insert(IncomingOverflow {
                    water: water_overflow_for_neighbour,
                    soil: soil_overflow_for_neighbour,
                });
            }
        }

        // TODO: assert that soil_lost_to_overflow is not greater than soil
        elevation.water.value = (elevation.water.value - overflow.water).max(0.0);
        elevation.soil.value = (elevation.soil.value - overflow.soil).max(0.0);
        elevation.bedrock.value = (elevation.bedrock.value - overflow.soil).max(0.0);
    }
}

// Groundwater overflow is applied to this neighbours water level
pub fn apply_water_overflow(
    mut debug: ResMut<Epochs>,
    mut query: Query<
        (Entity, &mut ElevationBundle, &IncomingOverflow, &TileType),
        Added<IncomingOverflow>,
    >,
    mut commands: Commands,
) {
    debug.fn_order.push("apply_water_overflow".to_string());

    for (entity, mut elevation, incoming_overflow, tile_type) in query.iter_mut() {
        elevation.water.value += tile_type.handle_ground_water(incoming_overflow.water);
        elevation.soil.value += tile_type.handle_soil(incoming_overflow.soil);
        commands.entity(entity).remove::<IncomingOverflow>();
    }
}
