use bevy::prelude::*;

use crate::tiles::TileType;
use crate::utils::RandomSelection;

#[derive(Debug, serde::Deserialize)]
pub struct WorldAttributes {
    pub erosion: ErosionAttributes,
    pub elevation: ElevationAttributes,
    pub temperature: TemperatureAttributes,
    pub map: MapAttributes,
    pub ecosystem: EcosystemAttributes,
}

impl WorldAttributes {
    pub fn load() -> Self {
        let config_str = include_str!("../defaults.json");
        let config: Config = serde_json::from_str(config_str).unwrap();
        Self {
            erosion: ErosionAttributes::from(&config),
            elevation: ElevationAttributes::from(&config),
            temperature: TemperatureAttributes::from(&config),
            map: MapAttributes::from(&config),
            ecosystem: EcosystemAttributes::from(&config),
        }
    }
}

#[derive(Debug, serde::Deserialize, Resource)]
pub struct ErosionAttributes {
    pub erosion_factor: f32,
    pub erosion_scale: f32,
}

impl From<&Config> for ErosionAttributes {
    fn from(config: &Config) -> Self {
        Self {
            erosion_factor: config.erosion_factor,
            erosion_scale: config.erosion_scale,
        }
    }
}

#[derive(Debug, serde::Deserialize, Resource)]
pub struct ElevationAttributes {
    pub highest_elevation: f32,
    pub vulcanism: f32,
    pub mountain_spread: f32,
    pub elevation_increment: f32,
    pub mountain_point: f32,
    pub hill_point: f32,
    pub sea_level: f32,
    pub soil_and_water_height_display_factor: f32,
}

impl From<&Config> for ElevationAttributes {
    fn from(config: &Config) -> Self {
        Self {
            highest_elevation: config.highest_elevation,
            vulcanism: config.vulcanism,
            mountain_spread: config.mountain_spread * config.map_radius as f32,
            elevation_increment: config.elevation_increment,
            mountain_point: config.mountain_point,
            hill_point: config.hill_point,
            sea_level: config.sea_level,
            soil_and_water_height_display_factor: config.soil_and_water_height_display_factor,
        }
    }
}

#[derive(Debug, serde::Deserialize, Resource)]
pub struct TemperatureAttributes {
    pub base_temperature: f32,
    pub latitude_temperature_variation: f32,
    pub altitude_temperature_variation: f32,
}

impl From<&Config> for TemperatureAttributes {
    fn from(config: &Config) -> Self {
        Self {
            base_temperature: config.base_temperature,
            latitude_temperature_variation: config.latitude_temperature_variation,
            altitude_temperature_variation: config.altitude_temperature_variation,
        }
    }
}

#[derive(Debug, serde::Deserialize, Resource)]
pub struct MapAttributes {
    pub hex_size: f32,
    pub map_radius: u16,
}

impl From<&Config> for MapAttributes {
    fn from(config: &Config) -> Self {
        Self {
            hex_size: config.hex_size,
            map_radius: config.map_radius,
        }
    }
}

#[derive(Debug, serde::Deserialize, Resource)]
pub struct EcosystemAttributes {
    pub precipitation_factor: f32,
    pub evaporation_factor: f32,
    pub terrain_change_sensitivity: f32,
}

impl From<&Config> for EcosystemAttributes {
    fn from(config: &Config) -> Self {
        Self {
            precipitation_factor: config.precipitation_factor,
            evaporation_factor: config.evaporation_factor,
            terrain_change_sensitivity: config.terrain_change_sensitivity,
        }
    }
}

#[derive(Debug, serde::Deserialize, Resource)]
pub struct Config {
    hex_size: f32,
    map_radius: u16,
    erosion_factor: f32,
    erosion_scale: f32,
    precipitation_factor: f32,
    evaporation_factor: f32,
    highest_elevation: f32,
    vulcanism: f32,
    mountain_spread: f32,
    elevation_increment: f32,
    sea_level: f32,
    terrain_change_sensitivity: f32,
    mountain_point: f32,
    hill_point: f32,
    soil_and_water_height_display_factor: f32,
    base_temperature: f32,
    latitude_temperature_variation: f32,
    altitude_temperature_variation: f32,
}

///////////////////////////////////////// TileGeneration ////////////////////////////////////////////////

// TODO: this doesn't need to be a trait
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
                a if a <= self.elevation.sea_level => TileType::Ocean,
                a if a >= self.elevation.highest_elevation * self.elevation.mountain_point => {
                    TileType::Mountain
                }
                a if a >= self.elevation.highest_elevation * self.elevation.hill_point => {
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

///////////////////////////////////////// Randomness ////////////////////////////////////////////////
