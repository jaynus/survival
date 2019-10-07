#![allow(clippy::pub_enum_variant_names)]
use amethyst::{core::math::Point3, ecs::Entity, input::InputEvent, tiles::iters::Region};
use hibitset::BitSet;
use std::fmt::{self, Display};
use strum_macros::{EnumCount, EnumDiscriminants};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[strum_discriminants(name(AxisType), derive(PartialOrd, Ord, Hash, EnumCount))]
pub enum AxisBinding {
    CameraX,
    CameraY,
    CameraZ,
    CameraScale,
}

#[derive(
    Clone, Copy, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize, EnumDiscriminants,
)]
#[strum_discriminants(name(ActionType), derive(PartialOrd, Ord, Hash, EnumCount))]
pub enum ActionBinding {
    Select,
    DoAction,

    Pause,

    UpZ,
    DownZ,
}

impl Display for AxisBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for ActionBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct BindingTypes;
impl amethyst::input::BindingTypes for BindingTypes {
    type Axis = AxisBinding;
    type Action = ActionBinding;
}

#[derive(Clone, Debug)]
pub enum FilteredInputEvent {
    Filtered(InputEvent<BindingTypes>),
    Free(InputEvent<BindingTypes>),
}

#[derive(
    Clone, Copy, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize, EnumDiscriminants,
)]
pub enum InputStateFlags {
    Normal,
    Selection,
    Placement,
}
impl Default for InputStateFlags {
    fn default() -> Self {
        InputStateFlags::Normal
    }
}

#[derive(Debug, Clone)]
pub enum SelectionData {
    PawnGroup {
        region: Option<Region>,
        entities: BitSet,
    },
    ItemGroup {
        region: Option<Region>,
        entities: BitSet,
    },
    Building(Entity),
}

#[derive(Debug, Clone)]
pub struct InputState {
    pub current: InputStateFlags,
    pub selection: Option<SelectionData>,
    pub last_selection: Option<SelectionData>,

    pub mouse_world_position: Point3<f32>,
}
impl InputState {
    pub fn update_selection(&mut self, data: Option<SelectionData>) {
        self.last_selection = self.selection.clone();
        self.selection = data;
    }
}
impl Default for InputState {
    fn default() -> Self {
        Self {
            current: InputStateFlags::default(),
            selection: None,
            last_selection: None,

            mouse_world_position: Point3::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PlayerInputEvent {
    StartBuildingPlacement { building_id: u32 },
}
