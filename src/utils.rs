use super::tiles::TileType;

use rand::{prelude::SliceRandom, Rng};

///////////////////////////////////////// Randomness ////////////////////////////////////////////////
///
pub trait RandomSelection<T> {
    fn pick_random(&self) -> T;
}

impl RandomSelection<bool> for f32 {
    fn pick_random(&self) -> bool {
        rand::thread_rng().gen::<f32>() < *self
    }
}

impl RandomSelection<TileType> for Vec<TileType> {
    fn pick_random(&self) -> TileType {
        self.choose(&mut rand::thread_rng()).unwrap().clone()
    }
}

impl RandomSelection<TileType> for Vec<(TileType, f32)> {
    fn pick_random(&self) -> TileType {
        let mut rng = rand::thread_rng();
        let total_weight: f32 = self.iter().map(|(_, weight)| weight).sum();
        let mut random_weight = rng.gen_range(0.0..total_weight);

        for (tile_type, weight) in self {
            random_weight -= weight;
            if random_weight <= 0.0 {
                return tile_type.clone();
            }
        }
        panic!("No tile type selected")
    }
}

// impl RandomSelection<Terrain> for Vec<Terrain> {
//     fn pick_random(&self) -> Terrain {
//         self.choose(&mut rand::thread_rng()).unwrap().clone()
//     }
// }

// impl RandomSelection<Terrain> for Vec<(Terrain, f32)> {
//     fn pick_random(&self) -> Terrain {
//         let mut rng = rand::thread_rng();
//         let total_weight: f32 = self.iter().map(|(_, weight)| weight).sum();
//         let mut random_weight = rng.gen_range(0.0..total_weight);

//         for (terrain, weight) in self {
//             random_weight -= weight;
//             if random_weight <= 0.0 {
//                 return terrain.clone();
//             }
//         }
//         panic!("No terrain selected")
//     }
// }
