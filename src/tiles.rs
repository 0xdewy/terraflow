use super::{Humidity, SoilElevation, Temperature, WaterElevation, TERRAIN_CHANGE_SENSITIVITY};
use bevy::prelude::*;

const CERTAIN: f32 = 1.0;
const HIGH_ODDS: f32 = 0.8;
const MED_ODDS: f32 = 0.5;
const LOW_ODDS: f32 = 0.2;

#[derive(Clone, Debug, Copy, Component)]
pub enum TileType {
    Ocean,
    Water,
    Mountain,
    Grass,
    Hills, // Visual changes, but same as grass
    Desert,
    Dirt,
    Rocky, // Visual change, but same as dirt
    Forest,
    Ice,
    Jungle,
    Swamp,
    Waste,
}

impl TileType {
    pub fn default_ground_water(&self) -> f32 {
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

    pub fn default_humidity(&self) -> f32 {
        match self {
            TileType::Ocean | TileType::Water | TileType::Swamp | TileType::Jungle => 1.0,
            TileType::Ice | TileType::Grass | TileType::Hills | TileType::Forest => 0.7,
            TileType::Dirt | TileType::Rocky | TileType::Mountain | TileType::Desert => 0.5,
            TileType::Waste => 0.2,
        }
    }

    pub fn default_pollution(&self) -> f32 {
        match self {
            TileType::Ocean | TileType::Water | TileType::Swamp | TileType::Jungle => 0.0,
            TileType::Ice | TileType::Grass | TileType::Hills | TileType::Forest => 0.0,
            TileType::Dirt | TileType::Rocky | TileType::Mountain | TileType::Desert => 0.0,
            TileType::Waste => 1.0,
        }
    }

    pub fn default_soil(&self) -> f32 {
        match self {
            TileType::Grass
            | TileType::Hills
            | TileType::Forest
            | TileType::Jungle
            | TileType::Swamp => 1.0,
            TileType::Desert | TileType::Waste => 0.3,
            TileType::Rocky | TileType::Dirt => 0.1,
            TileType::Ocean | TileType::Water => 0.0,
            TileType::Mountain | TileType::Ice => 0.0,
        }
    }

    pub fn overflow_amount(&self, water_elevation: f32, soil_elevation: f32) -> f32 {
        match self {
            TileType::Ocean => 0.0,
            _ => {
                let water_above_soil = (water_elevation - soil_elevation).max(0.0);
                return water_above_soil;
            }
        }
    }

    // TODO: deserts evaporate faster?
    // pub fn evaporation_system(&self, temperature: f32, water_level: f32) -> f32 {
    //     match self {
    //         TileType::Ocean | TileType::Water => {
    //             let evaporation = temperature * water_level;
    //             return evaporation;
    //         }
    //     }
    // }

    // // 1.0 = everything escapes, 0.0 = nothing escapes
    pub fn precipitation_factor(&self) -> f32 {
        match self {
            TileType::Ocean | TileType::Water => 0.1,
            TileType::Dirt | TileType::Desert | TileType::Waste => 0.2,
            TileType::Swamp => 0.2,
            TileType::Ice | TileType::Grass | TileType::Forest => 0.5,
            TileType::Hills | TileType::Rocky | TileType::Jungle => 0.7,
            TileType::Mountain => 0.9,
        }
    }
}

const HIGH_HUMIDITY: f32 = 0.8;
const LOW_HUMIDITY: f32 = 0.2;

// Returns odds that a tile will change to a different type
pub trait WeatherEffects: Sized {
    fn apply_weather(&self, tile_type: &TileType) -> Vec<(TileType, f32)>;
    fn exceeds_limit(&self, tile_type: &TileType) -> bool;
    fn below_limit(&self, tile_type: &TileType) -> bool;
}

impl WeatherEffects for Humidity {
    fn apply_weather(&self, tile_type: &TileType) -> Vec<(TileType, f32)> {
        if self.exceeds_limit(tile_type) {
            match tile_type {
                TileType::Waste => return vec![(TileType::Swamp, LOW_ODDS)],
                TileType::Swamp | TileType::Jungle => return vec![(TileType::Jungle, HIGH_ODDS)],
                TileType::Grass | TileType::Forest => return vec![(TileType::Jungle, MED_ODDS)],
                TileType::Dirt | TileType::Desert => return vec![(TileType::Grass, LOW_ODDS)],
                TileType::Rocky => return vec![(TileType::Hills, LOW_ODDS)],
                _ => return vec![(*tile_type, CERTAIN)],
            }
        }
        vec![(*tile_type, CERTAIN)]
    }

    fn exceeds_limit(&self, tile_type: &TileType) -> bool {
        match tile_type {
            TileType::Ocean | TileType::Water => false,
            TileType::Waste => false,
            TileType::Swamp | TileType::Jungle => self.value > HIGH_HUMIDITY,
            TileType::Ice | TileType::Grass | TileType::Hills | TileType::Forest => {
                self.value > LOW_HUMIDITY
            }
            TileType::Dirt | TileType::Rocky | TileType::Mountain | TileType::Desert => {
                self.value < LOW_HUMIDITY
            }
        }
    }

