use core::amethyst::ecs::{Component, VecStorage, World};
use iaus::NamedDecision;
use std::{collections::HashSet, hash::Hash, sync::Arc, time::Duration};

pub mod considerations;
pub mod planner;
pub mod utility;

pub struct CurrentGoalComponent {
    action_id: u32,
}
impl Component for CurrentGoalComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct UtilityStateComponent {
    pub available_decisions: HashSet<DecisionEntry>,
}
impl Component for UtilityStateComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct DecisionEntry {
    decision: Arc<dyn NamedDecision<World>>,
    last_score: f32,
    last_tick: Duration,
}
impl Hash for DecisionEntry {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) { self.decision.name().hash(hasher) }
}
impl PartialEq for DecisionEntry {
    fn eq(&self, other: &DecisionEntry) -> bool { self.decision.name().eq(&other.decision.name()) }
}
impl Eq for DecisionEntry {}
