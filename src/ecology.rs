pub enum TerrainType {
    Mountains,
    Ice,
    Rocky,
    Hills,
    Dirt,
    Grass,
    Desert,
    Plains,
    Jungle,
    Forest,
    Swamp,
    Wasteland,
    WaterDeep,
    WaterClear,
}

pub enum WeatherEvent {
    Rain,
    Snow,
    Sun,
    Storm,
    Wind,
    Fog,
    HeatWave,
    ColdSnap,
    Hail,
    Lightning,
    Blizzard,
    Drought,
}

////////////////////////// RESOURCES //////////////////////////
///

pub enum PrimaryResourceTier1 {
    Food,
    Energy,
    ConsumableFuel,
    RenewableFuel,
    Water,
    Oxygen,
    BioMaterial,
    FuelSource,
    Stone,
}

pub enum PrimaryResourceTier2 {
    BioMaterialOutbuilding,
    Metals,
    Coal,
}

pub enum InColonyProduction {
    RareMetals,
    ExoticElements,
    Oil,
    Artifacts,
    RefinedFuel,
    ProcessedFoods,
    Plastics,
}

pub enum CraftedResourceTier2 {
    NanoMaterials,
}

pub enum CraftedResourceTier3 {
    SuperconductiveElectronics,
    SyntheticNanofiber,
    EnergyCrystal,
    GeneticMaterial,
}

pub enum SecondaryResource {
    SecondaryResources,
    QuantumMaterial,
    TerraformingCatalyst,
    GravitronResonators,
    Alloys,
}

////////////////////// Terrain Traits /////////////////////////
pub trait Floodable {
    fn can_flood(&self) -> bool;
    fn flood(&mut self);
}

pub trait Erodeable {
    fn next_forms(&self) -> Vec<TerrainType>;
}

pub trait WeatherInfluencer {
    fn creates(&self) -> WeatherEvent;
}

pub trait ResourceCreator {
    fn creates(&self) -> Vec<PrimaryResourceTier1>;
}

pub trait WindInfluencer {
    fn blocks_wind(&self) -> bool;
}

pub trait TemperatureInfluencer {
    fn temperature_effect(&self) -> i8;
}

pub struct TerrainProperties {
    pub hardness: u8,
    pub volatility: u8,
    pub fertility: u8,
    pub richness: u8,
    pub elevation: u8,
    pub obstacles: u8,
    pub temperature: u8,
    pub wetness: u8,
    pub resources: Vec<PrimaryResourceTier1>,
}

////////////////////// Mountains /////////////////////////

struct Mountains {
    pub terrain: TerrainProperties,
}

impl Erodeable for Mountains {
    fn next_forms(&self) -> Vec<TerrainType> {
        let mut terrain_types = Vec::new();
        terrain_types.push(TerrainType::Rocky);
        terrain_types.push(TerrainType::Hills);
        terrain_types
    }
}

impl ResourceCreator for Mountains {
    fn creates(&self) -> Vec<PrimaryResourceTier1> {
        vec![PrimaryResourceTier1::Stone]
    }
}

////////////////////// Ice /////////////////////////
struct Ice {
    pub terrain: TerrainProperties,
}

impl TemperatureInfluencer for Ice {
    fn temperature_effect(&self) -> i8 {
        -10
    }
}

impl ResourceCreator for Ice {
    fn creates(&self) -> Vec<PrimaryResourceTier1> {
        vec![PrimaryResourceTier1::Water]
    }
}