    fn below_limit(&self, tile_type: &TileType) -> bool {
        match tile_type {
            TileType::Ocean | TileType::Water => false,
            TileType::Waste => false,
            TileType::Swamp | TileType::Jungle => self.value < LOW_HUMIDITY,
            TileType::Grass | TileType::Hills | TileType::Forest => self.value < LOW_HUMIDITY,
            _ => false,
        }
    }
}

impl WeatherEffects for (&WaterElevation, &SoilElevation) {
    fn apply_weather(&self, tile_type: &TileType) -> Vec<(TileType, f32)> {
        if self.exceeds_limit(tile_type) {
            match tile_type {
                TileType::Ocean | TileType::Water => return vec![(TileType::Ocean, CERTAIN)],
                TileType::Mountain => return vec![(TileType::Rocky, LOW_ODDS)],
                TileType::Rocky => return vec![(TileType::Dirt, LOW_ODDS)],
                TileType::Dirt => return vec![(TileType::Grass, LOW_ODDS)],
                TileType::Grass => return vec![(TileType::Forest, LOW_ODDS)],
                TileType::Forest => return vec![(TileType::Jungle, LOW_ODDS)],
                TileType::Jungle => return vec![(TileType::Swamp, LOW_ODDS)],
                TileType::Swamp => return vec![(TileType::Water, LOW_ODDS)],
                _ => return vec![(*tile_type, CERTAIN)],
            }
        }
        vec![(*tile_type, CERTAIN)]
    }

    fn exceeds_limit(&self, tile_type: &TileType) -> bool {
        match tile_type {
            TileType::Ocean | TileType::Water => false,
            TileType::Mountain => false,
            TileType::Ice => false,
            TileType::Dirt => self.0.value > self.1.value + TERRAIN_CHANGE_SENSITIVITY,
            TileType::Grass => self.0.value > self.1.value + TERRAIN_CHANGE_SENSITIVITY,
            TileType::Forest => self.0.value > self.1.value + TERRAIN_CHANGE_SENSITIVITY,
            TileType::Jungle => self.0.value > self.1.value + TERRAIN_CHANGE_SENSITIVITY,
            TileType::Swamp => self.0.value > self.1.value + TERRAIN_CHANGE_SENSITIVITY,
            _ => false,
        }
    }

    fn below_limit(&self, tile_type: &TileType) -> bool {
        match tile_type {
            TileType::Water => self.0.value < self.1.value,
            TileType::Ocean => false,
            TileType::Mountain => false,
            TileType::Ice => false,
            TileType::Swamp => self.0.value < self.1.value - TERRAIN_CHANGE_SENSITIVITY,
            _ => false,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct TileAssets {
    pub desert: (Handle<Mesh>, Handle<StandardMaterial>),
    pub dirt: (Handle<Mesh>, Handle<StandardMaterial>),
    pub forest: (Handle<Mesh>, Handle<StandardMaterial>),
    pub grass: (Handle<Mesh>, Handle<StandardMaterial>),
    pub hills: (Handle<Mesh>, Handle<StandardMaterial>),
    pub ice: (Handle<Mesh>, Handle<StandardMaterial>),
    pub jungle: (Handle<Mesh>, Handle<StandardMaterial>),
    pub mountain: (Handle<Mesh>, Handle<StandardMaterial>),
    pub ocean: (Handle<Mesh>, Handle<StandardMaterial>),
    pub rocky: (Handle<Mesh>, Handle<StandardMaterial>),
    pub swamp: (Handle<Mesh>, Handle<StandardMaterial>),
    pub waste: (Handle<Mesh>, Handle<StandardMaterial>),
    pub water: (Handle<Mesh>, Handle<StandardMaterial>),
}

impl TileAssets {
    pub fn new(asset_server: &Res<AssetServer>) -> Self {
        TileAssets {
            desert: (
                asset_server.load("tiles/Desert.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Desert.gltf#Material0"),
            ),
            dirt: (
                asset_server.load("tiles/Dirt.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Dirt.gltf#Material0"),
            ),
            forest: (
                asset_server.load("tiles/Forest.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Forest.gltf#Material0"),
            ),
            grass: (
                asset_server.load("tiles/Grass.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Grass.gltf#Material0"),
            ),
            hills: (
                asset_server.load("tiles/Hills.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Hills.gltf#Material0"),
            ),
            ice: (
                asset_server.load("tiles/Ice.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Ice.gltf#Material0"),
            ),
            jungle: (
                asset_server.load("tiles/Jungle.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Jungle.gltf#Material0"),
            ),
            mountain: (
                asset_server.load("tiles/Mountain.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Mountain.gltf#Material0"),
            ),
            ocean: (
                asset_server.load("tiles/Ocean.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Ocean.gltf#Material0"),
            ),
            rocky: (
                asset_server.load("tiles/Rocky.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Rocky.gltf#Material0"),
            ),
            swamp: (
                asset_server.load("tiles/Swamp.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Swamp.gltf#Material0"),
            ),
            waste: (
                asset_server.load("tiles/Waste.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Waste.gltf#Material0"),
            ),
            water: (
                asset_server.load("tiles/Water.gltf#Mesh0/Primitive0"),
                asset_server.load("tiles/Water.gltf#Material0"),
            ),
        }
    }

    pub fn mesh_and_material(
        &self,
        tile_type: &TileType,
    ) -> (Handle<Mesh>, Handle<StandardMaterial>) {
        match tile_type {
            TileType::Ocean => self.ocean.clone(),
            TileType::Water => self.water.clone(),
            TileType::Mountain => self.mountain.clone(),
            TileType::Hills => self.hills.clone(),
            TileType::Grass => self.grass.clone(),
            TileType::Desert => self.desert.clone(),
            TileType::Dirt => self.dirt.clone(),
            TileType::Forest => self.forest.clone(),
            TileType::Ice => self.ice.clone(),
            TileType::Jungle => self.jungle.clone(),
            TileType::Rocky => self.rocky.clone(),
            TileType::Swamp => self.swamp.clone(),
            TileType::Waste => self.waste.clone(),
        }
    }
}
