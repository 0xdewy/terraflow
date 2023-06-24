use crate::components::LowerNeighbours;

use super::terrain::TileType;

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

impl RandomSelection<u32> for Vec<u32> {
    fn pick_random(&self) -> u32 {
        *self.choose(&mut rand::thread_rng()).unwrap()
    }
}

impl RandomSelection<TileType> for Vec<TileType> {
    fn pick_random(&self) -> TileType {
        *self.choose(&mut rand::thread_rng()).unwrap()
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
                return *tile_type;
            }
        }
        panic!("No tile type selected")
    }
}

pub fn get_lowest_neighbour(lower_neighbours: &LowerNeighbours) -> u32 {

    let mut lowest_neighbours = Vec::new();
    let mut lowest_height = f32::MAX;
    
    for (id, neighbour_height) in &lower_neighbours.ids {
        if *neighbour_height < lowest_height {
            lowest_height = *neighbour_height;
            lowest_neighbours.clear();
            lowest_neighbours.push(*id);
        } else if (*neighbour_height - lowest_height).abs() < f32::EPSILON {
            lowest_neighbours.push(*id);
        }
    }
    
    let receiver_index = if !lowest_neighbours.is_empty() {
        lowest_neighbours.choose(&mut rand::thread_rng()).unwrap().index()
    } else {
        panic!("No lower neighbours found!");
    };

    receiver_index
}

