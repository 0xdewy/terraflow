use bevy::prelude::*;

use crate::world::ElevationAttributes;
use crate::{BedrockElevation, Humidity};

use crate::components::{SoilElevation, WaterElevation};

use strum::{IntoEnumIterator};
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

// TODO: add to config file
const HIGH_HUMIDITY: f32 = 0.8;
const LOW_HUMIDITY: f32 = 0.2;

const HIGH_WATER: f32 = 0.8;
const LOW_WATER: f32 = 0.2;

/*
 * apply weather effects to the terrain
 * returns an array of possible new terrain options
 */
pub trait WeatherEffects: Sized {
    fn apply_weather(&self, tile_type: &TileType) -> Vec<(TileType, f32)>;
    fn exceeds_limit(&self, tile_type: &TileType) -> bool;
    fn below_limit(&self, tile_type: &TileType) -> bool;
}

/*
 * Humidity weather effects on terrain
 *
 */
impl WeatherEffects for Humidity {
    fn apply_weather(&self, tile_type: &TileType) -> Vec<(TileType, f32)> {
        let mut probabilities = vec![];

        if self.exceeds_limit(tile_type) {
            probabilities.extend(match tile_type {
                TileType::Waste => vec![(TileType::Swamp, LOW_ODDS)],
                TileType::Grass => vec![(TileType::Forest, MED_ODDS)],
                TileType::Forest => vec![(TileType::Jungle, MED_ODDS)],
                TileType::Desert => vec![(TileType::Grass, LOW_ODDS), (TileType::Dirt, MED_ODDS)],
                TileType::Rocky => vec![(TileType::Hills, LOW_ODDS)],
                TileType::Dirt => vec![(TileType::Grass, MED_ODDS)],
                _ => vec![(*tile_type, CERTAIN)],
            });
        }
        if self.below_limit(tile_type) {
            probabilities.extend(match tile_type {
                TileType::Waste => vec![(TileType::Desert, LOW_ODDS)],
                TileType::Swamp | TileType::Jungle => {
                    vec![(TileType::Forest, MED_ODDS), (TileType::Desert, LOW_ODDS)]
                }
                TileType::Grass => vec![(TileType::Dirt, MED_ODDS), (TileType::Desert, MED_ODDS)],
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
                self.value > LOW_HUMIDITY
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

/*
 * Groundwater weather effects on terrain_change_sensitivity
 * (water_elevation, soil_elevation, terrain_change_sensitivity)
 */
impl WeatherEffects for (&WaterElevation, &SoilElevation, &f32) {
    fn apply_weather(&self, tile_type: &TileType) -> Vec<(TileType, f32)> {
        if self.exceeds_limit(tile_type) {
            match tile_type {
                TileType::Ocean | TileType::Water => return vec![(TileType::Ocean, CERTAIN)],
                TileType::Mountain => return vec![(TileType::Rocky, LOW_ODDS)],
                TileType::Rocky => return vec![(TileType::Dirt, MED_ODDS)],
                TileType::Dirt => return vec![(TileType::Grass, LOW_ODDS)],
                TileType::Grass => {
                    return vec![(TileType::Forest, MED_ODDS), (TileType::Water, LOW_ODDS)]
                }
                TileType::Forest => {
                    return vec![(TileType::Jungle, LOW_ODDS), (TileType::Water, MED_ODDS)]
                }
                TileType::Jungle => return vec![(TileType::Swamp, MED_ODDS)],
                TileType::Swamp => return vec![(TileType::Water, MED_ODDS)],
                TileType::Hills => return vec![(TileType::Water, LOW_ODDS)],
                TileType::Desert => return vec![(TileType::Grass, HIGH_ODDS)],
                _ => return vec![(*tile_type, CERTAIN)],
            }
        } else if self.below_limit(tile_type) {
            match tile_type {
                TileType::Swamp => {
                    return vec![(TileType::Dirt, LOW_ODDS), (TileType::Grass, MED_ODDS)]
                }
                TileType::Water => {
                    return vec![(TileType::Swamp, HIGH_ODDS), (TileType::Forest, MED_ODDS)]
                }
                TileType::Forest => return vec![(TileType::Grass, LOW_ODDS)],
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
            TileType::Dirt => self.0.value > self.1.value + self.2,
            TileType::Grass => self.0.value > self.1.value + self.2,
            TileType::Forest => self.0.value > self.1.value + self.2,
            TileType::Jungle => self.0.value > self.1.value + self.2,
            TileType::Swamp => self.0.value > self.1.value + self.2,
            _ => false,
        }
    }

    fn below_limit(&self, tile_type: &TileType) -> bool {
        match tile_type {
            TileType::Water => self.0.value < self.1.value,
            TileType::Ocean => false,
            TileType::Mountain => false,
            TileType::Ice => false,
            TileType::Swamp => self.0.value < self.1.value - self.2,
            _ => self.0.value < LOW_WATER,
        }
    }
}

// Elevation change
// (BedrockElevation, MaxElevation)
impl WeatherEffects for (&BedrockElevation, &ElevationAttributes) {
    fn apply_weather(&self, tile_type: &TileType) -> Vec<(TileType, f32)> {
        if self.exceeds_limit(tile_type) {
            match tile_type {
                TileType::Rocky => return vec![(TileType::Mountain, HIGH_ODDS)],
                TileType::Hills => return vec![(TileType::Mountain, HIGH_ODDS)],
                TileType::Ice => return vec![(TileType::Mountain, MED_ODDS)],
                TileType::Dirt => return vec![(TileType::Rocky, HIGH_ODDS)],
                TileType::Desert => return vec![(TileType::Rocky, HIGH_ODDS)],
                TileType::Grass => return vec![(TileType::Hills, HIGH_ODDS)],
                TileType::Ocean | TileType::Water => return vec![(*tile_type, CERTAIN)],
                _ => return vec![(TileType::Hills, MED_ODDS)],
            }
        }
        if self.below_limit(tile_type) {
            match tile_type {
                TileType::Mountain => return vec![(TileType::Rocky, HIGH_ODDS)],
                TileType::Hills => return vec![(TileType::Grass, HIGH_ODDS)],
                TileType::Rocky => return vec![(TileType::Dirt, HIGH_ODDS)],
                _ => return vec![(TileType::Ocean, MED_ODDS)],
            }
        }

        vec![(*tile_type, CERTAIN)]
    }

    fn exceeds_limit(&self, tile_type: &TileType) -> bool {
        match tile_type {
            TileType::Mountain => false,
            TileType::Rocky | TileType::Hills => {
                self.0.value / self.1.highest_elevation > self.1.mountain_point
            }
            TileType::Ice => self.0.value / self.1.highest_elevation > self.1.mountain_point,
            _ => self.0.value / self.1.highest_elevation > self.1.hill_point,
        }
    }

    fn below_limit(&self, tile_type: &TileType) -> bool {
        match tile_type {
            TileType::Mountain => self.0.value / self.1.highest_elevation < self.1.mountain_point,
            TileType::Rocky | TileType::Hills => {
                self.0.value / self.1.highest_elevation < self.1.hill_point
            }
            _ => false,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub enum TileAsset {
    Type(Handle<Scene>),
}

impl TileAsset {
    fn new(scene_handle: Handle<Scene>) -> Self {
        TileAsset::Type(scene_handle)
    }
}

impl Default for TileAsset {
    fn default() -> Self {
        TileAsset::Type(Handle::default())
    }
}

#[derive(Resource, Default, Clone)]
pub struct TileAssets {
    pub assets: Vec<TileAsset>,
}

impl TileAssets {
    pub fn new(asset_server: &Res<AssetServer>) -> Self {
        let assets = TileType::iter()
            .map(|tile_type| {
                let tile_str = format!("{:?}", tile_type);
                let scene_handle = asset_server.load(format!("tiles/{}.glb#Scene0", tile_str));
                TileAsset::new(scene_handle)
            })
            .collect();

        TileAssets { assets }
    }

    pub fn get_scene_handle(&self, tile_type: TileType) -> Option<&Handle<Scene>> {
        self.assets.get(tile_type as usize).and_then(|asset| {
            if let TileAsset::Type(handle) = asset {
                Some(handle)
            } else {
                None
            }
        })
    }
}

