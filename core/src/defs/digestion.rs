use crate::defs::{
    body::BodyDefinitionId, foliage::FoliageCategory, psyche::NeedDecay, Definition,
    InheritDefinition, Named,
};
use smallvec::SmallVec;
use strum_macros::AsRefStr;
use survival_derive::NamedDefinition;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize, AsRefStr,
)]
pub enum EdibleState {
    Raw,
    Cooked,
    Rotten,
    Burnt,
    Any,
}
impl Default for EdibleState {
    fn default() -> Self {
        EdibleState::Raw
    }
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize, AsRefStr,
)]
pub enum EdibleKind {
    Meat(BodyDefinitionId),
    Foliage(Option<FoliageCategory>),
    Liquid,
    Any,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct DigestionPart {
    body_part_name: String,

    capacity: u32,
    capacity_decay: NeedDecay,

    calorie_efficiency: u32,
    hydration_effiency: u32,
}

#[derive(NamedDefinition, Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DigestionDefinition {
    name: String,

    inherits: Option<String>,

    #[serde(skip)]
    id: Option<u32>,

    can_eat: Option<SmallVec<[(EdibleState, EdibleKind); 9]>>,

    #[serde(default)]
    digestion_parts: Vec<DigestionPart>,

    #[serde(default)]
    calorie_burn_rate: Option<NeedDecay>,

    #[serde(default)]
    hydration_burn_rate: Option<NeedDecay>,
}

impl InheritDefinition for DigestionDefinition {
    fn inherit_from(&mut self, parent: &Self) {
        if self.can_eat.is_none() {
            self.can_eat = parent.can_eat.clone();
        }
        self.calorie_burn_rate = self
            .calorie_burn_rate
            .map_or(parent.calorie_burn_rate, Some);
        self.hydration_burn_rate = self
            .hydration_burn_rate
            .map_or(parent.hydration_burn_rate, Some);

        parent
            .digestion_parts
            .iter()
            .for_each(|p| self.digestion_parts.push(p.clone()));
    }

    fn parent(&self) -> Option<&str> {
        self.inherits.as_ref().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::init_test_log;

    #[test]
    fn digestion_serialize() {
        init_test_log();

        let b = DigestionDefinition::default();

        let buildings = vec![b];

        let serialized = ron::ser::to_string_pretty(
            &buildings,
            ron::ser::PrettyConfig::new()
                .with_depth_limit(10)
                .with_separate_tuple_members(false)
                .with_enumerate_arrays(false)
                .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME),
        )
        .unwrap();
        println!("{}", serialized);
    }

    #[test]
    fn digestion_deserialize() -> Result<(), failure::Error> {
        init_test_log();
        let f = std::fs::File::open("/home/jaynus/dev/survival/resources/defs/digestion.ron")?;
        let _: Vec<DigestionDefinition> = ron::de::from_reader(f)?;

        Ok(())
    }
}
