use bevy::prelude::*;
use hexx::*;

use super::tiles::TileType;
use super::world::WorldAttributes;

use std::{cmp::max, collections::HashMap};

pub const EROSION_FACTOR: f32 = 0.5;
pub const EROSION_SCALE: f32 = 0.1;
pub const PRECIPITATION_FACTOR: f32 = 0.5;

#[derive(Resource, Clone)]
pub struct TerrainMap {
    pub map: HashMap<Hex, Terrain>,
    pub entity_map: HashMap<Entity, Hex>,
    pub world_attributes: WorldAttributes,
}

impl TerrainMap {
    pub fn epoch(&mut self) {
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

#[derive(Debug, Clone)]
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

    pub fn fertility(&self) -> f32 {
        return self.soil + self.humidity - self.pollution;
    }

    // TODO: make temperature a dynamic attribute
    pub fn evaporation(&self) -> f32 {
        // Each tile could have a default value?
        return self.ground_water * self.temperature;
    }

    // TODO: should there be a soil factor?
    pub fn overflow_level(&self) -> f32 {
        return self.soil;
    }

    // TODO: mountains will run out of soil, should volcanoes add soil?
    // TODO: save volcano points and produce soil from them + raise elevation
    pub fn erosion_rate(&self, overflow: f32) -> f32 {
        if overflow > 0.0 {
            return EROSION_FACTOR * (1.0 - self.soil) * overflow;
        } else {
            return 0.0;
        }
    }

    pub fn update_deposits(&mut self, overflow: f32, erosion: f32) {
        self.ground_water += overflow;
        self.soil += erosion;
    }

    pub fn precipitation(&self, neighbours: &Vec<Terrain>) -> f32 {
        // TODO: update humidity of higher neighbours
        // TODO: get the highest neighbour
        return (self.humidity - self.temperature) * PRECIPITATION_FACTOR;
    }

    pub fn epoch(&mut self, neighbours: &mut Vec<Terrain>) {
        self.humidity = self.humidity - self.precipitation(neighbours) + self.evaporation();
        let overflow = self.ground_water - self.overflow_level();

        let mut erosion = self.erosion_rate(overflow);
        let erosion_effect_on_soil = self.soil.min(erosion);
        self.soil = (self.soil - erosion_effect_on_soil).max(0.0);
        erosion -= erosion_effect_on_soil;

        self.altitude -= erosion * EROSION_SCALE;
        self.ground_water -= overflow;

        let mut lower_neighbours: Vec<&mut Terrain> = neighbours
            .iter_mut()
            .filter(|neighbour| neighbour.altitude < self.altitude)
            .collect();
        let lower_neighbours_count = lower_neighbours.len() as f32; // assuming f32 for arithmetic operations

        // TODO: create a modifier so lower elevation neighbours get more deposits
        let overflow_share = overflow / lower_neighbours_count;
        let erosion_share = erosion / lower_neighbours_count;

        lower_neighbours
            .iter_mut()
            .for_each(|neighbour| neighbour.update_deposits(overflow_share, erosion_share));
        // TODO: handle tile changes (e.g. desertification, flooding, etc.)
    }
}

///////////////////////////////////////// Terrain Defaults ////////////////////////////////////////////////
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
