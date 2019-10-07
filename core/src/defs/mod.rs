pub mod action;
pub mod body;
pub mod building;
pub mod creature;
pub mod digestion;
pub mod foliage;
pub mod item;
pub mod material;
pub mod property;
pub mod psyche;
pub mod race;
pub mod reaction;
pub mod sprites;

use hibitset::BitSet;
use std::{
    collections::HashMap,
    fs::metadata,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub trait Named {
    fn name(&self) -> &str;
    fn id(&self) -> Option<u32> { None }
    fn set_id(&mut self, _id: u32) {}
}

pub trait Definition: Named {}

pub trait DefinitionComponent {
    type DefinitionType: Definition + Clone + std::fmt::Debug;
    fn fetch_def<'a>(
        &self,
        storage: &'a DefinitionStorage<Self::DefinitionType>,
    ) -> Option<&'a Self::DefinitionType>;
}

pub trait InheritDefinition {
    fn inherit_from(&mut self, parent: &Self);

    fn parent(&self) -> Option<&str>;
    fn has_parent(&self) -> bool { self.parent().is_some() }
}

pub trait InheritDefinitionStorage {
    fn apply_inherits(&mut self) -> Result<(), failure::Error>;
}

pub struct DefinitionLookup<'a, T>
where
    T: std::fmt::Debug + Clone + Definition + for<'b> serde::Deserialize<'b>,
{
    pub storage: &'a DefinitionStorage<T>,
}
impl<'a, T> DefinitionLookup<'a, T>
where
    T: std::fmt::Debug + Clone + Definition + for<'b> serde::Deserialize<'b>,
{
    pub fn new(storage: &'a DefinitionStorage<T>) -> Self { Self { storage } }
}
impl<'a, T> DefinitionLookup<'a, T>
where
    T: std::fmt::Debug + Clone + Definition + for<'b> serde::Deserialize<'b>,
{
    pub fn find(&self, name: &str) -> Option<&T> { self.get(self.storage.get_id(name)?) }

    pub fn get(&self, id: u32) -> Option<&T> { self.storage.get(id) }
}

#[derive(Clone, Default, Debug)]
pub struct DefinitionStorage<T: Clone + std::fmt::Debug, D = T> {
    bitset: BitSet,
    lookup: HashMap<String, u32>,
    storage: Vec<D>,
    source: PathBuf,
    _marker: std::marker::PhantomData<(T, D)>,
}

impl<T> DefinitionStorage<T>
where
    T: std::fmt::Debug + Clone + Definition + for<'a> serde::Deserialize<'a>,
{
    pub fn find(&self, name: &str) -> Option<&T> { self.get(self.get_id(name)?) }
    pub fn find_mut(&mut self, name: &str) -> Option<&mut T> { self.get_mut(self.get_id(name)?) }

    pub fn get(&self, id: u32) -> Option<&T> { self.storage.get(id as usize) }
    pub fn get_mut(&mut self, id: u32) -> Option<&mut T> { self.storage.get_mut(id as usize) }

    pub fn get_id(&self, name: &str) -> Option<u32> {
        self.lookup.get(&name.to_lowercase()).copied()
    }

    pub fn raw_storage(&self) -> &Vec<T> { &self.storage }

    pub fn keys(&self) -> impl Iterator<Item = &String> { self.lookup.keys() }

    pub fn iter(&self) -> impl Iterator<Item = &T> { self.storage.iter() }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> { self.storage.iter_mut() }

    pub fn has_key(&self, name: &str) -> bool { self.lookup.get(name).is_some() }

    pub fn has_id(&self, id: u32) -> bool { self.storage.len() >= id as usize }

    pub fn bitset(&self) -> &BitSet { &self.bitset }

    pub fn len(&self) -> usize { self.storage.len() }
    pub fn is_empty(&self) -> bool { self.len() < 1 }

    pub fn reload(&mut self) -> Result<(), failure::Error> {
        self.storage.clear();
        self.lookup.clear();

        self.internal_from_folder(self.source.clone())?;

        Ok(())
    }

    fn internal_from_folder<P>(&mut self, folder: P) -> Result<(), failure::Error>
    where
        P: AsRef<Path>,
    {
        log::trace!(
            "Loading definitions from path: cwd={}, path={}",
            std::env::current_dir().unwrap().display(),
            folder.as_ref().display()
        );
        // Collect the files and try the root name as well.
        let mut files = Vec::new();

        // Add the root entry
        let root_file = folder.as_ref().with_extension("ron");
        if let Ok(meta) = metadata(&root_file) {
            if meta.file_type().is_file() {
                files.push(root_file);
            }
        }

        for entry in WalkDir::new(folder.as_ref())
            .into_iter()
            .filter_map(Result::ok)
        {
            if let Ok(meta) = entry.metadata() {
                if meta.file_type().is_file() {
                    files.push(entry.into_path());
                }
            }
        }

        files.sort();

        for entry in files {
            log::trace!("Attempting definitions from: {:?}", entry);

            let file = std::fs::OpenOptions::new().read(true).open(entry)?;
            let def_entries = ron::de::from_reader::<std::fs::File, Vec<T>>(file)?;

            self.storage.reserve(self.storage.len() + def_entries.len());
            self.lookup.reserve(self.lookup.len() + def_entries.len());

            for def in &def_entries {
                self.insert(def.clone());
            }
        }

        Ok(())
    }

    pub fn insert(&mut self, mut def: T) {
        def.set_id(self.storage.len() as u32);
        self.bitset.add(def.id().unwrap());

        self.lookup
            .insert(def.name().to_lowercase().to_string(), def.id().unwrap());
        log::trace!("Inserted '{}'", def.name().to_lowercase());
        self.storage.push(def);
    }

    pub fn from_folder<P>(folder: P) -> Result<Self, failure::Error>
    where
        P: AsRef<Path>,
    {
        let mut s = Self::new(&folder);
        s.internal_from_folder(folder)?;
        Ok(s)
    }

    pub fn new<P>(folder: &P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            bitset: BitSet::new(),
            source: folder.as_ref().to_path_buf(),
            lookup: HashMap::new(),
            storage: Vec::new(),
            _marker: Default::default(),
        }
    }
}

