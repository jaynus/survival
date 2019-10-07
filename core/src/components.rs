pub use crate::defs::{
    building::BuildingDefinition,
    digestion::{DigestionDefinition, EdibleKind, EdibleState},
    foliage::FoliageDefinition,
    item::{ItemDefinition, ItemPart},
    material::*,
    property::{Dimensions, Property, PropertyCategory, PropertyKind},
    psyche::NeedState,
    race::{Attributes, RaceDefinition},
    Definition, DefinitionComponent, DefinitionStorage,
};
use amethyst::{
    core::{
        components::Transform,
        ecs::{Component, Entity, FlaggedStorage, ReadStorage, VecStorage},
        math::{Point3, Vector3},
        Time,
    },
    tiles::iters::Region,
};
use fnv::FnvHashMap;
use shrinkwraprs::Shrinkwrap;
use specs_hierarchy::Parent;
use std::{iter::FromIterator, time::Duration};
use strum::EnumProperty;
use survival_derive::DefinitionComponent;

pub type AllComponents<'a> = (
    ReadStorage<'a, ItemComponent>,
    ReadStorage<'a, ItemParentComponent>,
    ReadStorage<'a, StaticTransform>,
    ReadStorage<'a, PawnComponent>,
    ReadStorage<'a, IdleComponent>,
    ReadStorage<'a, RaceComponent>,
    ReadStorage<'a, PropertiesComponent>,
    ReadStorage<'a, BuildingComponent>,
    ReadStorage<'a, ItemComponent>,
    ReadStorage<'a, FoliageComponent>,
);

#[derive(Default)]
pub struct StaticTransform(pub Transform);
impl Component for StaticTransform {
    type Storage = VecStorage<Self>;
}

pub struct PawnComponent {
    pub name: String,
}
impl Component for PawnComponent {
    type Storage = VecStorage<Self>;
}
impl Default for PawnComponent {
    fn default() -> Self {
        Self {
            name: "asdf".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IdleComponent {
    started: Duration,
}
impl IdleComponent {
    pub fn new(now: &Time) -> Self {
        Self {
            started: now.absolute_time(),
        }
    }

    pub fn duration(&self, now: &Time) -> Duration {
        self.duration_since(now.absolute_time())
    }

    pub fn duration_since(&self, now: Duration) -> Duration {
        now - self.started
    }

    pub fn set(&mut self, now: Duration) {
        self.started = now;
    }

    pub fn reset(&mut self, now: &Time) {
        self.started = now.absolute_time()
    }
}
impl Component for IdleComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default)]
pub struct ItemPartState {
    pub name: String,
    pub state: u8,
    pub material: MaterialRef,
}

#[derive(DefinitionComponent, Debug, Default)]
#[def(ItemDefinition)]
pub struct ItemComponent {
    pub def: u32,
    pub parts: Vec<ItemPartState>,
}
impl ItemComponent {
    pub fn new(
        def: u32,
        material: &MaterialRef,
        def_storage: &DefinitionStorage<ItemDefinition>,
    ) -> Self {
        Self {
            def,
            parts: Self::create_parts(material, def_storage.get(def).unwrap()),
        }
    }

    fn create_parts(material: &MaterialRef, def: &ItemDefinition) -> Vec<ItemPartState> {
        def.parts
            .iter()
            .map(|(part, _)| ItemPartState {
                name: part.name.clone(),
                state: 255,
                material: material.clone(),
            })
            .collect()
    }
}
impl Component for ItemComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Debug)]
pub enum ItemParentRelationship {
    Inside,
    On,
    Worn,
}

#[derive(Debug)]
pub struct ItemParentComponent {
    pub parent: Entity,
    pub relationship: ItemParentRelationship,
}
impl ItemParentComponent {
    pub fn new(parent: Entity, relationship: ItemParentRelationship) -> Self {
        Self {
            parent,
            relationship,
        }
    }
}

impl Component for ItemParentComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}
impl Parent for ItemParentComponent {
    fn parent_entity(&self) -> Entity {
        self.parent
    }
}

#[derive(DefinitionComponent, Default, Debug, Clone, Copy)]
#[def(FoliageDefinition)]
pub struct FoliageComponent {
    pub def: u32,
}
impl FoliageComponent {
    pub fn new(id: u32, _: &DefinitionStorage<FoliageDefinition>) -> Self {
        Self { def: id }
    }
}
impl Component for FoliageComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(DefinitionComponent, Default, Debug, Clone, Copy)]
#[def(BuildingDefinition)]
pub struct BuildingComponent {
    pub def: u32,
}
impl BuildingComponent {
    pub fn new(id: u32, _: &DefinitionStorage<BuildingDefinition>) -> Self {
        Self { def: id }
    }
}
impl Component for BuildingComponent {
    type Storage = VecStorage<Self>;
}

pub enum PropertiesMergeResolution {
    Overwrite,
    Keep,
    Error,
}

#[derive(Default, Debug, Clone)]
pub struct PropertiesComponent {
    inner: FnvHashMap<PropertyKind, Property>,
}
impl PropertiesComponent {
    pub fn from_iter_ref<'a, I: Iterator<Item = &'a Property>>(iter: I) -> Self {
        let mut inner = FnvHashMap::default();
        iter.for_each(|p| {
            assert!(inner.insert(p.clone().into(), p.clone()).is_none());
        });
        Self { inner }
    }

    pub fn get(&self, prop: PropertyKind) -> Option<&Property> {
        self.inner.get(&prop)
    }

