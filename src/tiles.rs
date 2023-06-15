use bevy::prelude::*;

use super::{Humidity, SoilElevation, WaterElevation, TERRAIN_CHANGE_SENSITIVITY};

use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount, EnumIter};

const CERTAIN: f32 = 1.0;
const HIGH_ODDS: f32 = 0.8;
const MED_ODDS: f32 = 0.5;
const LOW_ODDS: f32 = 0.2;

#[derive(Clone, EnumCount, EnumIter, Debug, Copy, PartialEq, Hash, Component)]
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
    // how much precipitation adds to groundwater
    pub fn handle_precipitation(&self, precipitation: f32) -> f32 {
        match self {
            TileType::Ocean => 0.0,
            _ => precipitation,
        }
    }

    // how much evaporation takes away from ground_water
    pub fn handle_evaporation(&self, evaporation: f32) -> f32 {
        match self {
            TileType::Ocean => 0.0,
            _ => evaporation,
        }
    }

    pub fn overflow_amount(&self, water_elevation: f32, soil_elevation: f32) -> f32 {
        match self {
            TileType::Ocean => 0.0,
            _ => (water_elevation - soil_elevation).max(0.0),
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
        let mut probabilities = vec![];

        if self.exceeds_limit(tile_type) {
            probabilities.extend(match tile_type {
                TileType::Waste => vec![(TileType::Swamp, LOW_ODDS)],
                TileType::Grass => vec![(TileType::Forest, MED_ODDS)],
                TileType::Forest => vec![(TileType::Jungle, MED_ODDS)],
                TileType::Desert => vec![(TileType::Grass, MED_ODDS)],
                TileType::Rocky => vec![(TileType::Hills, LOW_ODDS)],
                _ => vec![(*tile_type, CERTAIN)],
            });
        }
        if self.below_limit(tile_type) {
            probabilities.extend(match tile_type {
                TileType::Waste => vec![(TileType::Desert, LOW_ODDS)],
                TileType::Swamp | TileType::Jungle => vec![(TileType::Forest, LOW_ODDS)],
                TileType::Grass => vec![(TileType::Dirt, MED_ODDS)],
                TileType::Forest => vec![(TileType::Grass, MED_ODDS)],
                _ => vec![(*tile_type, CERTAIN)],
            })
        }

        if probabilities.is_empty() {
            return vec![(*tile_type, CERTAIN)];
        }

        probabilities
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
        } else if self.below_limit(tile_type) {
            match tile_type {
                TileType::Swamp => {
                    return vec![(TileType::Dirt, LOW_ODDS), (TileType::Grass, MED_ODDS)]
                }
                TileType::Water => return vec![(TileType::Swamp, MED_ODDS)],
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

#[derive(Resource, Clone)]
pub enum TileAsset {
    Desert((Handle<Mesh>, Handle<StandardMaterial>)),
    Dirt((Handle<Mesh>, Handle<StandardMaterial>)),
    Forest((Handle<Mesh>, Handle<StandardMaterial>)),
    Grass((Handle<Mesh>, Handle<StandardMaterial>)),
    Hills((Handle<Mesh>, Handle<StandardMaterial>)),
    Ice((Handle<Mesh>, Handle<StandardMaterial>)),
    Jungle((Handle<Mesh>, Handle<StandardMaterial>)),
    Mountain((Handle<Mesh>, Handle<StandardMaterial>)),
    Ocean((Handle<Mesh>, Handle<StandardMaterial>)),
    Rocky((Handle<Mesh>, Handle<StandardMaterial>)),
    Swamp((Handle<Mesh>, Handle<StandardMaterial>)),
    Waste((Handle<Mesh>, Handle<StandardMaterial>)),
    Water((Handle<Mesh>, Handle<StandardMaterial>)),
}

impl Default for TileAsset {
    fn default() -> Self {
        TileAsset::Ocean((Handle::default(), Handle::default()))
    }
}

#[derive(Resource, Default, Clone)]
pub struct TileAssets {
    assets: [TileAsset; TileType::COUNT],
}

impl TileAssets {
    pub fn new(asset_server: &Res<AssetServer>) -> Self {
        let mut assets: [TileAsset; TileType::COUNT] = Default::default();

        for tile_type in TileType::iter() {
            assets[usize::from(tile_type)] = TileAssets::load_tile_asset(asset_server, tile_type);
        }

        TileAssets { assets }
    }

    fn load_tile_asset(asset_server: &Res<AssetServer>, tile_type: TileType) -> TileAsset {
        let tile_str = format!("{:?}", tile_type);
        let mesh = asset_server.load(format!("tiles/{}.gltf#Mesh0/Primitive0", tile_str));
        let material = asset_server.load(format!("tiles/{}.gltf#Material0", tile_str));

        match tile_type {
            TileType::Desert => TileAsset::Desert((mesh, material)),
            TileType::Dirt => TileAsset::Dirt((mesh, material)),
            TileType::Forest => TileAsset::Forest((mesh, material)),
            TileType::Grass => TileAsset::Grass((mesh, material)),
            TileType::Hills => TileAsset::Hills((mesh, material)),
            TileType::Ice => TileAsset::Ice((mesh, material)),
            TileType::Jungle => TileAsset::Jungle((mesh, material)),
            TileType::Mountain => TileAsset::Mountain((mesh, material)),
            TileType::Ocean => TileAsset::Ocean((mesh, material)),
            TileType::Rocky => TileAsset::Rocky((mesh, material)),
            TileType::Swamp => TileAsset::Swamp((mesh, material)),
            TileType::Waste => TileAsset::Waste((mesh, material)),
            TileType::Water => TileAsset::Water((mesh, material)),
        }
    }

    pub fn get_mesh_and_material(
        &self,
        tile_type: &TileType,
    ) -> (Handle<Mesh>, Handle<StandardMaterial>) {
        self.assets[usize::from(*tile_type)].clone().into()
    }
}

impl From<TileAsset> for (Handle<Mesh>, Handle<StandardMaterial>) {
    fn from(tile_asset: TileAsset) -> Self {
        match tile_asset {
            TileAsset::Desert(data)
            | TileAsset::Dirt(data)
            | TileAsset::Forest(data)
            | TileAsset::Grass(data)
            | TileAsset::Hills(data)
            | TileAsset::Ice(data)
            | TileAsset::Jungle(data)
            | TileAsset::Mountain(data)
            | TileAsset::Ocean(data)
            | TileAsset::Rocky(data)
            | TileAsset::Swamp(data)
            | TileAsset::Waste(data)
            | TileAsset::Water(data) => data,
        }
    }
}

impl From<TileType> for usize {
    fn from(tile_type: TileType) -> Self {
        tile_type as Self
    }
}
