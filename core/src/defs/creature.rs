use crate::{
    components::PropertiesComponent,
    defs::{property::Property, sprites::SpriteRef, Definition, HasProperties, Named},
};
use survival_derive::NamedDefinition;

#[derive(NamedDefinition, Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CreatureDefinition {
    name: String,

    #[serde(skip)]
    id: Option<u32>,

    pub sprite: Option<SpriteRef>,

    pub body: Option<String>,

    pub behavior: Option<String>,

    pub properties: Vec<Property>,
}
impl HasProperties for CreatureDefinition {
    fn default_properties(&self) -> PropertiesComponent {
        PropertiesComponent::from_iter_ref(self.properties.iter())
    }
}
