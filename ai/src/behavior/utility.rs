use super::*;
use core::amethyst::core::{legion::*, Time};
use crossbeam::queue::SegQueue;
use rayon::prelude::*;

pub struct UtilitySystem<R> {
    runner: R,
}
impl<R> UtilitySystem<R>
where
    R: FnMut(&mut World),
{
    fn with(runner: R) -> Self { Self { runner } }
}
impl<R> ThreadLocal for UtilitySystem<R>
where
    R: FnMut(&mut World),
{
    fn run(&mut self, world: &mut World) { (self.runner)(world) }
    fn dispose(self, world: &mut World) {}
}
pub struct UtilitySystemDesc;
impl UtilitySystemDesc {
    fn build(mut self, world: &mut World) -> Box<dyn ThreadLocal> {
        let mut query = <(Write<UtilityStateComponent>)>::query();

        let runner = move |world: &mut World| {
            let mut updated = {
                let time = world.resources.get::<Time>().unwrap();

                let timestamp = time.absolute_time();

                query.par_for_each(world, |mut utility| {
                    utility.available_decisions = utility
                        .available_decisions
                        .par_iter()
                        .map(|entry| {
                            let mut entry = entry.clone();
                            entry.last_tick = timestamp;
                            entry.last_score = entry.decision.score(world);

                            entry
                        })
                        .collect();
                });
            };
        };

        Box::new(UtilitySystem::with(runner))
    }
}
