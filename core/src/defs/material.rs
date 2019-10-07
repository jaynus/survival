use super::{Definition, InheritDefinition, Named};
use crate::{defs::DefinitionStorage, strum_macros::AsRefStr};
use survival_derive::NamedDefinition;

#[derive(
    AsRefStr,
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(u8)]
pub enum MaterialState {
    Solid,
    Powder,
    Paste,
    Liquid,
    Frozen,
    Gas,
}

impl Default for MaterialState {
    fn default() -> Self {
        MaterialState::Solid
    }
}

#[derive(
    AsRefStr,
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum RockSubCategory {
    Igneous,
    Metamorphic,
    Sedimentary,
}
impl Default for RockSubCategory {
    fn default() -> Self {
        RockSubCategory::Sedimentary
    }
}

#[derive(
    AsRefStr,
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum MaterialCategory {
    Rock { subcategory: RockSubCategory },
    Todo,
}
impl Default for MaterialCategory {
    fn default() -> Self {
        MaterialCategory::Todo
    }
}

#[derive(
    NamedDefinition,
    Debug,
    Clone,
    Default,
    Hash,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct MaterialStateDefinition {
    name: String,
    #[serde(skip)]
    id: Option<u32>,

    #[serde(default)]
    inherits: Option<String>,

    #[serde(default)]
    pub density: Option<u32>,
    #[serde(default)]
    pub hardness: Option<u32>,
    #[serde(default)]
    pub porosity: Option<u32>,
    #[serde(default)]
    pub permeability: Option<u32>,
    #[serde(default)]
    pub elasticity: Option<u32>,
    #[serde(default)]
    pub tensile_strength: Option<u32>,
    #[serde(default)]
    pub tensile_yield: Option<u32>,
    #[serde(default)]
    pub compressive_yield_strength: Option<u32>,
    #[serde(default)]
    pub fatigue_strength: Option<u32>,
    #[serde(default)]
    pub fracture_toughness: Option<u32>,

    #[serde(default)]
    pub flexural_strength: Option<u32>,
    #[serde(default)]
    pub shear_modulus: Option<u32>,
    #[serde(default)]
    pub poisson_ratio: Option<u32>,
    #[serde(default)]
    pub impact_toughness: Option<u32>,
    #[serde(default)]
    pub electric_resistance: Option<u32>,
    #[serde(default)]
    pub specific_heat_capacity: Option<u32>,
    #[serde(default)]
    pub thermal_conductivity: Option<u32>,
    #[serde(default)]
    pub abrasive_hardness: Option<u32>,

    pub sprite: (String, u32),
}

impl InheritDefinition for MaterialStateDefinition {
    fn inherit_from(&mut self, parent: &Self) {
        self.density = self.density.map_or(parent.density, Some);
        self.hardness = self.hardness.map_or(parent.hardness, Some);
        self.porosity = self.porosity.map_or(parent.porosity, Some);
        self.permeability = self.permeability.map_or(parent.permeability, Some);
        self.elasticity = self.elasticity.map_or(parent.elasticity, Some);
        self.tensile_strength = self.tensile_strength.map_or(parent.tensile_strength, Some);
        self.tensile_yield = self.tensile_yield.map_or(parent.tensile_yield, Some);
        self.compressive_yield_strength = self
            .compressive_yield_strength
            .map_or(parent.compressive_yield_strength, Some);
        self.fatigue_strength = self.fatigue_strength.map_or(parent.fatigue_strength, Some);
        self.fracture_toughness = self
            .fracture_toughness
            .map_or(parent.fracture_toughness, Some);
        self.flexural_strength = self
            .flexural_strength
            .map_or(parent.flexural_strength, Some);
        self.shear_modulus = self.shear_modulus.map_or(parent.abrasive_hardness, Some);
        self.poisson_ratio = self.poisson_ratio.map_or(parent.poisson_ratio, Some);
        self.impact_toughness = self.impact_toughness.map_or(parent.impact_toughness, Some);
        self.electric_resistance = self
            .electric_resistance
            .map_or(parent.electric_resistance, Some);
        self.specific_heat_capacity = self
            .specific_heat_capacity
            .map_or(parent.specific_heat_capacity, Some);
        self.thermal_conductivity = self
            .thermal_conductivity
            .map_or(parent.thermal_conductivity, Some);
        self.abrasive_hardness = self
            .abrasive_hardness
            .map_or(parent.abrasive_hardness, Some);
    }

