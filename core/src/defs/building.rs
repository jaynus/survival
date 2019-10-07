use crate::bitflags_serial;
use crate::{
    components::PropertiesComponent,
    defs::{
        property::{Dimensions, Property},
        sprites::SpriteRef,
        Definition, HasProperties, Named,
    },
};
use bitflags::*;
use survival_derive::NamedDefinition;

//use ai::Condition;

bitflags_serial! {
    pub struct BuildingFlags: u32 {
        const Water = 1;
        const Magma = 1 << 1;
        const Land  = 1 << 2;
    }
}

#[derive(
    NamedDefinition, Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct BuildingDefinition {
    name: String,
    #[serde(skip)]
    id: Option<u32>,

    #[serde(default)]
    pub flags: BuildingFlags,

    pub sprite: SpriteRef,

    pub dimensions: Dimensions, //cm3, x,y,z

    #[serde(default)]
    pub properties: Vec<Property>,
}

impl HasProperties for BuildingDefinition {
    fn default_properties(&self) -> PropertiesComponent {
        let mut ret = PropertiesComponent::from_iter_ref(self.properties.iter());
        ret.insert(Property::Building);
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::init_test_log;

    #[test]
    fn building_serialize() {
        init_test_log();

        let b = BuildingDefinition::default();

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
    fn reaction_deserialized() -> Result<(), failure::Error> {
        init_test_log();
        let f = std::fs::File::open("/home/jaynus/dev/survival/resources/defs/buildings.ron")?;
        let buildings: Vec<BuildingDefinition> = ron::de::from_reader(f)?;

        log::trace!("{:?}", buildings);

        Ok(())
    }
}
