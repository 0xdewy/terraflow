use bevy::prelude::*;
use hexx::*;

use rand::prelude::SliceRandom;
use std::collections::HashMap;

use super::tiles::TileType;
use super::utils::RandomSelection;

/// World size of the hexagons (outer radius)
pub const HEX_SIZE: Vec2 = Vec2::splat(2.0);
/// Map radius
pub const MAP_RADIUS: u32 = 50;

#[derive(Debug, Clone)]
pub struct AltitudeAttributes {
    pub highest_elevation: f32,
    pub vulcanism: f32,
    pub mountain_spread: u32,
    pub elevation_increment: f32,
    pub sea_level: f32,
}

impl AltitudeAttributes {
    pub fn new() -> Self {
        Self {
            highest_elevation: 10.0,
            vulcanism: 6.0,
            mountain_spread: MAP_RADIUS * 60 / 100,
            elevation_increment: 0.1,
            sea_level: 1.0,
        }
    }

    pub fn increment_height(&self, current_height: &mut f32, distance: u32) {
        let probability = 1.0 - (distance as f32 / self.mountain_spread as f32);
        if rand::random::<f32>() < probability {
            *current_height += self.elevation_increment;
        }
    }

    pub fn generate_altitude_map(&self, all_hexes: &Vec<Hex>) -> HashMap<Hex, f32> {
        let mut rng = rand::thread_rng();
        let mut altitude_map: HashMap<Hex, f32> = all_hexes.iter().map(|hex| (*hex, 0.0)).collect();
        let volcano_hexes: Vec<Hex> = all_hexes
            .choose_multiple(&mut rng, self.vulcanism as usize)
            .cloned()
            .collect();

        for hex in &volcano_hexes {
            altitude_map.insert(*hex, self.elevation_increment);
        }

        self.raise_volcanoes(&mut altitude_map, &volcano_hexes);
        altitude_map
    }

    fn raise_volcanoes(&self, altitude_map: &mut HashMap<Hex, f32>, volcano_hexes: &Vec<Hex>) {
        let mut max_height = 0.0;
        while max_height < self.highest_elevation {
            for hex in volcano_hexes {
                self.increment_height(altitude_map.get_mut(hex).unwrap(), 0);
                max_height = max_height.max(altitude_map[hex]);
                for rings_traversed in 1..=self.mountain_spread {
                    for neighbour in hex.ring(rings_traversed) {
                        if let Some(height) = altitude_map.get_mut(&neighbour) {
                            self.increment_height(height, rings_traversed);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemperatureAttributes {
    pub base_temperature: f32,
}

impl TemperatureAttributes {
    pub fn new() -> Self {
        Self {
            base_temperature: 50.0,
        }
    }

    pub fn generate_temperature_map(&self, altitude_map: &HashMap<Hex, f32>) -> HashMap<Hex, f32> {
        altitude_map
            .iter()
            .map(|(hex, &altitude)| (*hex, self.calculate_temperature(altitude, hex.y as f32)))
            .collect()
    }

    fn calculate_temperature(&self, altitude: f32, latitude: f32) -> f32 {
        const LATITUDE_TEMPERATURE_VARIATION: f32 = 60.0;
        const ALTITUDE_TEMPERATURE_VARIATION: f32 = 2.5;
        let normalized_y = latitude.abs() / MAP_RADIUS as f32;
        let latitude_temperature_mod = normalized_y * LATITUDE_TEMPERATURE_VARIATION;
        let altitude_temperature_mod = if altitude > 0.0 {
            altitude * ALTITUDE_TEMPERATURE_VARIATION
        } else {
            0.0
        };
        self.base_temperature - altitude_temperature_mod - latitude_temperature_mod
    }
}

#[derive(Debug, Clone)]
pub struct WorldAttributes {
    pub altitude: AltitudeAttributes,
    pub temperature: TemperatureAttributes,
}

impl WorldAttributes {
    pub fn new() -> Self {
        Self {
            altitude: AltitudeAttributes::new(),
            temperature: TemperatureAttributes::new(),
        }
    }
}

///////////////////////////////////////// TileGeneration ////////////////////////////////////////////////
///
pub trait TileTypeGenerator {
    fn spawn_tile(&self, latitude: f32, altitude: f32, temperature: f32) -> TileType;
}

impl TileTypeGenerator for WorldAttributes {
    // TODO: add more tile types and refactor this
    fn spawn_tile(&self, _latitude: f32, altitude: f32, temperature: f32) -> TileType {
        let cool_tiles: Vec<(TileType, f32)> = vec![
            (TileType::Grass, 0.4),
            (TileType::Forest, 0.3),
            (TileType::Water, 0.2),
            (TileType::Dirt, 0.2),
            (TileType::Hills, 0.1),
            (TileType::Rocky, 0.2),
            (TileType::Waste, 0.1),
            (TileType::Swamp, 0.1),
            (TileType::Jungle, 0.1),
        ];

        let hot_tiles: Vec<(TileType, f32)> = vec![
            (TileType::Jungle, 0.5),
            (TileType::Desert, 0.3),
            (TileType::Swamp, 0.3),
            (TileType::Dirt, 0.2),
            (TileType::Rocky, 0.3),
            (TileType::Waste, 0.1),
            (TileType::Forest, 0.1),
            (TileType::Grass, 0.1),
            (TileType::Water, 0.1),
        ];

        match temperature {
            t if t <= 0.0 => TileType::Ice,
            _ => match altitude {
                a if a <= self.altitude.sea_level => TileType::Ocean,
                a if a >= self.altitude.highest_elevation * 0.90 => TileType::Mountain,
                a if a >= self.altitude.highest_elevation * 0.70 => {
                    vec![TileType::Hills, TileType::Rocky].pick_random()
                }
                _ => {
                    if temperature <= self.temperature.base_temperature * 0.85 {
                        cool_tiles.pick_random()
                    } else {
                        hot_tiles.pick_random()
                    }
                }
            },
        }
    }
}

// TODO: handle visual changes to tiles
// Only effects the visuals of the tile
// trait VisualModifier {
//     fn altitude_tile_change(&self, tile_type: TileType, altitude: f32) -> TileType;
// }

// impl VisualModifier for WorldAttributes {
//     fn altitude_tile_change(&self, tile_type: TileType, altitude: f32) -> TileType {
//         // Change dirt -> rocky
//         // grass -> hills
//         match tile_type {
//             TileType::Dirt => {
//                 if altitude >= self.altitude.highest_elevation * 0.6 {
//                     TileType::Rocky
//                 } else {
//                     TileType::Dirt
//                 }
//             }
//             TileType::Grass => {
//                 if altitude >= self.altitude.highest_elevation * 0.6 {
//                     TileType::Hills
//                 } else {
//                     TileType::Grass
//                 }
//             }
//             _ => tile_type,
//         }
//     }
// }

///////////////////////////////////////// Randomness ////////////////////////////////////////////////
