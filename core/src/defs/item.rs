use crate::{
    components::PropertiesComponent,
    defs::{
        material::{MaterialCategory, MaterialState},
        property::{Dimensions, Property},
        sprites::SpriteRef,
        Definition, HasProperties, Named,
    },
};
use survival_derive::NamedDefinition;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ItemCategory {
    Weapon,
    Tool,
    Block,
    Organic,
    Unspecified,
}
impl Default for ItemCategory {
    fn default() -> Self {
        ItemCategory::Unspecified
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ItemPart {
    pub name: String,
    limit_materials: Vec<(Option<MaterialCategory>, MaterialState)>,
}

#[derive(NamedDefinition, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ItemDefinition {
    name: String,
    #[serde(skip)]
    id: Option<u32>,

    pub category: ItemCategory,

    pub dimensions: Option<Dimensions>,

    #[serde(default)]
    pub properties: Vec<Property>,

    pub sprite: SpriteRef,

    #[serde(default = "default_part")]
    pub parts: Vec<(ItemPart, u32)>,
}

pub fn default_part() -> Vec<(ItemPart, u32)> {
    vec![(
        ItemPart {
            name: "Default".to_string(),
            limit_materials: vec![(None, MaterialState::Solid)],
        },
        100,
    )]
}

impl HasProperties for ItemDefinition {
    fn default_properties(&self) -> PropertiesComponent {
        PropertiesComponent::from_iter_ref(self.properties.iter())
    }
}
