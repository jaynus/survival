use rayon::prelude::*;

use super::CurrentGoalComponent;
use core::{
    amethyst::{
        core::{SystemDesc, Time},
        ecs::{
            Component, Entities, Entity, ParJoin, Read, ReadExpect, ReadStorage, System,
            SystemData, VecStorage, World, WriteStorage,
        },
    },
    components::PropertiesComponent,
    defs::{
        action::{ActionConditionValue, ActionDefinition},
        DefinitionStorage,
    },
    fsm,
};
use derivative::Derivative;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct GoapPlannerComponent {
    condition_cache: Arc<Mutex<HashMap<fsm::Condition<ActionConditionValue>, (u64, bool)>>>,
}
impl GoapPlannerComponent {
    pub fn new(
        base_conditions: HashMap<fsm::Condition<ActionConditionValue>, (u64, bool)>,
    ) -> Self {
        Self {
            condition_cache: Arc::new(Mutex::new(base_conditions)),
        }
    }
}
impl Component for GoapPlannerComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
struct GoapAction<'a> {
    action: &'a ActionDefinition,
    conditions: Vec<&'a fsm::Condition<ActionConditionValue>>,
    conditions_map: Vec<&'a dyn goap::Condition<GoapState<'a>>>,
}
impl<'a> GoapAction<'a> {
    pub fn new(action: &'a ActionDefinition) -> Self {
        let conditions_map = action
            .conditions
            .iter()
            .map(|c| c as &dyn goap::Condition<GoapState>)
            .collect();
        Self {
            action,
            conditions_map,
            conditions: action.conditions.iter().collect(),
        }
    }
}
impl<'a, 'state> goap::Action<GoapState<'a>> for GoapAction<'a> {
    fn unique_id(&self) -> u64 { self.action.id.unwrap() as u64 }

    fn conditions(&self) -> &[&dyn goap::Condition<GoapState<'a>>] {
        self.conditions_map.as_slice()
    }

    fn apply(&self, state: &mut GoapState<'a>) {}
}

fn check_condition(condition: &fsm::Condition<ActionConditionValue>, state: &GoapState) -> bool {
    use core::fsm::{ConditionKind, ConditionTarget};
    log::trace!("Checking condition: {:?}", condition);

    let result = match condition.target {
        ConditionTarget::Me => match condition.kind {
            ConditionKind::Has => true,
            ConditionKind::Near(distance) => unimplemented!(),
        },
        ConditionTarget::Entity => {
            // Search to see if an entity matches
            false
        }
    };

    log::trace!("returning: {}", result);

    if let Ok(mut condition_cache) = state.condition_cache.lock() {
        condition_cache.insert(condition.clone(), (state.current_frame, result));
    }

    result
}

impl<'a> goap::Condition<GoapState<'a>> for fsm::Condition<ActionConditionValue> {
    fn unique_id(&self) -> u64 { calculate_hash(self) }
    fn check(&self, state: &GoapState<'a>) -> bool {
        if let Some(result) = state.condition_cache.lock().unwrap().get(self) {
            if result.0 == state.current_frame {
                log::trace!("{:?} - Returning cached result: {}", self, result.1);
                return result.1;
            }
        }

        check_condition(self, state)
    }
}

#[derive(Derivative, Clone)]
#[derivative(Debug(bound = ""))]
pub struct GoapState<'a> {
    current_frame: u64,
    source_entity: Entity,
    condition_cache: Arc<Mutex<HashMap<fsm::Condition<ActionConditionValue>, (u64, bool)>>>,

    #[derivative(Debug = "ignore")]
    storage: &'a <PropertiesComponent as Component>::Storage,
}

pub struct GoapPlannerSystem {
    condition_cache: HashMap<fsm::Condition<ActionConditionValue>, (u64, bool)>,
}

impl<'s> System<'s> for GoapPlannerSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, Time>,
        ReadExpect<'s, DefinitionStorage<ActionDefinition>>,
        WriteStorage<'s, GoapPlannerComponent>,
        ReadStorage<'s, CurrentGoalComponent>,
        ReadStorage<'s, PropertiesComponent>,
    );

    fn run(
        &mut self,
        (entities, time, action_defs, mut planner_storage, goal_storage, properties_storage): Self::SystemData,
    ) {
        let actions = action_defs
            .raw_storage()
            .iter()
            .map(|action| GoapAction::new(action))
            .collect::<Vec<_>>();
        let action_refs = actions
            .iter()
            .map(|a| a as &dyn goap::Action<GoapState>)
            .collect::<Vec<_>>();

        log::trace!("GoapPlannerSystem::run");
        (&entities, &mut planner_storage, &goal_storage)
            .par_join()
            .for_each(|(source_entity, planner_data, goal_data)| {
                log::trace!("GoapPlannerSystem::iter");
                let state = GoapState {
                    source_entity,
                    current_frame: time.frame_number(),
                    condition_cache: planner_data.condition_cache.clone(),
                    storage: properties_storage.unprotected_storage(),
                };

                let goal_action = action_defs.get(goal_data.action_id).unwrap();
                let goal_conditions_refs = goal_action
                    .conditions
                    .iter()
                    .map(|a| a as &dyn goap::Condition<GoapState>)
                    .collect::<Vec<_>>();

                let plan = goap::Planner::plan(&state, &goal_conditions_refs, &action_refs);

                log::trace!("Resolved plan: {:?}", plan);
            });
    }
}

#[derive(Default)]
pub struct GoapPlannerSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, GoapPlannerSystem> for GoapPlannerSystemDesc {
    fn build(self, world: &mut World) -> GoapPlannerSystem {
        <GoapPlannerSystem as System<'_>>::SystemData::setup(world);

        log::trace!("GoapPlannerSystemDesc::build()");

        let frame = world.fetch::<Time>().frame_number();

        let action_defs = world.fetch::<DefinitionStorage<ActionDefinition>>();
        let mut condition_cache = HashMap::with_capacity(action_defs.len());
        for action in action_defs.iter() {
            for condition in &action.conditions {
                condition_cache.insert(condition.clone(), (frame, false));
            }
        }
        GoapPlannerSystem { condition_cache }
    }
}

fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {
    use std::hash::Hasher;
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        amethyst::ecs::{Builder, WorldExt},
        defs::Named,
    };

    fn add_test_goap_stuff(system: &GoapPlannerSystem, world: &mut World) {
        let action_id = world
            .fetch::<DefinitionStorage<ActionDefinition>>()
            .find("Fell Tree")
            .unwrap()
            .id()
            .unwrap();

        world
            .create_entity()
            .with(GoapPlannerComponent::new(system.condition_cache.clone()))
            .with(CurrentGoalComponent { action_id })
            .build();

        world.fetch_mut::<Time>().increment_frame_number();
    }

    #[test]
    fn simple_planner_test() {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut world = World::new();

        <GoapPlannerSystem as System>::SystemData::setup(&mut world);
        world.insert(Time::default());

        world.insert(
            DefinitionStorage::<ActionDefinition>::from_folder(
                "/home/jaynus/dev/survival/resources/defs/actions",
            )
            .unwrap(),
        );

        let mut system = GoapPlannerSystemDesc::default().build(&mut world);

        add_test_goap_stuff(&system, &mut world);
        let data = <GoapPlannerSystem as System>::SystemData::fetch(&world);
        system.run(data);
    }
}
