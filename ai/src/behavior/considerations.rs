use core::amethyst::ecs::World;
use iaus::{curves, ConsiderationFn, Description};

use std::sync::Arc;

pub fn hunger() -> Arc<ConsiderationFn<World>> {
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
    ))
}
