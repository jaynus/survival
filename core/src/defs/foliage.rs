use crate::{
    components::PropertiesComponent,
    defs::{
        material::MaterialLayerRef,
        property::{Dimensions, Property},
        sprites::SpriteRef,
        Definition, HasProperties, Named,
    },
};
use survival_derive::NamedDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum FoliageCategory {
    Tree,
    Brush,
}

#[derive(NamedDefinition, Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FoliageDefinition {
    name: String,

    #[serde(skip)]
    id: Option<u32>,

    pub category: FoliageCategory,

    #[serde(default)]
    pub sprite: SpriteRef,

    pub base_dimensions: Dimensions, //cm3, x,y,z

    #[serde(default)]
    pub variance_dimensions: Dimensions, //cm3, x,y,z

    pub material_layers: Vec<MaterialLayerRef>,

    #[serde(default)]
    pub properties: Vec<Property>,
}

impl HasProperties for FoliageDefinition {
    fn default_properties(&self) -> PropertiesComponent {
        let mut component = PropertiesComponent::from_iter_ref(self.properties.iter());
        component.insert(Property::Foliage(self.category));

        component
    }
}
