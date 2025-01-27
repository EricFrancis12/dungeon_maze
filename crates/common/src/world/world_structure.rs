use crate::world::Chunk;
use bevy::prelude::{Asset, TypePath};
use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};
use strum::{IntoEnumIterator, VariantArray};
use strum_macros::{Display, EnumIter, VariantArray};

#[derive(Asset, Clone, Deserialize, Serialize, TypePath)]
pub struct WorldStructure {
    pub chunks: Vec<Chunk>,
}

#[derive(
    Clone,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    VariantArray,
)]
pub enum WorldStructureName {
    #[default]
    None,
    EmptySpace1,
    FilledWithChairs1,
    House1,
    StairsAltar1,
    StaircaseTower2,
}

impl WorldStructureName {
    pub fn radius(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::EmptySpace1 | Self::FilledWithChairs1 | Self::House1 | Self::StairsAltar1 => 1,
            Self::StaircaseTower2 => 2,
        }
    }

    pub fn max_radius() -> u32 {
        Self::iter().map(|ws| ws.radius()).max().unwrap_or(0)
    }

    pub fn weight(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::EmptySpace1 => 3.0,
            Self::FilledWithChairs1 => 1.0,
            Self::House1 => 3.0,
            Self::StairsAltar1 => 4.0,
            Self::StaircaseTower2 => 4.0,
        }
    }

    pub fn total_weight() -> f32 {
        Self::iter().fold(0.0, |acc, curr| acc + curr.weight())
    }

    pub fn choose(rng: &mut StdRng) -> Self {
        let variants = Self::VARIANTS;

        let weights: Vec<f32> = variants.iter().map(|ws| ws.weight()).collect();
        let rand_weight = rng.gen_range(0.0..Self::total_weight());

        let mut cumulative_weight = 0.0;
        for (index, &weight) in weights.iter().enumerate() {
            cumulative_weight += weight;
            if rand_weight < cumulative_weight {
                return variants[index].clone();
            }
        }

        Self::default()
    }
}
