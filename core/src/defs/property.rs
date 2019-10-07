use crate::{
    amethyst::core::math::Vector3,
    bitflags_serial,
    defs::{
        digestion::{EdibleKind, EdibleState},
        foliage::FoliageCategory,
    },
};
use bitflags::*;
use strum_macros::{AsRefStr, EnumDiscriminants, EnumProperty};

#[derive(
    Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Dimensions {
    Cube { x: u64, y: u64, z: u64 }, // xyz, mm
    Sphere { radius: u64 },          //mm
}
impl Default for Dimensions {
    fn default() -> Self { Dimensions::Sphere { radius: 0 } }
}

impl From<&Vector3<u64>> for Dimensions {
    fn from(rhv: &Vector3<u64>) -> Self {
        Dimensions::Cube {
            x: rhv.x,
            y: rhv.y,
            z: rhv.z,
        }
    }
}
impl From<Vector3<u64>> for Dimensions {
    fn from(rhv: Vector3<u64>) -> Self { Self::from(&rhv) }
}

impl From<&Vector3<u32>> for Dimensions {
    fn from(rhv: &Vector3<u32>) -> Self {
        Dimensions::Cube {
            x: u64::from(rhv.x),
            y: u64::from(rhv.y),
            z: u64::from(rhv.z),
        }
    }
}
impl From<Vector3<u32>> for Dimensions {
    fn from(rhv: Vector3<u32>) -> Self { Self::from(&rhv) }
}

#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Average<T: Copy + Clone + PartialEq + Eq + std::hash::Hash + PartialOrd + Ord> {
    pub mean: T,
    pub deviation: T,
}

bitflags_serial! {
    pub struct WearableLocation: u16 {
        const Head          = 1 << 1;
        const Face          = 1 << 2;

        const LeftTorso     = 1 << 3;
        const RightTorso    = 1 << 4;
        const Torso         = 1 << 5;

        const UpperLeg      = 1 << 6;
        const LowerLeg      = 1 << 7;
        const Leg           = 1 << 8;

        const UpperArm      = 1 << 9;
        const LowerArm      = 1 << 10;
        const Arm           = 1 << 11;

        const Hand          = 1 << 12;
        const Foot          = 1 << 13;
    }
}

bitflags_serial! {
    pub struct MovementFlags: u16 {
        const Walk = 1 << 1;
    }
}

bitflags_serial! {
    pub struct ManipulateFlags: u16 {
        const Any = 1;

    }
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize, AsRefStr,
)]
pub enum InteractionType {
    Load,
    Unload,
    Use,
    Invalid,
}
impl Default for InteractionType {
    fn default() -> Self { InteractionType::Invalid }
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize, AsRefStr,
)]
pub enum PropertyCategory {
    Body,
    Foliage,
}

#[derive(
    Debug, Clone, Copy, Hash, EnumProperty, EnumDiscriminants, serde::Serialize, serde::Deserialize,
)]
#[allow(clippy::derive_hash_xor_eq)]
#[strum_discriminants(name(PropertyKind))]
#[strum_discriminants(derive(Hash, AsRefStr))]
#[repr(u8)]
pub enum Property {
    Edible(EdibleKind, EdibleState),

    //Craft properties?
    Cooking(u8),
    Chopping(u8),
    Stonecutting(u8),
    Digging(u8),

    // Combat properties
    Stabbing(u8),
    Bashing(u8),
    Cutting(u8),

    CanPickup,

    Wearable {
        location: WearableLocation,
    },
    Container {
        dimensions: Dimensions,
    },
    Size {
        dimensions: Dimensions, // cm3
    },
    Interactable(InteractionType),
    Empty,

    #[strum(props=(Category="Body"))]
    Movement(MovementFlags),

    #[strum(props=(Category="Body"))]
    MovementSpeed(u32),

    #[strum(props=(Category="Body"))]
    Manipulate(ManipulateFlags),

    // Foliage types
    #[strum(props=(Category="Foliage"))]
    Foliage(FoliageCategory),

    Building,
}
impl PartialEq for Property {
    fn eq(&self, other: &Self) -> bool {
        let left: PropertyKind = self.into();
        let right: PropertyKind = other.into();
        left == right
    }
}
impl Eq for Property {}
impl Default for Property {
    fn default() -> Self { Property::Empty }
}
