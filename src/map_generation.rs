
use std::collections::HashMap;

use hexx::Hex;

use crate::world::{ElevationAttributes, TemperatureAttributes};

pub fn increment_height(
    elevation_attributes: &ElevationAttributes,
    current_height: &mut f32,
    distance_from_volcano: u16,
) {
    let probability = 1.0 - (distance_from_volcano as f32 / elevation_attributes.mountain_spread);
    if rand::random::<f32>() < probability {
        *current_height += elevation_attributes.elevation_increment;
    }
}

pub fn generate_altitude_map(
    elevation_attributes: &ElevationAttributes,
    all_hexes: &Vec<Hex>,
    volcano_hexes: &Vec<Hex>,
) -> HashMap<Hex, f32> {
    let mut altitude_map: HashMap<Hex, f32> = all_hexes.iter().map(|hex| (*hex, 0.0)).collect();
    for hex in volcano_hexes {
        altitude_map.insert(*hex, elevation_attributes.elevation_increment);
    }

    raise_volcanoes(elevation_attributes, &mut altitude_map, volcano_hexes);
    altitude_map
}

/*
 * This function mutates the altitude map to raise the volcanoes and
 * surrounding terrain
 *
 * the further a tile is from the volcano, the less likely it will have its
 * terrain raised
 *
 * the probability of a tile being raised is 1 - (distance from volcano / mountain spread)")
 */
fn raise_volcanoes(
    elevation_attributes: &ElevationAttributes,
    altitude_map: &mut HashMap<Hex, f32>,
    volcano_hexes: &Vec<Hex>,
) {
    let mut max_height = 0.0;
    while max_height < elevation_attributes.highest_elevation {
        for hex in volcano_hexes {
            increment_height(elevation_attributes, altitude_map.get_mut(hex).unwrap(), 0);
            max_height = max_height.max(altitude_map[hex]);
            for rings_traversed in 1..=elevation_attributes.mountain_spread as u16 {
                for neighbour in hex.ring(rings_traversed as u32) {
                    if let Some(height) = altitude_map.get_mut(&neighbour) {
                        increment_height(elevation_attributes, height, rings_traversed);
                    }
                }
            }
        }
    }
}

pub fn get_distances_from_volcanos(
    elevation_attributes: &ElevationAttributes,
    volcano_hexes: &Vec<Hex>,
) -> HashMap<Hex, Vec<u16>> {
    let mut distances_from_volcanos: HashMap<Hex, Vec<u16>> = HashMap::new();

    for hex in volcano_hexes {
        distances_from_volcanos.insert(*hex, vec![0]);
        for rings_traversed in 1..=elevation_attributes.mountain_spread as u16 {
            for neighbour in hex.ring(rings_traversed as u32) {
                let mut distances = distances_from_volcanos
                    .get(&neighbour)
                    .unwrap_or(&vec![])
                    .clone();
                distances.push(rings_traversed);
                distances_from_volcanos.insert(neighbour, distances.clone());
            }
        }
    }

    distances_from_volcanos
}

pub fn generate_temperature_map(
    temperature_attributes: &TemperatureAttributes,
    map_radius: u16,
    altitude_map: &HashMap<Hex, f32>,
) -> HashMap<Hex, f32> {
    altitude_map
        .iter()
        .map(|(hex, &altitude)| {
            (
                *hex,
                calculate_temperature(temperature_attributes, map_radius, altitude, hex.y as f32),
            )
        })
        .collect()
}

fn calculate_temperature(
    temperature_attributes: &TemperatureAttributes,
    map_radius: u16,
    altitude: f32,
    latitude: f32,
) -> f32 {
    let normalized_y = latitude.abs() / map_radius as f32;
    let latitude_temperature_mod =
        normalized_y * temperature_attributes.latitude_temperature_variation;
    let altitude_temperature_mod = if altitude > 0.0 {
        altitude * temperature_attributes.altitude_temperature_variation
    } else {
        0.0
    };
    temperature_attributes.base_temperature - altitude_temperature_mod - latitude_temperature_mod
}
