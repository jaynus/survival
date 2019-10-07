use crate::defs::{
    material::{MaterialRef, MaterialState},
    property::Property,
    Definition, Named,
};
use strum_macros::{AsRefStr, EnumDiscriminants};

use survival_derive::NamedDefinition;

#[derive(Debug, Clone, EnumDiscriminants, serde::Serialize, serde::Deserialize)]
#[strum_discriminants(name(KindType))]
#[strum_discriminants(derive(Hash, AsRefStr))]
pub enum Kind {
    Item(String),
    Properties(Vec<Property>),
    Location {
        name: String,
        #[serde(default)]
        distance: u8,
        #[serde(default)]
        level: u8,
    },
    Skill {
        name: String,
        #[serde(default)]
        level: u8,
    },
    Invalid,
}
impl Default for Kind {
    fn default() -> Self {
        Kind::Invalid
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Material {
    Any(MaterialState),
    Source, // Used for product reagents
    Material(MaterialRef),
}
impl Default for Material {
    fn default() -> Self {
        Material::Any(MaterialState::Solid)
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Reagent {
    pub kind: Kind,
    #[serde(default = "Reagent::default_consume")]
    pub consume: bool,
    #[serde(default = "Reagent::default_materials")]
    pub materials: Vec<Material>,
    #[serde(default = "Reagent::default_count")]
    pub count: usize,
}
impl Reagent {
    fn default_materials() -> Vec<Material> {
        vec![Material::Any(MaterialState::Solid)]
    }
    const fn default_count() -> usize {
        1
    }
    const fn default_consume() -> bool {
        false
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Product {
    pub kind: Kind,
    #[serde(default = "Product::default_material")]
    pub material: Material,
    #[serde(default = "Product::default_count")]
    pub count: usize,
}
impl Product {
    const fn default_material() -> Material {
        Material::Source
    }
    const fn default_count() -> usize {
        1
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReactionDuration {
    interaction: u64,
    delay: u64,
    skill_weight: u64,
}

#[derive(NamedDefinition, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReactionDefinition {
    name: String,
    #[serde(skip)]
    id: Option<u32>,
    pub category: crate::fsm::TaskCategory,
    pub duration: ReactionDuration,
    pub reagents: Vec<Reagent>,
    pub product: Product,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::init_test_log;

    #[test]
    fn reaction_deserialized() -> Result<(), failure::Error> {
        init_test_log();
        let f = std::fs::File::open("/home/jaynus/dev/survival/resources/defs/reactions.ron")?;
        let reactions: Vec<ReactionDefinition> = ron::de::from_reader(f)?;

        log::trace!("{:?}", reactions);

        Ok(())
    }

    #[test]
    fn reaction_serialized() {
        init_test_log();

        use crate::defs::material::{MaterialRef, MaterialState};

        let mut r = ReactionDefinition::default();
        r.name = "reaction name".to_string();
        r.reagents = vec![
            Reagent {
                kind: Kind::Item("reagent item name".to_string()),
                materials: vec![Material::Material(MaterialRef::new(
                    "marble",
                    MaterialState::Solid,
                ))],
                consume: false,
                count: 1,
            },
            Reagent {
                kind: Kind::Location {
                    name: "reagent station name".to_string(),
                    distance: 1,
                    level: 1,
                },
                consume: false,
                materials: Reagent::default_materials(),
                count: 1,
            },
            Reagent {
                kind: Kind::Skill {
                    name: "reagent skill name".to_string(),
                    level: 1,
                },
                consume: false,
                materials: Reagent::default_materials(),
                count: 1,
            },
        ];
        r.product = Product {
            kind: Kind::Item("product item name".to_string()),
            material: Material::Source,
            count: 1,
        };

        let reactions = vec![r];

        let serialized = ron::ser::to_string_pretty(
            &reactions,
            ron::ser::PrettyConfig::new()
                .with_depth_limit(10)
                .with_separate_tuple_members(false)
                .with_enumerate_arrays(false)
                .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME),
        )
        .unwrap();
        log::trace!("{}", serialized);
    }
}
