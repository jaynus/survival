use core::amethyst::{
    core::Time,
    ecs::{Entities, ParJoin, Read, ReadStorage, RunNow, SystemData, World, WriteStorage},
};
use rayon::prelude::*;

use super::*;

type UtilitySystemData<'s> = (
    Read<'s, Time>,
    Entities<'s>,
    ReadStorage<'s, UtilityStateComponent>,
);

pub struct UtilitySystem {}
impl<'s> RunNow<'s> for UtilitySystem {
    fn setup(&mut self, world: &mut World) {
        log::trace!("UtilitySystem::setup()");
        UtilitySystemData::setup(world);
    }

    fn run_now(&mut self, world: &World) {
        let mut updated = {
            let (time, entities, utility_storage) = UtilitySystemData::fetch(world);
            let timestamp = time.absolute_time();

            (&entities, &utility_storage)
                .par_join()
                .map(move |(entity, utility)| {
                    (
                        entity,
                        utility
                            .available_decisions
                            .par_iter()
                            .map(|entry| {
                                let mut entry = entry.clone();
                                entry.last_tick = timestamp;
                                entry.last_score = entry.decision.score(world);

                                entry
                            })
                            .collect(),
                    )
                })
                .collect::<Vec<_>>()
        };

        {
            let (mut utility_storage,) = <(WriteStorage<'_, UtilityStateComponent>,)>::fetch(world);

            updated.drain(..).for_each(|(entity, decisions)| {
                let utility = utility_storage.get_mut(entity).unwrap();
                utility.available_decisions = decisions;
            });
        }
    }

    fn dispose(self: Box<Self>, _: &mut World) {
        log::trace!("Dispose UtilitySystem");
    }
}
