use rayon::prelude::*;

use super::CurrentGoalComponent;
use core::{
    amethyst::core::{
        legion::{system::PreparedWorld, *},
        Time,
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

pub type ConditionCachePtr = Arc<Mutex<ConditionCache>>;
pub type ConditionCache = HashMap<fsm::Condition<ActionConditionValue>, (u64, bool)>;

pub struct GoapPlannerComponent {
    condition_cache: ConditionCachePtr,
}
impl GoapPlannerComponent {
    pub fn new(base_conditions: ConditionCache) -> Self {
        Self {
            condition_cache: Arc::new(Mutex::new(base_conditions)),
        }
    }
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
    fn unique_id(&self) -> u64 { u64::from(self.action.id.unwrap()) }

    fn conditions(&self) -> &[&dyn goap::Condition<GoapState<'a>>] {
        self.conditions_map.as_slice()
    }

    fn apply(&self, _: &mut GoapState<'a>) {}
}

fn check_condition(condition: &fsm::Condition<ActionConditionValue>, state: &GoapState) -> bool {
    use core::fsm::{ConditionKind, ConditionTarget};
    log::trace!("Checking condition: {:?}", condition);

    let result = match condition.target {
        ConditionTarget::Me => match condition.kind {
            ConditionKind::Has => true,
            ConditionKind::Near(_distance) => unimplemented!(),
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
    condition_cache: ConditionCachePtr,

    #[derivative(Debug = "ignore")]
    storage: &'a PreparedWorld,
}

#[derive(Default)]
pub struct GoapPlannerSystemDesc {}
impl SystemDesc for GoapPlannerSystemDesc {
    fn build(mut self, world: &mut World) -> Box<dyn Schedulable> {
        let frame = world.resources.get::<Time>().unwrap().frame_number();

        let action_defs = world
            .resources
            .get::<DefinitionStorage<ActionDefinition>>()
            .unwrap();
        let mut condition_cache = HashMap::with_capacity(action_defs.len());
        for action in action_defs.iter() {
            for condition in &action.conditions {
                condition_cache.insert(condition.clone(), (frame, false));
            }
        }
        world.resources.insert(condition_cache);

        SystemBuilder::<()>::new("GoapPlannerSystem")
            .read_resource::<Time>()
            .read_resource::<DefinitionStorage<ActionDefinition>>()
            .read_component::<PropertiesComponent>()
            .with_query(<(Read<CurrentGoalComponent>, Write<GoapPlannerComponent>)>::query())
            .build(move |_, world, (time, action_defs), query| {
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
                query.par_entities_for_each(|(source_entity, (mut goal_data, planner_data))| {
                    log::trace!("GoapPlannerSystem::iter");
                    let state = GoapState {
                        source_entity,
                        current_frame: time.frame_number(),
                        condition_cache: planner_data.condition_cache.clone(),
                        storage: world,
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
            })
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
        amethyst::core::legion::{Universe, World},
        defs::Named,
    };

    fn add_test_goap_stuff(world: &mut World) {
        let action_id = world
            .resources
            .get::<DefinitionStorage<ActionDefinition>>()
            .unwrap()
            .find("Fell Tree")
            .unwrap()
            .id()
            .unwrap();

        let cache = world.resources.get::<ConditionCache>().unwrap().clone();

        world.insert(
            (),
            vec![(
                CurrentGoalComponent { action_id },
                GoapPlannerComponent::new(cache),
            )],
        );

        world
            .resources
            .get_mut::<Time>()
            .unwrap()
            .increment_frame_number();
    }

    #[test]
    fn simple_planner_test() {
        let _ = env_logger::builder().is_test(true).try_init();

        let universe = Universe::new();
        let mut world = universe.create_world();

        world.resources.insert(Time::default());

        world.resources.insert(
            DefinitionStorage::<ActionDefinition>::from_folder(
                "/home/jaynus/dev/survival/resources/defs/actions",
            )
            .unwrap(),
        );

        let mut system = GoapPlannerSystemDesc::default().build(&mut world);

        add_test_goap_stuff(&mut world);

        system.run(&world);
    }
}
