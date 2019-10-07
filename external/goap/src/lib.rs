#![allow(dead_code)]

use derivative::Derivative;
use rayon::prelude::*;
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub trait State: Clone + Debug + Send + Sync {}
impl<T> State for T where T: Clone + Debug + Send + Sync {}

pub trait Condition<S: State>: Debug + Send + Sync {
    fn unique_id(&self) -> u64;

    fn check(&self, state: &S) -> bool;
}
impl<S: State> PartialEq<dyn Condition<S>> for dyn Condition<S> {
    fn eq(&self, other: &dyn Condition<S>) -> bool { self.unique_id().eq(&other.unique_id()) }
}
impl<S: State> Eq for dyn Condition<S> {}

impl<S: State> Hash for dyn Condition<S> {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) { self.unique_id().hash(hasher) }
}

pub trait Action<S: State>: Debug + Send + Sync {
    fn unique_id(&self) -> u64;

    fn conditions(&self) -> &[&dyn Condition<S>];

    fn apply(&self, state: &mut S);

    fn check(&self, state: &S) -> bool {
        for condition in self.conditions() {
            if !condition.check(state) {
                return false;
            }
        }
        true
    }

    fn distance(&self, state: &S) -> usize {
        self.conditions()
            .par_iter()
            .filter_map(|condition| {
                if condition.check(state) {
                    None
                } else {
                    Some(1)
                }
            })
            .sum()
    }
}
impl<S: State> PartialEq<dyn Action<S>> for dyn Action<S> {
    fn eq(&self, other: &dyn Action<S>) -> bool { self.unique_id().eq(&other.unique_id()) }
}
impl<S: State> Eq for dyn Action<S> {}
impl<S: State> Hash for dyn Action<S> {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) { self.unique_id().hash(hasher) }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
struct PlanNode<'a, S: State> {
    action: Option<&'a dyn Action<S>>,
    state: S,
}
impl<'a, S: State> Hash for PlanNode<'a, S> {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        if let Some(action) = &self.action {
            action.unique_id().hash(hasher)
        } else {
            std::u64::MAX.hash(hasher)
        }
    }
}
impl<'a, S: State> PartialEq<PlanNode<'a, S>> for PlanNode<'a, S> {
    fn eq(&self, other: &PlanNode<S>) -> bool {
        if let Some(action) = &self.action {
            if let Some(other_action) = &other.action {
                action.unique_id().eq(&other_action.unique_id())
            } else {
                false
            }
        } else {
            false
        }
    }
}
impl<'a, S: State> Eq for PlanNode<'a, S> {}

impl<'a, S: State> PlanNode<'a, S> {
    fn new(state: S) -> Self {
        Self {
            action: None,
            state,
        }
    }
    fn with_action(action: &'a dyn Action<S>, state: S) -> Self {
        Self {
            action: Some(action),
            state,
        }
    }

