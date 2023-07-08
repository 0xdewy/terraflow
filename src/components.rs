use bevy::prelude::*;
use hexx::Hex;
use std::fmt;

use crate::terrain::TileType;
use crate::Epochs;

///////////////////////////////// Intermediary Components /////////////////////////////////////////
///
/// This component is for display last epoch weather data
#[derive(Debug, Clone, Copy, Component)]
pub struct DebugWeatherBundle {
    pub overflow: Overflow,
    pub overflow_received: OverflowReceived,
    pub humidity_received: HumidityReceived,
    pub humidity_sent: HumiditySent,
    pub evaporation: Evaporation,
    pub precipitation: Precipitation,
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct HumidityReceived {
    pub value: f32,
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct HumiditySent {
    pub value: f32,
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct OverflowReceived {
    pub water: f32,
    pub soil: f32,
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Overflow {
    pub water: f32,
    pub soil: f32,
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Precipitation {
    pub value: f32,
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Evaporation {
    pub value: f32,
}

/////////////////// Dynamic Components /////////////////////////////////////////
// Dynamically assigned to entities to apply weather changes

#[derive(Debug, Clone, Copy, Component, Default)]
pub struct TileTypeChanged;

#[derive(Debug, Clone, Component, Default)]
pub struct OutgoingOverflow {
    pub water: f32,
    pub soil: f32,
}

#[derive(Debug, Clone, Component, Default)]
pub struct IncomingOverflow {
    pub water: f32,
    pub soil: f32,
}

#[derive(Debug, Clone, Component)]
pub struct PendingHumidityRedistribution {
    pub value: f32,
}

///////////////////////////////////////////////////////////////////////////////

////////////////////////// Components /////////////////////////////////////////

#[derive(Debug, Clone, Component)]
pub struct DistancesFromVolcano(pub Vec<u16>);

#[derive(Debug, Clone, Component)]
pub struct HexCoordinates(pub Hex);

#[derive(Debug, Clone, Copy, Component)]
pub struct ElevationBundle {
    pub bedrock: BedrockElevation,
    pub soil: SoilElevation,
    pub water: WaterElevation,
}

impl ElevationBundle {
    pub fn from(tile_type: TileType, bedrock: f32, water: f32) -> Self {
        let water_level: WaterElevation = match tile_type {
            TileType::Ocean => water.into(),
            _ => tile_type.into(),
        };
        ElevationBundle {
            bedrock: bedrock.into(),
            soil: tile_type.into(),
            water: water_level,
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct BedrockElevation {
    pub value: f32,
}

impl From<f32> for BedrockElevation {
    fn from(value: f32) -> Self {
        BedrockElevation { value }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct SoilElevation {
    pub value: f32,
}

impl From<TileType> for SoilElevation {
    fn from(tile_type: TileType) -> SoilElevation {
        match tile_type {
            TileType::Grass | TileType::Forest | TileType::Jungle | TileType::Swamp => {
                SoilElevation { value: 0.5 }
            }
            TileType::Hills => SoilElevation { value: 0.2 },
            TileType::Desert | TileType::Waste => SoilElevation { value: 0.3 },
            TileType::Rocky | TileType::Dirt => SoilElevation { value: 0.1 },
            TileType::Ocean | TileType::Water => SoilElevation { value: 0.0 },
            TileType::Mountain | TileType::Ice => SoilElevation { value: 0.0 },
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct WaterElevation {
    pub value: f32,
}

impl From<f32> for WaterElevation {
    fn from(value: f32) -> Self {
        WaterElevation { value }
    }
}

impl From<TileType> for WaterElevation {
    fn from(tile_type: TileType) -> Self {
        match tile_type {
            TileType::Ocean => 0.0.into(),
            TileType::Water | TileType::Swamp => 1.0.into(),
            TileType::Ice
            | TileType::Grass
            | TileType::Hills
            | TileType::Forest
            | TileType::Jungle => 0.8.into(),
            TileType::Dirt | TileType::Rocky => 0.5.into(),
            TileType::Mountain | TileType::Desert | TileType::Waste => 0.4.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Humidity {
    pub value: f32,
}

impl From<f32> for Humidity {
    fn from(value: f32) -> Self {
        Humidity { value }
    }
}

impl From<TileType> for Humidity {
    fn from(tile_type: TileType) -> Humidity {
        match tile_type {
            TileType::Ocean | TileType::Water | TileType::Swamp | TileType::Jungle => 1.0.into(),
            TileType::Ice | TileType::Grass | TileType::Hills | TileType::Forest => 0.7.into(),
            TileType::Dirt | TileType::Rocky | TileType::Mountain | TileType::Desert => 0.5.into(),
            TileType::Waste => 0.2.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Temperature {
    pub value: f32,
}

#[derive(Debug, Clone, Component)]
pub struct Neighbours {
    pub ids: Vec<Entity>,
}

#[derive(Debug, Clone, Component)]
pub struct HigherNeighbours {
    pub ids: Vec<(Entity, f32)>,
}

#[derive(Debug, Clone, Component)]
pub struct LowerNeighbours {
    pub ids: Vec<(Entity, f32)>,
}

impl fmt::Display for Epochs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Epochs: {} \n", self.epochs)?;
        write!(f, "Epochs left: {:?}", self.epochs_to_run)
    }
}

impl fmt::Display for HexCoordinates {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hex Coordinates: {:?}", self.0)
    }
}

impl fmt::Display for ElevationBundle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Bedrock Elevation: {}", self.bedrock.value)?;
        writeln!(f, "Soil Elevation: {}", self.soil.value)?;
        writeln!(f, "Water Elevation: {}", self.water.value)
    }
}

impl fmt::Display for BedrockElevation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for SoilElevation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for WaterElevation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for Humidity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for Overflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "water: {} soil: {}", self.water, self.soil)
    }
}

impl fmt::Display for OverflowReceived {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "water: {} soil: {}", self.water, self.soil)
    }
}

impl fmt::Display for Precipitation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for Evaporation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for Neighbours {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Neighbours: {:?}", self.ids)
    }
}

impl fmt::Display for HigherNeighbours {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Higher Neighbours: {:?}", self.ids)
    }
}

impl fmt::Display for LowerNeighbours {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lower Neighbours: {:?}", self.ids)
    }
}
