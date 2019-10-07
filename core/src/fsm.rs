use crate::{defs::property::InteractionType, shrinkwraprs::Shrinkwrap};
use amethyst::{
    core::math::{Point3, Vector3},
    ecs::Entity,
};
use std::fmt::Debug;

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    strum_macros::EnumIter,
    strum_macros::AsRefStr,
    strum_macros::EnumCount,
    num_derive::FromPrimitive,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(usize)]
pub enum TaskCategory {
    Mining = 0,
    Woodcutting = 1,
    Woodcrafting = 2,
    Woodworking = 3,
    Stonecutting = 4,
    Stonecrafting = 5,
    Hunting = 6,
    Farming = 7,
    Fishing = 8,
    HaulingFood = 9,
    HaulingItems = 10,
    HaulingStone = 11,
    HaulingWood = 12,
    HaulingRefuse = 13,
    Cleaning = 14,
    Doctoring = 15,
    Construction = 16,
    Unspecified = 17,
}
impl Default for TaskCategory {
    fn default() -> Self { TaskCategory::Unspecified }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MovementEvent {
    Direction(Vector3<u8>),
    To(Point3<u32>),
    Target,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActionStatus {
    Cancelled,
    Failure,
    Fired,
    Active,
    Success,
}
impl Default for ActionStatus {
    fn default() -> Self { ActionStatus::Fired }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ActionTarget {
    Entity(Entity),
    Location(Point3<u32>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ActionEvent {
    pub source: Option<Entity>,
    pub targets: Vec<ActionTarget>,
    pub event: Event,
}
impl ActionEvent {
    pub fn new(source: Option<Entity>, targets: Vec<ActionTarget>, event: Event) -> Self {
        Self {
            source,
            targets,
            event,
        }
    }
}

#[derive(Shrinkwrap)]
pub struct ActionComplete(ActionEvent);

#[derive(
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    strum_macros::AsRefStr,
    strum_macros::Display,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Event {
    Interact(InteractionType),
    Move(MovementEvent),
    Pickup,
    ActivateReaction(String),
    Invalid,
}
impl Default for Event {
    fn default() -> Self { Event::Invalid }
}

pub trait ConditionValue:
    Debug + Clone + PartialEq + Eq + serde::Serialize + for<'de> serde::Deserialize<'de>
{
}
impl<T> ConditionValue for T where
    T: Debug + Clone + PartialEq + Eq + serde::Serialize + for<'de> serde::Deserialize<'de>
{
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub enum ConditionTarget {
    Me,
    Entity,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub enum ConditionKind {
    Near(u32),
    Has,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub enum ConditionEquality {
    Is,
    Not,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Condition<V: ConditionValue> {
    pub target: ConditionTarget,
    pub equality: ConditionEquality,
    pub kind: ConditionKind,
    pub value: V,
}

impl<V: ConditionValue> std::fmt::Display for Condition<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?})", *self)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename(serialize = "Condition", deserialize = "Condition"))]
#[serde(bound(deserialize = "V: ConditionValue"))]
struct ConditionTuple<V: ConditionValue>(
    ConditionTarget,
    #[serde(default = "ConditionTuple::<V>::default_equality")] ConditionEquality,
    ConditionKind,
    V,
);
impl<V: ConditionValue> ConditionTuple<V> {
    fn default_equality() -> ConditionEquality { ConditionEquality::Is }
}

impl<V: ConditionValue> serde::Serialize for Condition<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ConditionTuple(self.target, self.equality, self.kind, self.value.clone())
            .serialize(serializer)
    }
}
impl<'de, V: ConditionValue> serde::Deserialize<'de> for Condition<V> {
    fn deserialize<D>(deserializer: D) -> Result<Condition<V>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tuple = ConditionTuple::<V>::deserialize(deserializer)?;
        Ok(Condition {
            target: tuple.0,
            equality: tuple.1,
            kind: tuple.2,
            value: tuple.3.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct TestValue(String);

    #[test]
    fn deserialize_condition() {
        let de: Condition<TestValue> = ron::de::from_str("(Me, Is, Near(5), (\"asdf\"))").unwrap();
        assert_eq!(
            de,
            Condition {
                target: ConditionTarget::Me,
                equality: ConditionEquality::Is,
                kind: ConditionKind::Near(5),
                value: TestValue("asdf".to_string())
            }
        );
    }
    #[test]
    fn serialize_condition() {
        let val = Condition {
            target: ConditionTarget::Me,
            equality: ConditionEquality::Is,
            kind: ConditionKind::Near(5),
            value: TestValue("asdf".to_string()),
        };

        assert_eq!(
            ron::ser::to_string_pretty(&val, Default::default()).unwrap(),
            "(Me, Is, Near(5), (\"asdf\"))"
        )
    }
}
