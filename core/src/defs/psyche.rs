use crate::defs::{Definition, Named};
use arr_macro::arr;
use num_derive::{FromPrimitive, ToPrimitive};
use shrinkwraprs::Shrinkwrap;
use strum_macros::{AsRefStr, EnumCount, EnumIter};
use survival_derive::NamedDefinition;
pub type PsycheTraitId = u32;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    AsRefStr,
    EnumIter,
    EnumCount,
    FromPrimitive,
    ToPrimitive,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(u8)]
pub enum NeedKind {
    Creativity = 0,
    Social = 1,
    Love = 2,
    Safety = 3,
    HungerTolerance = 4,
    ThirstTolerance = 5,
    PainTolerance = 6,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, serde::Deserialize, serde::Serialize)]
pub struct NeedDecay {
    pub value: i16,
    #[serde(default = "NeedDecay::minmax")]
    pub minmax: (i16, i16),
    pub time: u32,
}
impl Default for NeedDecay {
    fn default() -> Self {
        Self {
            value: -5,
            minmax: Self::minmax(),
            time: 3600,
        }
    }
}
impl NeedDecay {
    fn minmax() -> (i16, i16) {
        (i8::min_value().into(), i8::max_value().into())
    }
}

#[derive(Debug, Copy, Clone, serde::Deserialize, serde::Serialize)]
pub struct NeedState {
    pub value: i16,
    pub weight: i16,
    pub decay: NeedDecay,
    #[serde(skip)]
    pub acc: u32,
}
impl Default for NeedState {
    fn default() -> Self {
        Self {
            value: 0,
            weight: 0,
            decay: NeedDecay::default(),
            acc: 0,
        }
    }
}

#[derive(Shrinkwrap, Debug, Clone, serde::Deserialize, serde::Serialize)]
#[shrinkwrap(mutable)]
pub struct NeedsContainer(pub [(NeedKind, NeedState); NEEDKIND_COUNT]);
impl Default for NeedsContainer {
    fn default() -> Self {
        use strum::IntoEnumIterator;
        let mut iter = NeedKind::iter();
        Self(arr![(iter.next().unwrap(), NeedState::default()); 7])
    }
}
impl NeedsContainer {
    pub fn set_need(&mut self, kind: NeedKind, value: NeedState) {
        assert_eq!(self.0[kind as usize].0, kind);
        self.0[kind as usize].1 = value;
    }

    pub fn need(&self, kind: NeedKind) -> &NeedState {
        assert_eq!(self.0[kind as usize].0, kind);
        &self.0[kind as usize].1
    }

    pub fn need_mut(&mut self, kind: NeedKind) -> &mut NeedState {
        assert_eq!(self.0[kind as usize].0, kind);
        &mut self.0[kind as usize].1
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum NeedEffectValue {
    Static(i16),
    Decay(NeedDecay),
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NeedEffect {
    pub kind: NeedKind,
    pub value: NeedEffectValue,
}
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum PsycheTraitEffectKind {
    NeedEffect(NeedEffect),
    None,
}

#[derive(NamedDefinition, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PsycheTraitDefinition {
    name: String,

    description: String,

    #[serde(default)]
    id: Option<u32>,

    pub effects: Vec<PsycheTraitEffectKind>,
}

pub type PsycheTraitRef = (String, f32);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::{race::RaceDefinition, DefinitionStorage};
    use crate::tests::init_test_log;

    #[test]
    pub fn gen_race_psyche() -> Result<(), failure::Error> {
        init_test_log();

        let _races = DefinitionStorage::<RaceDefinition>::from_folder("../resources/defs/races")?;

        let _traits =
            DefinitionStorage::<PsycheTraitDefinition>::from_folder("../resources/defs/psyche")?;

        let test_trait = PsycheTraitDefinition {
            name: "Introvert".to_string(),
            description: "".to_string(),
            id: None,
            effects: vec![PsycheTraitEffectKind::NeedEffect(NeedEffect {
                kind: NeedKind::Social,
                value: NeedEffectValue::Decay(NeedDecay::default()),
            })],
        };
        println!(
            "{}",
            ron::ser::to_string_pretty(
                &test_trait,
                ron::ser::PrettyConfig::new()
                    .with_depth_limit(10)
                    .with_separate_tuple_members(false)
                    .with_enumerate_arrays(false)
                    .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME),
            )?
        );

        Ok(())
    }
}
