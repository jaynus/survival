use core::{
    amethyst::ecs::{Component, VecStorage},
    defs::psyche::{NeedsContainer, PsycheTraitId},
    fnv::FnvHashMap,
    shrinkwraprs::Shrinkwrap,
    smallvec::SmallVec,
};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PersonalityComponent {
    pub traits: SmallVec<[(bool, PsycheTraitId, FnvHashMap<usize, u32>); 32]>,
}
impl Component for PersonalityComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Shrinkwrap, Debug, Default, serde::Deserialize, serde::Serialize)]
#[shrinkwrap(mutable)]
pub struct PyscheNeedsComponent(pub NeedsContainer);
impl PyscheNeedsComponent {
    pub fn new(inner: NeedsContainer) -> Self {
        Self(inner)
    }
}
impl Component for PyscheNeedsComponent {
    type Storage = VecStorage<Self>;
}