    pub fn insert(&mut self, prop: Property) -> Option<Property> {
        self.inner.insert(prop.into(), prop)
    }

    pub fn contains(&self, property: PropertyKind) -> bool {
        self.inner.contains_key(&property)
    }
    pub fn contains_value(&self, property: &Property) -> bool {
        // TODO: Ord compare
        self.inner.contains_key(&(*property).into())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Property)> {
        self.inner.iter().map(|(_, p)| p)
    }

    pub fn clear_category(&mut self, category: PropertyCategory) {
        self.inner
            .retain(|_, p| p.get_str("Category").unwrap() != category.as_ref());
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn merge(
        mut self,
        resolution: PropertiesMergeResolution,
        other: &PropertiesComponent,
    ) -> Self {
        other.iter().for_each(|prop| {
            let key: PropertyKind = prop.into();

            match resolution {
                PropertiesMergeResolution::Error => {
                    if self.inner.contains_key(&key) {
                        panic!("Duplicate key with merge resolution of error")
                    }
                }
                PropertiesMergeResolution::Keep => {
                    if self.inner.contains_key(&key) {
                        return;
                    }
                }
                _ => {}
            }
            self.inner.insert(key, *prop);
        });
        self
    }
}

impl Extend<Property> for PropertiesComponent {
    fn extend<T: IntoIterator<Item = Property>>(&mut self, iter: T) {
        iter.into_iter().for_each(|p| {
            assert!(self.inner.insert(p.into(), p).is_none());
        });
    }
}
impl<'a> Extend<&'a Property> for PropertiesComponent {
    fn extend<T: IntoIterator<Item = &'a Property>>(&mut self, iter: T) {
        iter.into_iter().for_each(|p| {
            assert!(self.inner.insert(p.clone().into(), p.clone()).is_none());
        });
    }
}

impl FromIterator<Property> for PropertiesComponent {
    fn from_iter<I: IntoIterator<Item = Property>>(iter: I) -> Self {
        let mut inner = FnvHashMap::default();
        iter.into_iter().for_each(|p| {
            let t: PropertyKind = p.into();
            assert!(inner.insert(t, p).is_none());
        });

        Self { inner }
    }
}

impl From<&[Property]> for PropertiesComponent {
    fn from(slice: &[Property]) -> Self {
        PropertiesComponent::from_iter_ref(slice.iter())
    }
}

impl Component for PropertiesComponent {
    type Storage = VecStorage<Self>;
}

#[derive(DefinitionComponent, Debug, Default)]
#[def(RaceDefinition)]
pub struct RaceComponent {
    pub def: u32,
}
impl RaceComponent {
    pub fn new(def: u32) -> Self {
        Self { def }
    }
}
impl Component for RaceComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Shrinkwrap, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct AttributesComponent(Attributes);
impl AttributesComponent {
    pub fn new(inner: Attributes) -> Self {
        Self(inner)
    }
}
impl Component for AttributesComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct EdibleComponent {
    state: EdibleState,
    kind: EdibleKind,
    calories: Option<u32>,
    hydration: Option<u32>,
}
impl EdibleComponent {
    pub fn new(kind: EdibleKind) -> Self {
        Self {
            state: EdibleState::default(),
            kind,
            calories: None,
            hydration: None,
        }
    }

    pub fn with_state(mut self, state: EdibleState) -> Self {
        self.state = state;
        self
    }

    pub fn with_calories(mut self, calories: u32) -> Self {
        self.calories = Some(calories);
        self
    }

    pub fn with_hydration(mut self, hydration: u32) -> Self {
        self.hydration = Some(hydration);
        self
    }
}
impl Component for EdibleComponent {
    type Storage = VecStorage<Self>;
}

#[derive(DefinitionComponent, Default, Debug, Clone, Copy)]
#[def(DigestionDefinition)]
pub struct DigestionComponent {
    pub def: u32,

    calories: NeedState,
    hydration: NeedState,
}
impl DigestionComponent {
    pub fn new(id: u32, _: &DefinitionStorage<DigestionDefinition>) -> Self {
        Self {
            def: id,
            calories: NeedState::default(),
            hydration: NeedState::default(),
        }
    }
}
impl Component for DigestionComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PawnType {
    Player,
    AI,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TypeTagComponent {
    Building,
    Foliage,
    Item,
    Pawn(PawnType),
}
impl Component for TypeTagComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SpatialComponent {
    pub dimensions: Dimensions,
    pub mass: u64,
}
impl SpatialComponent {
    pub fn new(dimensions: Dimensions, mass: u64) -> Self {
        Self { dimensions, mass }
    }

    pub fn occupies_tiles(&self, position: &Point3<u32>) -> Region {
        let size = self.tile_size();
        Region::new(*position, *position + size)
    }

    pub fn tile_size(&self) -> Vector3<u32> {
        match self.dimensions {
            Dimensions::Cube { x, y, z } => Vector3::new(
                (x / crate::tiles::TILE_SCALE).max(1) as u32,
                (y / crate::tiles::TILE_SCALE).max(1) as u32,
                (z / crate::tiles::TILE_SCALE).max(1) as u32,
            ),
            _ => unimplemented!(),
        }
    }
}
impl Component for SpatialComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Copy)]
pub struct TilePosition(pub Point3<u32>);
impl Component for TilePosition {
    type Storage = VecStorage<Self>;
}
impl Default for TilePosition {
    fn default() -> Self {
        Self(Point3::new(0, 0, 0))
    }
}
