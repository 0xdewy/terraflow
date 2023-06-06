use bevy::prelude::*;
use hexx::*;

use rand::prelude::SliceRandom;
use std::collections::HashMap;

use super::assets::TileType;

/// World size of the hexagons (outer radius)
pub const HEX_SIZE: Vec2 = Vec2::splat(2.0);
/// Map radius
pub const MAP_RADIUS: u32 = 80;

pub struct WorldAttributes {
    pub elevation: f32,
    pub vulcanism: f32,
    pub mountain_spread: u32,
    pub elevation_increment: f32,
    pub sea_level: f32,
    pub base_temperature: f32,
}

impl Default for WorldAttributes {
    fn default() -> Self {
        Self {
            elevation: 10.0,
            vulcanism: 3.0,
            mountain_spread: MAP_RADIUS * 90 / 100, // Note: MAP_RADIUS must be defined somewhere in the scope
            elevation_increment: 0.5,
            sea_level: 3.0,
            base_temperature: 30.0,
        }
    }
}

impl WorldAttributes {
    /////////////////////////////// Altitude ///////////////////////////////////////////////////////
    /// 
    
    pub fn tile_from_weather(&self, altitude: f32, _temperature: f32) -> TileType {
        if altitude <= self.sea_level {
            TileType::Ocean
        } else if altitude >= self.elevation * 0.8 {
            TileType::Mountain
        } else if altitude >= self.elevation * 0.6 {
            TileType::Hills
        } else {
            TileType::Grass
        }
    }

    // The base probability is 1 when distance is 0 (at the volcano)
    // The probability decreases linearly as the distance increases, down to 0 at the furthest point
    pub fn increment_height(&self, current_height: &mut f32, distance: u32) {
        let distance_f32 = distance as f32;
        let probability = 1.0 - (distance_f32 / self.mountain_spread as f32);

        if rand::random::<f32>() < probability {
            *current_height += self.elevation_increment;
        }
    }

    pub fn generate_altitude_map(&self, all_hexes: &Vec<Hex>) -> HashMap<Hex, f32> {
        let mut rng = rand::thread_rng();

        let mut altitude_map: HashMap<Hex, f32> = all_hexes.iter().map(|hex| (*hex, 0.0)).collect();

        // Randomly pick hexes to start elevation gains from
        let volcano_hexes: Vec<Hex> = all_hexes
            .choose_multiple(&mut rng, self.vulcanism as usize)
            .cloned()
            .collect();

        // Set volcano hexes' initial altitude to ELEVATION_INCREMENT
        for hex in &volcano_hexes {
            altitude_map.insert(*hex, self.elevation_increment);
        }

        let mut max_height = 0.0;
        while max_height < self.elevation {
            for hex in &volcano_hexes {
                // Update the volcano height
                self.increment_height(altitude_map.get_mut(hex).unwrap(), 0);
                max_height = max_height.max(altitude_map[hex] as f32);

                // Update the neighbors' heights based on their distance to the volcano
                for rings_traversed in 1..=self.mountain_spread as u32 {
                    for neighbour in hex.ring(rings_traversed) {
                        if let Some(height) = altitude_map.get_mut(&neighbour) {
                            self.increment_height(height, rings_traversed);
                            max_height = max_height.max(*height as f32);
                        }
                    }
                }
            }
        }
        altitude_map
    }

    /////////////////////////////// Temperature ///////////////////////////////////////////////////////
    pub fn generate_temperature_map(&self, altitude_map: &HashMap<Hex, f32>) -> HashMap<Hex, f32> {
        // Constant factor that defines how much temperature varies from equator to the poles
        const LATITUDE_TEMPERATURE_VARIATION: f32 = 60.0;
        // Constant factor that defines how much the altitude affects the temperature
        const ALTITUDE_TEMPERATURE_VARIATION: f32 = 6.5;

        altitude_map
            .iter()
            .map(|(hex, &altitude)| {
                // Normalize y from -1 (south pole) to 1 (north pole)
                let normalized_y = (hex.y as f32).abs() / (MAP_RADIUS as f32);

                // Compute the temperature modifier based on latitude
                let latitude_temperature_mod = normalized_y * LATITUDE_TEMPERATURE_VARIATION;

                // Compute the temperature modifier based on altitude, linear and always negative
                let altitude_temperature_mod = if altitude > self.sea_level {
                    (altitude - self.sea_level) * ALTITUDE_TEMPERATURE_VARIATION
                } else {
                    0.0
                };

                // Base temperature - altitude modifier - latitude modifier
                let temperature =
                    self.base_temperature - altitude_temperature_mod - latitude_temperature_mod;

                (*hex, temperature)
            })
            .collect()
    }
}
