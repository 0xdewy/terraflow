use bevy::prelude::*;
use hexx::*;

use crate::utils::RandomSelection;

use super::tiles::TileType;
use super::world::WorldAttributes;

use std::{cmp::max, collections::HashMap};

pub const EROSION_FACTOR: f32 = 0.05;
pub const EROSION_SCALE: f32 = 0.1;
pub const PRECIPITATION_FACTOR: f32 = 0.03;
pub const HUMIDITY_FACTOR: f32 = 0.03;
pub const HUMIDITY_TRAVEL_FACTOR: f32 = 0.1;
pub const EVAPORATION_FACTOR: f32 = 0.03;

#[derive(Resource, Clone)]
pub struct TerrainMap {
    pub map: HashMap<Hex, Terrain>,
    pub entity_map: HashMap<Entity, Hex>,
    pub world_attributes: WorldAttributes,
}

impl TerrainMap {
    pub fn epoch(&mut self) {
        // TODO: this is probably wasteful, can we mutate it as a reference?
        let mut new_map = self.map.clone(); // Cloning the map to operate on

        for (hex, terrain) in new_map.iter_mut() {
            let hex_neighbours = hex.ring(1);
            let mut neighbours: Vec<Terrain> = hex_neighbours
                .into_iter()
                .filter_map(|hex_neighbour| self.map.get(&hex_neighbour).cloned())
                .collect();

            terrain.epoch(&mut neighbours);
        }

        self.map = new_map; // Assigning back the updated map
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Terrain {
    pub entity: Entity,
    pub coordinates: Hex,
    pub tile_type: TileType,
    pub altitude: f32,
    pub temperature: f32,
    pub pollution: f32,
    pub ground_water: f32,
    pub humidity: f32,
    pub soil: f32,
}

impl Terrain {
    pub fn new(
        entity: Entity,
        coordinates: Hex,
        tile_type: TileType,
        altitude: f32,
        temperature: f32,
    ) -> Self {
        Self {
            entity,
            coordinates,
            tile_type: tile_type.clone(),
            altitude,
            temperature,
            pollution: tile_type.default_pollution(),
            ground_water: tile_type.default_ground_water(),
            humidity: tile_type.default_humidity(),
            soil: tile_type.default_soil(),
        }
    }

    pub fn epoch(&mut self, neighbours: &mut Vec<Terrain>) {
        let precipitation = self.precipitation();
        self.humidity = self.humidity - precipitation + self.evaporation();
        self.ground_water += precipitation;

        match self.tile_type {
            TileType::Ocean | TileType::Water => {}
            _ => {
                self.ground_water -= self.evaporation().max(0.0);
            }
        }

        // overflow causes erosion
        let overflow = (self.ground_water - self.overflow_level()).max(0.0);
        let erosion = self.apply_erosion(overflow);

        // lose soil, altitude and water
        self.soil = (self.soil - erosion).max(0.0);
        self.altitude -= erosion * EROSION_SCALE;
        self.ground_water -= overflow;

        // distribute ground water
        let mut ground_water_receiver = self.ground_water_receiver(neighbours);
        ground_water_receiver.update_deposits(overflow, erosion);

        // distribute humidity
        let humidity_escape_paths = neighbours.get_higher(self.altitude);
        let odds = humidity_escape_paths.len() as f32 / neighbours.len() as f32;
        if odds.pick_random() {
            let amount_humidity = self.humidity * HUMIDITY_TRAVEL_FACTOR;
            let mut humidity_receiver = self.humidity_receiver(neighbours);
            humidity_receiver.humidity += amount_humidity;
            self.humidity -= amount_humidity;
        }

        // TODO: handle tile changes (e.g. desertification, flooding, etc.)
    }

    pub fn fertility(&self) -> f32 {
        return self.soil + self.humidity - self.pollution;
    }

    // TODO: make temperature a dynamic attribute
    pub fn evaporation(&self) -> f32 {
        // Each tile could have a default value?
        return self.ground_water * self.temperature * EVAPORATION_FACTOR;
    }

    // TODO: tiletype should probably influence the overflow level?
    pub fn overflow_level(&self) -> f32 {
        return self.soil;
    }

    // TODO: mountains will run out of soil, should volcanoes add soil?
    // TODO: save volcano points and produce soil from them + raise elevation
    pub fn erosion_rate(&self, overflow: f32) -> f32 {
        // will return 0 if there is no overflow
        return EROSION_FACTOR * (1.0 - self.soil) * overflow;
    }

    pub fn apply_erosion(&self, overflow: f32) -> f32 {
        let mut erosion = self.erosion_rate(overflow);
        let erosion_effect_on_soil = self.soil.min(erosion);
        erosion -= erosion_effect_on_soil;
        erosion
    }

    pub fn update_deposits(&mut self, overflow: f32, erosion: f32) {
        self.ground_water += overflow;
        self.soil += erosion;
    }

    pub fn precipitation(&self) -> f32 {
        if self.tile_type.precipitation_factor().pick_random() {
            return self.humidity * PRECIPITATION_FACTOR;
        }

        0.0
    }

    // Returns the proportion (percentage) of the overflow/erosion that each neighbour will receive
    fn ground_water_receiver(&self, neighbours: &Vec<Terrain>) -> Terrain {
        let lower_neighbours = neighbours.get_lower(self.altitude);

        if lower_neighbours.is_empty() {
            return neighbours.pick_random();
        }

        let odds_array = lower_neighbours
            .iter()
            .map(|neighbour| (neighbour.clone(), self.altitude - neighbour.altitude))
            .collect::<Vec<(Terrain, f32)>>();

        odds_array.pick_random()
    }

    fn humidity_receiver(&self, neighbours: &Vec<Terrain>) -> Terrain {
        let higher_neighbours = neighbours.get_higher(self.altitude);

        if higher_neighbours.is_empty() {
            return neighbours.pick_random();
        }

        // Higher altitude difference should increase odds that humidity goes there
        let odds_array = higher_neighbours
            .iter()
            .map(|neighbour| (neighbour.clone(), neighbour.altitude - self.altitude))
            .collect::<Vec<(Terrain, f32)>>();

        odds_array.pick_random()
    }
}

pub trait Altitude {
    fn get_higher(&self, altitude: f32) -> Self;
    fn get_lower(&self, altitude: f32) -> Self;
}

impl Altitude for Vec<Terrain> {
    fn get_higher(&self, altitude: f32) -> Self {
        self.iter()
            .filter(|terrain| terrain.altitude > altitude)
            .cloned()
            .collect()
    }
    fn get_lower(&self, altitude: f32) -> Self {
        self.iter()
            .filter(|terrain| terrain.altitude < altitude)
            .cloned()
            .collect()
    }
}

///////////////////////////////////////// Terrain Defaults ////////////////////////////////////////////////
// TODO: implement in TileType directly
pub trait TerrainDefaults {
    fn default_ground_water(&self) -> f32;
    fn default_humidity(&self) -> f32;
    fn default_soil(&self) -> f32;
    fn default_pollution(&self) -> f32;
}

impl TerrainDefaults for TileType {
    fn default_ground_water(&self) -> f32 {
        match self {
            TileType::Ocean | TileType::Water | TileType::Swamp => 1.0,
            TileType::Ice
            | TileType::Grass
            | TileType::Hills
            | TileType::Forest
            | TileType::Jungle => 0.7,
            TileType::Dirt | TileType::Rocky => 0.5,
            TileType::Mountain | TileType::Desert | TileType::Waste => 0.2,
        }
    }

    fn default_humidity(&self) -> f32 {
        match self {
            TileType::Ocean | TileType::Water | TileType::Swamp | TileType::Jungle => 1.0,
            TileType::Ice | TileType::Grass | TileType::Hills | TileType::Forest => 0.7,
            TileType::Dirt | TileType::Rocky | TileType::Mountain | TileType::Desert => 0.5,
            TileType::Waste => 0.2,
        }
    }

    fn default_pollution(&self) -> f32 {
        match self {
            TileType::Ocean | TileType::Water | TileType::Swamp | TileType::Jungle => 0.0,
            TileType::Ice | TileType::Grass | TileType::Hills | TileType::Forest => 0.0,
            TileType::Dirt | TileType::Rocky | TileType::Mountain | TileType::Desert => 0.0,
            TileType::Waste => 1.0,
        }
    }

    fn default_soil(&self) -> f32 {
        match self {
            TileType::Grass
            | TileType::Hills
            | TileType::Forest
            | TileType::Jungle
            | TileType::Swamp => 1.0,
            TileType::Ocean | TileType::Water => 0.2,
            TileType::Mountain | TileType::Ice => 0.0,
            TileType::Desert | TileType::Waste => 0.3,
            TileType::Rocky | TileType::Dirt => 0.1,
        }
    }
}