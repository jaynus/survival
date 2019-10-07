use crate::{
    defs::{property::Property, Definition, Named},
    fsm::{Condition, Event},
    shrinkwraprs::Shrinkwrap,
};
use survival_derive::NamedDefinition;

#[derive(
    Shrinkwrap,
    Debug,
    Default,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ActionId(u32);
impl From<u32> for ActionId {
    fn from(id: u32) -> ActionId { ActionId(id) }
}
impl From<usize> for ActionId {
    fn from(id: usize) -> ActionId { ActionId(id as u32) }
}
impl Into<usize> for ActionId {
    fn into(self) -> usize { self.0 as usize }
}
impl Into<u32> for ActionId {
    fn into(self) -> u32 { self.0 }
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ActionCategory {
    Unspecified,
}
impl Default for ActionCategory {
    fn default() -> Self { ActionCategory::Unspecified }
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ActionSourceType {
    World,
    Region,
    Pawn,
    Attack,
    Ingest,
}
impl Default for ActionSourceType {
    fn default() -> Self { ActionSourceType::Pawn }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub enum ActionConditionValue {
    Property(Property),
    Item,
    Target,
}

#[derive(NamedDefinition, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionDefinition {
    pub name: String,

    #[serde(skip)]
    pub id: Option<u32>,

    pub category: ActionCategory,

    pub event: Event,

    pub adjective: String,

    #[serde(default)]
    pub source: ActionSourceType,
    pub base_time: f32,

    #[serde(default)]
    pub conditions: Vec<Condition<ActionConditionValue>>,

    #[serde(default)]
    pub targets: Vec<Condition<ActionConditionValue>>,

    #[serde(default)]
    pub post_conditions: Vec<(Condition<ActionConditionValue>, bool)>,
}
impl Eq for ActionDefinition {}
impl PartialEq for ActionDefinition {
    fn eq(&self, _: &Self) -> bool { false }
}
