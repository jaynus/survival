use crate::pathing::PathingResult;
use core::{
    amethyst::ecs::{Component, VecStorage},
    fsm::{ActionEvent, ActionStatus},
};

pub use crate::behavior::UtilityStateComponent;

#[derive(Default, Clone)]
pub struct CurrentPathingComponent {
    pub current_path: Option<PathingResult>,
}
impl CurrentPathingComponent {
    pub fn new(current_path: Option<PathingResult>) -> Self { Self { current_path } }
    pub fn finished(&mut self) { self.current_path = None; }
}
impl Component for CurrentPathingComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct CurrentActionComponent {
    pub inner: ActionEvent,
    pub status: ActionStatus,
}
impl CurrentActionComponent {
    pub fn new(inner: ActionEvent) -> Self {
        Self {
            inner,
            status: ActionStatus::default(),
        }
    }
}
impl Component for CurrentActionComponent {
    type Storage = VecStorage<Self>;
}