impl<T> InheritDefinitionStorage for DefinitionStorage<T>
where
    T: std::fmt::Debug + InheritDefinition + Definition + for<'a> serde::Deserialize<'a> + Clone,
{
    fn apply_inherits(&mut self) -> Result<(), failure::Error> {
        let mut parents = Vec::new();

        for item in self.iter() {
            if let Some(parent_name) = item.parent() {
                parents.push((item.id().unwrap(), parent_name.to_string()));
            }
        }

        for (index, parent) in &parents {
            let parent = { self.iter().find(|v| v.name() == *parent).unwrap().clone() };
            if let Some(item) = self.get_mut(*index as u32) {
                item.inherit_from(&parent);
            }
        }

        Ok(())
    }
}

pub trait HasProperties {
    fn default_properties(&self) -> crate::components::PropertiesComponent;
}

#[cfg(test)]
mod tests {
    use super::material::MaterialDefinition;
    use super::*;

    #[test]
    fn material_definitions() -> Result<(), failure::Error> {
        let storage = DefinitionStorage::<MaterialDefinition>::from_folder(Path::new(
            "../resources/defs/materials",
        ))?;

        assert!(storage.keys().count() > 0);

        Ok(())
    }

    #[test]
    fn load_all_defs() -> Result<(), failure::Error> {
        let mut storage =
            DefinitionStorage::<MaterialDefinition>::from_folder("../resources/defs/materials")?;
        storage.apply_inherits()?;

        let mut storage = DefinitionStorage::<crate::defs::body::BodyDefinition>::from_folder(
            "../resources/defs/bodies",
        )?;
        storage.apply_inherits()?;

        let mut storage =
            DefinitionStorage::<crate::defs::digestion::DigestionDefinition>::from_folder(
                "../resources/defs/digestion",
            )?;
        storage.apply_inherits()?;

        let _ = DefinitionStorage::<crate::defs::race::RaceDefinition>::from_folder(
            "../resources/defs/races",
        )?;

        let _ = DefinitionStorage::<crate::defs::creature::CreatureDefinition>::from_folder(
            "../resources/defs/creatures",
        )?;

        let _ = DefinitionStorage::<crate::defs::action::ActionDefinition>::from_folder(
            "../resources/defs/actions",
        )?;

        let _ = DefinitionStorage::<crate::defs::reaction::ReactionDefinition>::from_folder(
            "../resources/defs/reactions",
        )?;

        let _ = DefinitionStorage::<crate::defs::building::BuildingDefinition>::from_folder(
            "../resources/defs/buildings",
        )?;

        let _ = DefinitionStorage::<crate::defs::foliage::FoliageDefinition>::from_folder(
            "../resources/defs/foliage",
        )?;

        let _ = DefinitionStorage::<crate::defs::item::ItemDefinition>::from_folder(
            "../resources/defs/items",
        )?;

        let _ = DefinitionStorage::<crate::defs::behavior::BehaviorDefinition>::from_folder(
            "../resources/defs/behaviors",
        )?;

        Ok(())
    }
}
