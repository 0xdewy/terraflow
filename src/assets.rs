use bevy::prelude::*;

pub enum TileType {
    Ocean,
    Water,
    Mountain,
    Hills,
    Grass,
    Desert,
    Dirt,
    Forest,
    Ice,
    Jungle,
    Rocky,
    Swamp,
    Waste,
}

pub struct Ocean;
pub struct Water;
pub struct Mountain;
pub struct Hills;
pub struct Grass;
pub struct Desert;
pub struct Dirt;
pub struct Forest;
pub struct Ice;
pub struct Jungle;
pub struct Rocky;
pub struct Swamp;
pub struct Waste;

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

    pub fn mesh_and_material(&self, tile_type: &TileType) -> (Handle<Mesh>, Handle<StandardMaterial>) {
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

// #[derive(Debug, Clone)]
// pub struct SceneTileAssets {
//     pub desert: Handle<Scene>,
//     pub dirt: Handle<Scene>,
//     pub forest: Handle<Scene>,
//     pub grass: Handle<Scene>,
//     pub hills: Handle<Scene>,
//     pub ice: Handle<Scene>,
//     pub jungle: Handle<Scene>,
//     pub mountain: Handle<Scene>,
//     pub ocean: Handle<Scene>,
//     pub rocky: Handle<Scene>,
//     pub swamp: Handle<Scene>,
//     pub waste: Handle<Scene>,
//     pub water: Handle<Scene>,
// }

// // TODO: Scenes are breaking the picking plugin
// pub fn _load_tile_assets_scene(asset_server: &Res<AssetServer>) -> SceneTileAssets {
//     SceneTileAssets {
//         desert: asset_server.load("tiles/Desert.gltf#Scene0"),
//         dirt: asset_server.load("tiles/Dirt.gltf#Scene0"),
//         forest: asset_server.load("tiles/Forest.gltf#Scene0"),
//         grass: asset_server.load("tiles/Grass.gltf#Scene0"),
//         hills: asset_server.load("tiles/Hills.gltf#Scene0"),
//         ice: asset_server.load("tiles/Ice.gltf#Scene0"),
//         jungle: asset_server.load("tiles/Jungle.gltf#Scene0"),
//         mountain: asset_server.load("tiles/Mountain.gltf#Scene0"),
//         ocean: asset_server.load("tiles/Ocean.gltf#Scene0"),
//         rocky: asset_server.load("tiles/Rocky.gltf#Scene0"),
//         swamp: asset_server.load("tiles/Swamp.gltf#Scene0"),
//         waste: asset_server.load("tiles/Waste.gltf#Scene0"),
//         water: asset_server.load("tiles/Water.gltf#Scene0"),
//     }
// }
