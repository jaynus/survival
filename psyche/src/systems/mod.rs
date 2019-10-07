use crate::components::{PersonalityComponent, PyscheNeedsComponent};
use core::{
    amethyst::{
        core::{SystemBundle, SystemDesc},
        derive::SystemDesc,
        ecs::{
            prelude::DispatcherBuilder, ParJoin, Read, ReadStorage, System, SystemData, World,
            WriteStorage,
        },
    },
    clock::{Instant, WorldTime},
    components::AttributesComponent,
    defs::{psyche::PsycheTraitDefinition, DefinitionStorage},
    rayon::prelude::*,
};

#[derive(Default, SystemDesc)]
pub struct NeedsDecaySystem {
    pub last: Instant,
}
impl<'s> System<'s> for NeedsDecaySystem {
    type SystemData = (
        Read<'s, WorldTime>,
        Read<'s, DefinitionStorage<PsycheTraitDefinition>>,
        ReadStorage<'s, AttributesComponent>,
        WriteStorage<'s, PersonalityComponent>,
        WriteStorage<'s, PyscheNeedsComponent>,
    );

    fn run(
        &mut self,
        (time, trait_defs, attributes_storage, mut personality_storage, mut needs_storage): Self::SystemData,
    ) {
        // Skip execution if the game time hasn't progressed
        if self.last == time.now() {
            return;
        }

        (
            &attributes_storage,
            &mut personality_storage,
            &mut needs_storage,
        )
            .par_join()
            .for_each(|(a, b, c)| self.tick(&time, &trait_defs, a, b, c));

        self.reset(time.now());
    }
}
impl NeedsDecaySystem {
    pub fn reset(&mut self, time: Instant) {
        self.last = time;
    }

    fn tick(
        &self,
        time: &WorldTime,
        trait_defs: &DefinitionStorage<PsycheTraitDefinition>,
        _attributes: &AttributesComponent,
        personality: &mut PersonalityComponent,
        needs: &mut PyscheNeedsComponent,
    ) {
        let time_elapsed = (time.now() - self.last).value();
        needs.iter_mut().for_each(|(_, state)| {
            if state.decay.value == 0 || state.decay.time == 0 {
                return;
            }

            state.acc += time_elapsed as u32;
            while state.acc >= state.decay.time {
                let new_value = state.value + state.decay.value;
                if state.decay.minmax.0 < new_value && state.decay.minmax.1 > new_value {
                    state.value = new_value;
                }

                state.acc -= state.decay.time;
            }
        });

        use core::defs::psyche::{NeedEffectValue, PsycheTraitEffectKind};

        // Iterate through the traits, if any are a Decay type, we apply them
        personality
            .traits
            .iter_mut()
            .filter(|(active, _, _)| *active)
            .for_each(|(_, trait_id, accums)| {
                let def = trait_defs.get(*trait_id).unwrap();
                def.effects.iter().enumerate().for_each(|(i, effect)| {
                    if let PsycheTraitEffectKind::NeedEffect(effect) = effect {
                        if let NeedEffectValue::Decay(decay) = effect.value {
                            if decay.value == 0 || decay.time == 0 {
                                return;
                            }

                            let mut accum = accums.get(&i).unwrap_or(&0) + time_elapsed as u32;
                            while accum >= decay.time {
                                let new_value = needs.need(effect.kind).value + decay.value;
                                if decay.minmax.0 < new_value && decay.minmax.1 > new_value {
                                    needs.need_mut(effect.kind).value = new_value;
                                }

                                accum -= decay.time;
                            }

                            accums.insert(i, accum);
                        }
                    }
                });
            });
    }
}

#[derive(Default)]
pub struct PsycheBundle;
impl<'a, 'b> SystemBundle<'a, 'b> for PsycheBundle {
    fn build(
        self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), core::amethyst::Error> {
        builder.add(NeedsDecaySystem::default(), "NeedsDecaySystem", &[]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{defs::psyche::NeedKind, tests::init_test_log};

    #[test]
    fn tick_decay() -> Result<(), core::failure::Error> {
        init_test_log();
        let mut sys = NeedsDecaySystem::default();

        let traits =
            DefinitionStorage::<PsycheTraitDefinition>::from_folder("resources/defs/psyche")?;

        let time = WorldTime::default();
        let attributes = AttributesComponent::default();
        let mut personality = PersonalityComponent::default();
        let mut needs = PyscheNeedsComponent::default();

        assert_eq!(needs.need(NeedKind::Creativity).value, 0);

        time.elapse_raw(10.0);
        sys.tick(&time, &traits, &attributes, &mut personality, &mut needs);
        sys.last = time.now();
        assert_eq!(needs.need(NeedKind::Creativity).value, -1);

        time.elapse_raw(10.0);
        sys.tick(&time, &traits, &attributes, &mut personality, &mut needs);
        assert_eq!(needs.need(NeedKind::Creativity).value, -2);

        Ok(())
    }
}
