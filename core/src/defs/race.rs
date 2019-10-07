use crate::{
    components::PropertiesComponent,
    defs::{
        property::Property,
        psyche::{NeedsContainer, PsycheTraitRef},
        Definition, HasProperties, Named,
    },
};
use survival_derive::NamedDefinition;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Attributes {
    // Body
    pub strength: u16,
    pub agility: u16,
    pub toughness: u16,
    pub endurance: u16,
    pub immunity: u16,
    pub healing: u16,

    // Mental
    pub analytical: u16,
    pub focus: u16,
    pub willpower: u16,
    pub creativity: u16,
    pub intuition: u16,
    pub patience: u16,
    pub memory: u16,
    pub linguistic: u16,
    pub spatial: u16,
    pub kinesthetic: u16,
    pub empathy: u16,
    pub social: u16,
}
impl Attributes {
    #[allow(clippy::too_many_lines)]
    pub fn generate<R>(rng: &mut R, def: &RaceDefinition) -> Self
    where
        R: crate::rand::Rng,
    {
        use crate::rand_distr::{Distribution, Normal};

        Self {
            strength: Normal::new(
                f32::from(def.attributes.0.strength),
                f32::from(def.attributes.1.strength),
            )
            .unwrap()
            .sample(rng) as u16,
            agility: Normal::new(
                f32::from(def.attributes.0.agility),
                f32::from(def.attributes.1.agility),
            )
            .unwrap()
            .sample(rng) as u16,
            toughness: Normal::new(
                f32::from(def.attributes.0.toughness),
                f32::from(def.attributes.1.toughness),
            )
            .unwrap()
            .sample(rng) as u16,
            endurance: Normal::new(
                f32::from(def.attributes.0.endurance),
                f32::from(def.attributes.1.endurance),
            )
            .unwrap()
            .sample(rng) as u16,
            immunity: Normal::new(
                f32::from(def.attributes.0.immunity),
                f32::from(def.attributes.1.immunity),
            )
            .unwrap()
            .sample(rng) as u16,
            healing: Normal::new(
                f32::from(def.attributes.0.healing),
                f32::from(def.attributes.1.healing),
            )
            .unwrap()
            .sample(rng) as u16,
            analytical: Normal::new(
                f32::from(def.attributes.0.analytical),
                f32::from(def.attributes.1.analytical),
            )
            .unwrap()
            .sample(rng) as u16,
            focus: Normal::new(
                f32::from(def.attributes.0.focus),
                f32::from(def.attributes.1.focus),
            )
            .unwrap()
            .sample(rng) as u16,
            willpower: Normal::new(
                f32::from(def.attributes.0.willpower),
                f32::from(def.attributes.1.willpower),
            )
            .unwrap()
            .sample(rng) as u16,
            creativity: Normal::new(
                f32::from(def.attributes.0.creativity),
                f32::from(def.attributes.1.creativity),
            )
            .unwrap()
            .sample(rng) as u16,
            intuition: Normal::new(
                f32::from(def.attributes.0.intuition),
                f32::from(def.attributes.1.intuition),
            )
            .unwrap()
            .sample(rng) as u16,
            patience: Normal::new(
                f32::from(def.attributes.0.patience),
                f32::from(def.attributes.1.patience),
            )
            .unwrap()
            .sample(rng) as u16,
            memory: Normal::new(
                f32::from(def.attributes.0.memory),
                f32::from(def.attributes.1.memory),
            )
            .unwrap()
            .sample(rng) as u16,
            linguistic: Normal::new(
                f32::from(def.attributes.0.linguistic),
                f32::from(def.attributes.1.linguistic),
            )
            .unwrap()
            .sample(rng) as u16,
            spatial: Normal::new(
                f32::from(def.attributes.0.spatial),
                f32::from(def.attributes.1.spatial),
            )
            .unwrap()
            .sample(rng) as u16,
            kinesthetic: Normal::new(
                f32::from(def.attributes.0.kinesthetic),
                f32::from(def.attributes.1.kinesthetic),
            )
            .unwrap()
            .sample(rng) as u16,
            empathy: Normal::new(
                f32::from(def.attributes.0.empathy),
                f32::from(def.attributes.1.empathy),
            )
            .unwrap()
            .sample(rng) as u16,
            social: Normal::new(
                f32::from(def.attributes.0.social),
                f32::from(def.attributes.1.social),
            )
            .unwrap()
            .sample(rng) as u16,
        }
    }

    pub fn default_deviation() -> Self {
        Self {
            strength: 200,
            agility: 200,
            toughness: 200,
            endurance: 200,
            immunity: 200,
            healing: 200,
            analytical: 200,
            focus: 200,
            willpower: 200,
            creativity: 200,
            intuition: 200,
            patience: 200,
            memory: 200,
            linguistic: 200,
            spatial: 200,
            kinesthetic: 200,
            empathy: 200,
            social: 200,
        }
    }

    pub fn default_with_deviation() -> (Self, Self) {
        (Self::default(), Self::default_deviation())
    }
}
impl Default for Attributes {
    fn default() -> Self {
        Self {
            strength: 1000,
            agility: 1000,
            toughness: 1000,
            endurance: 1000,
            immunity: 1000,
            healing: 1000,
            analytical: 1000,
            focus: 1000,
            willpower: 1000,
            creativity: 1000,
            intuition: 1000,
            patience: 1000,
            memory: 1000,
            linguistic: 1000,
            spatial: 1000,
            kinesthetic: 1000,
            empathy: 1000,
            social: 1000,
        }
    }
}

#[derive(NamedDefinition, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RaceDefinition {
    name: String,

    #[serde(skip)]
    id: Option<u32>,
    pub body: String,

    #[serde(default)]
    pub properties: Vec<Property>,

    #[serde(default)]
    pub psyche: Vec<(PsycheTraitRef, f32)>,

    #[serde(default = "Attributes::default_with_deviation")]
    pub attributes: (Attributes, Attributes), //base, deviation

    #[serde(default)]
    pub needs: NeedsContainer,
}
impl HasProperties for RaceDefinition {
    fn default_properties(&self) -> PropertiesComponent {
        PropertiesComponent::from_iter_ref(self.properties.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::init_test_log;

    #[test]
    pub fn serialize_race() -> Result<(), failure::Error> {
        let humans = RaceDefinition {
            name: "Human".to_string(),
            id: None,
            body: "humanoid".to_string(),
            properties: Vec::new(),
            psyche: Vec::new(),
            attributes: Attributes::default_with_deviation(),
            needs: NeedsContainer::default(),
        };

        println!(
            "{}",
            ron::ser::to_string_pretty(
                &humans,
                ron::ser::PrettyConfig::new()
                    .with_depth_limit(10)
                    .with_separate_tuple_members(false)
                    .with_enumerate_arrays(false)
                    .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME),
            )?
        );

        Ok(())
    }
    #[test]
    pub fn gen_race_attributes() {
        use rand::SeedableRng;

        init_test_log();

        let file = std::fs::File::open("../resources/defs/races.ron").unwrap();
        let races: Vec<RaceDefinition> = crate::ron::de::from_reader(&file).unwrap();

        let mut rng = crate::rand_xorshift::XorShiftRng::from_seed([
            1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4,
        ]);

        for _ in 0..100 {
            let attributes = Attributes::generate(&mut rng, &races[0]);
            log::trace!("{:?}", attributes);
        }
    }
}