    fn neighbors<'b>(
        &self,
        actions: &'b [&'b dyn Action<S>],
        goal: &'b [&'b dyn Condition<S>],
    ) -> Vec<(PlanNode<'b, S>, usize)> {
        let neighbors = actions
            .par_iter()
            .filter_map(|action| {
                if let Some(self_action) = &self.action {
                    if action.unique_id() == self_action.unique_id() {
                        return None;
                    }
                }

                let mut state_copy = self.state.clone();
                action.apply(&mut state_copy);

                if action.check(&state_copy) {
                    let distance = Self::distance(&state_copy, &Some(*action), goal);
                    Some((PlanNode::with_action(action.clone(), state_copy), distance))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        neighbors
    }

    fn distance(state: &S, action: &Option<&dyn Action<S>>, goal: &[&dyn Condition<S>]) -> usize {
        let distance = if let Some(_) = action {
            goal.par_iter()
                .filter_map(|condition| {
                    if !condition.check(&state) {
                        Some(1)
                    } else {
                        None
                    }
                })
                .sum()
        } else {
            goal.len()
        };

        distance
    }
}

#[derive(Default)]
pub struct Planner<S: State> {
    _marker: PhantomData<S>,
}
impl<S: State> Planner<S> {
    pub fn plan<'b>(
        state: &S,
        goal: &'b [&'b dyn Condition<S>],
        actions: &'b [&'b dyn Action<S>],
    ) -> Option<Vec<&'b dyn Action<S>>> {
        log::trace!("Beginning plan @ [{:?}]", goal);

        use pathfinding::prelude::astar;

        let start = PlanNode::<S>::new(state.clone());

        if let Some(plan) = astar(
            &start,
            |ref node| node.neighbors(actions, goal),
            |ref node| PlanNode::distance(&node.state, &node.action, goal),
            |ref node| PlanNode::distance(&node.state, &node.action, goal) == 0,
        ) {
            Some(
                plan.0
                    .iter()
                    .skip(1)
                    .map(|node| node.action.as_ref().unwrap().clone())
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, Clone)]
    pub struct TestState {
        value_one: bool,
        value_two: bool,
        value_three: bool,
    }

    #[derive(Default, Debug, Clone)]
    pub struct TestConditionOne {}
    impl Condition<TestState> for TestConditionOne {
        fn unique_id(&self) -> u64 { 1 }
        fn check(&self, state: &TestState) -> bool { state.value_one }
    }

    #[derive(Default, Debug, Clone)]
    pub struct TestConditionTwo {}
    impl Condition<TestState> for TestConditionTwo {
        fn unique_id(&self) -> u64 { 2 }
        fn check(&self, state: &TestState) -> bool { state.value_two }
    }

    #[derive(Default, Debug, Clone)]
    pub struct TestConditionThree {}
    impl Condition<TestState> for TestConditionThree {
        fn unique_id(&self) -> u64 { 3 }
        fn check(&self, state: &TestState) -> bool { state.value_three }
    }

    #[derive(Debug, Clone)]
    pub struct TestActionOne<'a> {
        condition: [&'a dyn Condition<TestState>; 1],
    }
    impl<'a> TestActionOne<'a> {
        pub fn new(condition: &'a dyn Condition<TestState>) -> Self {
            Self {
                condition: [condition],
            }
        }
    }
    impl<'a> Action<TestState> for TestActionOne<'a> {
        fn unique_id(&self) -> u64 { 1 }

        fn conditions(&self) -> &[&dyn Condition<TestState>] { &self.condition }

        fn apply(&self, state: &mut TestState) { state.value_two = true; }
    }

    #[derive(Debug, Clone)]
    pub struct TestActionTwo<'a> {
        condition: [&'a dyn Condition<TestState>; 1],
    }
    impl<'a> TestActionTwo<'a> {
        pub fn new(condition: &'a dyn Condition<TestState>) -> Self {
            Self {
                condition: [condition],
            }
        }
    }
    impl<'a> Action<TestState> for TestActionTwo<'a> {
        fn unique_id(&self) -> u64 { 2 }

        fn conditions(&self) -> &[&dyn Condition<TestState>] { &self.condition }

        fn apply(&self, state: &mut TestState) { state.value_three = true; }
    }

    #[test]
    fn simple_usage() {
        let _ = env_logger::builder().is_test(true).try_init();

        let conditions: [&dyn Condition<TestState>; 3] = [
            &TestConditionOne::default(),
            &TestConditionTwo::default(),
            &TestConditionThree::default(),
        ];
        let actions: [&dyn Action<TestState>; 2] = [
            &TestActionOne::new(conditions[0]),
            &TestActionTwo::new(conditions[1]),
        ];

        let goal: [&dyn Condition<TestState>; 1] = [conditions[2]];

        let state = TestState {
            value_one: true,
            ..Default::default()
        };

        let plan = Planner::plan(&state, &goal, &actions);
        println!("Plan = {:?}", plan);
    }
}
