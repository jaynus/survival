#![deny(clippy::pedantic, clippy::all)]
#![feature(custom_attribute)]
#![allow(
    clippy::module_name_repetitions,
    clippy::type_complexity,
    unused_attributes,
    unused_variables,
    dead_code
)]

use core::amethyst::ecs::{Component, FlaggedStorage, VecStorage};
use core::defs::{
    body::{BodyDefinition, Joint, Part, PartLayer},
    DefinitionComponent, DefinitionStorage,
};
use survival_derive::DefinitionComponent;
pub mod bundle;
pub mod inventory;
pub mod systems;

pub mod components {
    pub use crate::BodyComponent;
}

#[derive(Default, Debug)]
pub struct LayerState {
    pub layer_idx: usize,
}
impl LayerState {
    pub fn new(layer_idx: usize, _layer: &PartLayer) -> Self {
        Self { layer_idx }
    }
}

#[derive(Default, Debug)]
pub struct PartState {
    pub node_idx: usize,
    pub layer_states: Vec<LayerState>,
}
impl PartState {
    pub fn new(node_idx: usize, part: &Part) -> Self {
        Self {
            node_idx,
            layer_states: part
                .layers
                .iter()
                .enumerate()
                .map(|(idx, l)| LayerState::new(idx, l))
                .collect(),
        }
    }
}

#[derive(Default, Debug)]
pub struct JointState {
    pub joint_idx: usize,
}
impl JointState {
    pub fn new(joint_idx: usize, _part: &Joint) -> Self {
        Self { joint_idx }
    }
}

#[derive(DefinitionComponent, Debug)]
#[def(BodyDefinition)]
pub struct BodyComponent {
    pub part_states: Vec<PartState>,
    pub joint_states: Vec<JointState>,
    pub def: u32,
}
impl Component for BodyComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl BodyComponent {
    pub fn new(id: u32, storage: &DefinitionStorage<BodyDefinition>) -> Self {
        let definition = storage.get(id).unwrap();

        Self {
            part_states: definition
                .part_graph
                .as_ref()
                .unwrap()
                .raw_nodes()
                .iter()
                .enumerate()
                .map(|(idx, n)| PartState::new(idx, &n.weight))
                .collect(),
            joint_states: definition
                .part_graph
                .as_ref()
                .unwrap()
                .raw_edges()
                .iter()
                .enumerate()
                .map(|(idx, n)| JointState::new(idx, &n.weight))
                .collect(),
            def: id,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn load_definition() {
        use core::defs::body::BodyDefinition;
        let file = std::fs::File::open("../resources/defs/bodies/humanoid.ron").unwrap();
        let body_def: Vec<BodyDefinition> = core::ron::de::from_reader(&file).unwrap();
    }
}
