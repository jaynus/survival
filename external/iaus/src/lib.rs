use derivative::Derivative;
use rayon::prelude::*;
use std::{fmt::Debug, hash::Hash, ops::RangeBounds, sync::Arc};

pub mod curves;
pub mod simple;
pub use curves::Curve;

pub trait NamedDecision<S: State>: Decision<S> + Description {}
pub trait NamedConsideration<S: State>: Consideration<S> + Description {}

pub trait State: Send + Sync {}
impl<T> State for T where T: Send + Sync {}

pub trait Description {
    fn name(&self) -> Option<&str> { None }
    fn description(&self) -> Option<&str> { None }
}

pub trait Consideration<S>: Debug + Send + Sync
where
    S: State,
{
    fn score(&self, state: &S) -> f32;
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct ConsiderationFn<S, R = std::ops::Range<f32>>
where
    S: State,
    R: RangeBounds<f32> + Send + Sync,
{
    pub name: Option<String>,
    pub description: Option<String>,
    pub curve: Option<Box<dyn Curve<R>>>,
    #[derivative(Debug = "ignore")]
    pub function: Box<(dyn Fn(&ConsiderationFn<S, R>, &S) -> f32 + Send + Sync)>,
}
impl<S, R> ConsiderationFn<S, R>
where
    S: State,
    R: RangeBounds<f32> + Send + Sync,
{
    pub fn new(
        name: Option<&str>,
        description: Option<&str>,
        curve: Option<Box<dyn Curve<R>>>,
        function: Box<(dyn Fn(&ConsiderationFn<S, R>, &S) -> f32 + Send + Sync)>,
    ) -> Self {
        Self {
            name: name.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            curve,
            function,
        }
    }
}
impl<S> Description for ConsiderationFn<S>
where
    S: State,
{
    fn name(&self) -> Option<&str> { self.name.as_ref().map(|s| s.as_str()) }
    fn description(&self) -> Option<&str> { self.description.as_ref().map(|s| s.as_str()) }
}
impl<S, R> Consideration<S> for ConsiderationFn<S, R>
where
    S: State,
    R: RangeBounds<f32> + Send + Sync,
{
    fn score(&self, state: &S) -> f32 { (self.function)(self, state) }
}

pub trait Decision<S>: Debug + Send + Sync
where
    S: State,
{
    fn considerations(&self) -> &[Arc<dyn Consideration<S>>];

    fn base(&self) -> f32 { 1.0 }

    fn score(&self, state: &S) -> f32 {
        let scores = self
            .considerations()
            .par_iter()
            .filter_map(|consider| {
                let score = consider.score(state);
                if score > 0.0 {
                    Some(score.min(1.0).max(0.0))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let base = self.base();
        let modifier = 1.0 - (1.0 / self.considerations().len() as f32);
        scores.into_par_iter().reduce(
            || base,
            |mut acc, score| {
                acc *= score;
                acc + ((1.0 - acc) * modifier * acc)
            },
        )
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct SimpleDecision<S>
where
    S: State,
{
    name: String,
    description: Option<String>,
    considerations: Vec<Arc<dyn Consideration<S>>>,
}
impl<S> SimpleDecision<S>
where
    S: State,
{
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            considerations: Vec::default(),
        }
    }
    pub fn with_consideration(mut self, consideration: Arc<dyn Consideration<S>>) -> Self {
        self.considerations.push(consideration);

        self
    }
}
impl<S> Description for SimpleDecision<S>
where
    S: State,
{
    fn name(&self) -> Option<&str> { Some(self.name.as_ref()) }
    fn description(&self) -> Option<&str> { self.description.as_ref().map(|s| s.as_str()) }
}
impl<S> Decision<S> for SimpleDecision<S>
where
    S: State,
{
    fn considerations(&self) -> &[Arc<dyn Consideration<S>>] { self.considerations.as_slice() }
}

#[derive(Debug)]
pub struct Bucket<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
    name: String,
    description: Option<String>,
    decisions: Vec<Arc<dyn Decision<S>>>,
    priority: P,
}
impl<S, P> Bucket<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
    pub fn new(name: &str, priority: P) -> Self {
        Self {
            priority,
            name: name.to_string(),
            description: None,
            decisions: Vec::default(),
        }
    }

    pub fn with_decision(mut self, consideration: Arc<dyn Decision<S>>) -> Self {
        self.decisions.push(consideration);

        self
    }
}
impl<S, P> Description for Bucket<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
    fn name(&self) -> Option<&str> { Some(self.name.as_ref()) }
    fn description(&self) -> Option<&str> { self.description.as_ref().map(|s| s.as_str()) }
}
impl<S, P> PartialEq for Bucket<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
    fn eq(&self, other: &Self) -> bool { self.priority == other.priority }
}
impl<S, P> Eq for Bucket<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
}

impl<S, P> PartialOrd for Bucket<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<S, P> Ord for Bucket<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.priority.cmp(&other.priority) }
}
impl<S, P> Hash for Bucket<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.priority.hash(state); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum TestStateEnum {
        One,
        Two,
        Three,
    }

    #[derive(Debug, Default)]
    struct TestState {
        blackboard: HashMap<String, TestStateEnum>,
    }

    #[test]
    fn basic_usage() {
        let _ = env_logger::builder().is_test(true).try_init();

        let considerations = [
            Arc::new(ConsiderationFn::new(
                Some("consideration 0 (Linear)"),
                None,
                Some(Box::new(curves::Linear {
                    range: std::ops::Range {
                        start: 0.0,
                        end: 256.0,
                    },
                    slope: 1.0,
                    intercept: 0.0,
                })),
                Box::new(|consideration, state| {
                    let result = consideration.curve.as_ref().unwrap().transform(155.0);
                    log::trace!("{} = {}", consideration.name().unwrap(), result);
                    result
                }),
            )),
            Arc::new(ConsiderationFn::new(
                Some("consideration 1 (Exp), 2.0exp"),
                None,
                Some(Box::new(curves::Exponential {
                    range: std::ops::Range {
                        start: 0.0,
                        end: 256.0,
                    },
                    power: 2.0,
                })),
                Box::new(|consideration, state| {
                    let result = consideration.curve.as_ref().unwrap().transform(155.0);
                    log::trace!("{} = {}", consideration.name().unwrap(), result);
                    result
                }),
            )),
        ];

        let decisions = [Arc::new(
            SimpleDecision::new("balls")
                .with_consideration(considerations[0].clone())
                .with_consideration(considerations[1].clone()),
        )];

        let state = TestState::default();

        let score = decisions[0].score(&state);
        assert_eq!(score, 0.46754268);

        let bucket = Arc::new(
            Bucket::<TestState, u32>::new("bucket", 1).with_decision(decisions[0].clone()),
        );

        let resolver = SimpleResolver::with_buckets(&[bucket.clone()]);
        println!("res = {:?}", resolver.get_top_decision(&state));
    }
}
