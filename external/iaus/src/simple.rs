use crate::{Bucket, Decision, State};
use derivative::Derivative;
use priority_queue::PriorityQueue;
use rayon::prelude::*;
use std::{fmt::Debug, hash::Hash, sync::Arc};

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct SimpleResolver<S, P>
where
    S: State,
    P: Debug + Ord + Hash + Send + Sync,
{
    buckets: PriorityQueue<Arc<Bucket<S, P>>, P>,
}
impl<S, P> SimpleResolver<S, P>
where
    S: State,
    P: Debug + Copy + Ord + Hash + Send + Sync,
{
    pub fn with_buckets(buckets: &[Arc<Bucket<S, P>>]) -> Self {
        let mut res = Self::default();

        buckets.iter().for_each(|bucket| {
            res.add_bucket(bucket.clone());
        });

        res
    }

    pub fn add_bucket(&mut self, bucket: Arc<Bucket<S, P>>) {
        let priority = bucket.priority;
        self.buckets.push(bucket, priority);
    }

    pub fn iter_buckets(&self) -> impl Iterator<Item = (&Arc<Bucket<S, P>>, &P)> {
        self.buckets.iter()
    }

    pub fn get_decisions(
        &self,
        state: &S,
    ) -> Vec<(Arc<Bucket<S, P>>, Vec<(Arc<dyn Decision<S>>, f32)>)> {
        let buckets = self.iter_buckets().collect::<Vec<_>>();

        buckets
            .par_iter()
            .filter_map(|bucket| {
                let bucket_results = bucket
                    .0
                    .decisions
                    .par_iter()
                    .filter_map(|decision| {
                        let score = decision.score(state);
                        if score > 0.0 {
                            Some((decision.clone(), score))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if bucket_results.is_empty() {
                    None
                } else {
                    Some((bucket.0.clone(), bucket_results))
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn get_top_decision(&self, state: &S) -> (Arc<Bucket<S, P>>, Arc<dyn Decision<S>>, f32) {
        let full_execution = self.get_decisions(state);
        (
            full_execution[0].0.clone(),
            full_execution[0].1[0].0.clone(),
            full_execution[0].1[0].1,
        )
    }
}