    fn parent(&self) -> Option<&str> {
        self.inherits.as_ref().map(String::as_str)
    }
}

#[derive(NamedDefinition, Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct MaterialDefinition {
    name: String,
    #[serde(skip)]
    id: Option<u32>,

    pub inherits: Option<String>,

    pub category: MaterialCategory,

    #[serde(default)]
    pub states: fnv::FnvHashMap<MaterialState, MaterialStateDefinition>,

    #[serde(default)]
    pub melt_point: Option<u64>,

    #[serde(default)]
    pub boil_point: Option<u64>,

    #[serde(default)]
    pub ignite_point: Option<u64>,

    #[serde(default)]
    pub freeze_point: Option<u64>,
}

impl PartialEq for MaterialDefinition {
    fn eq(&self, other: &Self) -> bool {
        if self.name == other.name
            && self.inherits == other.inherits
            && self.category == other.category
            && self.melt_point == other.melt_point
            && self.boil_point == other.boil_point
            && self.ignite_point == other.ignite_point
            && self.freeze_point == other.freeze_point
        {
            for (self_k, self_v) in &self.states {
                if let Some(v) = other.states.get(self_k) {
                    if v != self_v {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            return true;
        }

        false
    }
}
impl Eq for MaterialDefinition {}
impl std::hash::Hash for MaterialDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.inherits.hash(state);
        self.category.hash(state);
        self.melt_point.hash(state);
        self.boil_point.hash(state);
        self.ignite_point.hash(state);
        self.freeze_point.hash(state);

        self.states.iter().for_each(|(k, v)| {
            k.hash(state);
            v.hash(state);
        });
    }
}

impl InheritDefinition for MaterialDefinition {
    fn inherit_from(&mut self, parent: &Self) {
        self.freeze_point = self.freeze_point.map_or(parent.freeze_point, Some);
        self.melt_point = self.melt_point.map_or(parent.melt_point, Some);
        self.boil_point = self.boil_point.map_or(parent.boil_point, Some);
        self.ignite_point = self.ignite_point.map_or(parent.ignite_point, Some);

        parent.states.iter().for_each(|(k, v)| {
            if !self.states.contains_key(k) {
                self.states.insert(*k, v.clone());
            }
        });
    }

    fn parent(&self) -> Option<&str> {
        self.inherits.as_ref().map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MaterialRef {
    pub name: String,
    pub state: MaterialState,
}
impl MaterialRef {
    pub fn new(material_name: &str, state: MaterialState) -> Self {
        Self {
            name: material_name.to_string(),
            state,
        }
    }
}
impl Default for MaterialRef {
    fn default() -> Self {
        Self {
            name: "marble".to_string(),
            state: MaterialState::default(),
        }
    }
}
impl Named for MaterialRef {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MaterialLayerRefCompact {
    pub material_id: u32,
    pub value: u32,
    pub state: MaterialState,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MaterialLayerRef {
    pub name: Option<String>,
    pub value: u32,
    pub material: MaterialRef,
}
impl Default for MaterialLayerRef {
    fn default() -> Self {
        Self {
            name: None,
            value: 0,
            material: MaterialRef::default(),
        }
    }
}
impl MaterialLayerRef {
    pub fn new(layer_name: &str, material_name: &str, state: MaterialState, value: u32) -> Self {
        Self {
            name: Some(layer_name.to_string()),
            material: MaterialRef::new(material_name, state),
            value,
        }
    }

    pub fn to_compact(
        &self,
        defs: &DefinitionStorage<MaterialDefinition>,
    ) -> MaterialLayerRefCompact {
        MaterialLayerRefCompact {
            material_id: defs
                .find(&self.material.name)
                .unwrap_or_else(|| panic!("Invalid material name: {}", self.material.name))
                .id()
                .unwrap(),
            state: self.material.state,
            value: self.value,
        }
    }
}
