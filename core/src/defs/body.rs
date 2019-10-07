use crate::{
    bitflags_serial,
    components::PropertiesComponent,
    defs::{
        material::MaterialLayerRef,
        property::{Average, Dimensions, Property},
        sprites::SpriteRef,
        Definition, HasProperties, InheritDefinition, Named,
    },
};
use survival_derive::NamedDefinition;

use bitflags::*;
use petgraph;

pub type BodyDefinitionId = u32;

bitflags_serial! {
    pub struct PartFlags: u32 {
        const Stance        = 1 << 1;
        const FineMotor     = 1 << 2;
        const Organ         = 1 << 3;
        const Limb          = 1 << 4;

        const Skeleton      = 1 << 5;
        const Nervous       = 1 << 6;
        const Thought       = 1 << 7;
        const Flight        = 1 << 8;
        const Hear          = 1 << 9;
        const Sight         = 1 << 10;
        const Smell         = 1 << 11;
        const Eating        = 1 << 12;

        const Circulation   = 1 << 13;
        const Respitory     = 1 << 14;

        const UpperBody     = 1 << 15;
        const LowerBody     = 1 << 16;
        const Head          = 1 << 17;
    }
}

#[derive(NamedDefinition, Clone, Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct PartLayer {
    name: String,
    #[serde(skip)]
    id: Option<u32>,
    pub material: MaterialLayerRef,
    pub flags: PartFlags,
}
impl PartLayer {
    pub fn new(name: &str, material: MaterialLayerRef, flags: PartFlags) -> Self {
        Self {
            id: None,
            name: name.to_string(),
            material,
            flags,
        }
    }
}

#[derive(NamedDefinition, Clone, Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct Part {
    name: String,
    #[serde(skip)]
    id: Option<u32>,
    pub group: Option<String>,
    pub relative_size: u32,
    pub layers: Vec<PartLayer>,
}
impl Part {
    pub fn new(
        name: &str,
        relative_size: u32,
        group: Option<String>,
        layers: &[PartLayer],
    ) -> Self {
        Self {
            id: None,
            name: name.to_string(),
            group,
            layers: layers.to_vec(),
            relative_size,
        }
    }
}

bitflags_serial! {
    pub struct JointRelation: u32 {
        const Inside    = 1 << 1;
        const Outside   = 1 << 2;
        const Left      = 1 << 3;
        const Right     = 1 << 4;
        const Front     = 1 << 5;
        const Back      = 1 << 6;
        const Top       = 1 << 7;
        const Bottom    = 1 << 8;
    }
}

#[derive(Clone, Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct Joint {
    pub relation: JointRelation,
    pub relative_size: u32,
}

#[derive(NamedDefinition, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BodyDefinition {
    name: String,

    #[serde(default)]
    inherits: Option<String>,

    #[serde(default)]
    pub sprite: Option<SpriteRef>,

    #[serde(skip)]
    id: Option<u32>,

    #[serde(rename = "parts", default)]
    pub part_graph: Option<petgraph::Graph<Part, Joint>>,

    #[serde(default)]
    pub dimensions: Option<Average<Dimensions>>,

    #[serde(default)]
    pub mass: Option<Average<u64>>,

    #[serde(default = "default_digestion")]
    pub digestion: Option<String>,

    #[serde(default)]
    pub properties: Vec<Property>,
}

pub fn default_digestion() -> Option<String> {
    Some("default".to_string())
}

impl InheritDefinition for BodyDefinition {
    fn parent(&self) -> Option<&str> {
        self.inherits.as_ref().map(String::as_str)
    }

    fn inherit_from(&mut self, parent: &Self) {
        if self.part_graph.is_none() {
            self.part_graph = parent.part_graph.clone();
        }

        if self.digestion.is_none() {
            self.digestion = parent.digestion.clone();
        }

        self.mass = self.mass.map_or(parent.mass, Some);
        self.dimensions = self.dimensions.map_or(parent.dimensions, Some);

        if self.sprite.is_none() {
            self.sprite = parent.sprite.clone();
        }

        parent
            .properties
            .iter()
            .for_each(|p| self.properties.push(p.clone()));
    }
}

impl Default for BodyDefinition {
    fn default() -> Self {
        Self {
            id: None,
            inherits: None,
            name: "(undefined)".to_string(),
            part_graph: Some(petgraph::Graph::new()),
            digestion: default_digestion(),
            mass: Some(Average::default()),
            dimensions: Some(Average::default()),
            properties: Vec::new(),
            sprite: Some(SpriteRef::default()),
        }
    }
}

impl HasProperties for BodyDefinition {
    fn default_properties(&self) -> PropertiesComponent {
        PropertiesComponent::from_iter_ref(self.properties.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amethyst::core::math::Vector3;
    use crate::defs::sprites::SpriteSource;

    #[test]
    fn body_part_graph_serialization_test() -> Result<(), std::io::Error> {
        use std::io::Write;

        let mut details = BodyDefinition::default();
        details.name = "Humanoid".to_string();
        {
            let body = &mut details.part_graph;

            let head = body.as_mut().unwrap().add_node(Part::new(
                "Head",
                100,
                Some("head".to_string()),
                &[],
            ));
            {
                let brain = body.as_mut().unwrap().add_node(Part::new(
                    "Brain",
                    96,
                    Some("head".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    head,
                    brain,
                    Joint {
                        relation: JointRelation::Inside,
                        ..Default::default()
                    },
                );

                let r_ear = body.as_mut().unwrap().add_node(Part::new(
                    "Right Ear",
                    5,
                    Some("head".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    r_ear,
                    head,
                    Joint {
                        relation: JointRelation::Right | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let l_ear = body.as_mut().unwrap().add_node(Part::new(
                    "Left Ear",
                    5,
                    Some("head".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    l_ear,
                    head,
                    Joint {
                        relation: JointRelation::Left | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let r_eye = body.as_mut().unwrap().add_node(Part::new(
                    "Right Eye",
                    1,
                    Some("head".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    r_eye,
                    head,
                    Joint {
                        relation: JointRelation::Right
                            | JointRelation::Front
                            | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let l_eye = body.as_mut().unwrap().add_node(Part::new(
                    "Left Eye",
                    1,
                    Some("head".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    l_eye,
                    head,
                    Joint {
                        relation: JointRelation::Left
                            | JointRelation::Front
                            | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let nose = body.as_mut().unwrap().add_node(Part::new(
                    "Nose",
                    1,
                    Some("head".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    nose,
                    head,
                    Joint {
                        relation: JointRelation::Front | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let mouth = body.as_mut().unwrap().add_node(Part::new(
                    "Mouth",
                    3,
                    Some("head".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    mouth,
                    head,
                    Joint {
                        relation: JointRelation::Front | JointRelation::Outside,
                        ..Default::default()
                    },
                );
            }
            let neck = body.as_mut().unwrap().add_node(Part::new(
                "Neck",
                100,
                Some("torso".to_string()),
                &[],
            ));
            body.as_mut().unwrap().add_edge(
                neck,
                head,
                Joint {
                    relation: JointRelation::Bottom | JointRelation::Outside,
                    ..Default::default()
                },
            );

            let torso = body.as_mut().unwrap().add_node(Part::new(
                "Torso",
                2000,
                Some("torso".to_string()),
                &[],
            ));
            body.as_mut().unwrap().add_edge(
                torso,
                neck,
                Joint {
                    relation: JointRelation::Bottom | JointRelation::Outside,
                    ..Default::default()
                },
            );
            {
                let r_lung = body.as_mut().unwrap().add_node(Part::new(
                    "Right Lung",
                    200,
                    Some("torso".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    torso,
                    r_lung,
                    Joint {
                        relation: JointRelation::Right | JointRelation::Inside,
                        ..Default::default()
                    },
                );
                let l_lung = body.as_mut().unwrap().add_node(Part::new(
                    "Left Lung",
                    200,
                    Some("torso".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    torso,
                    l_lung,
                    Joint {
                        relation: JointRelation::Left | JointRelation::Inside,
                        ..Default::default()
                    },
                );

                let heart = body.as_mut().unwrap().add_node(Part::new(
                    "Heart",
                    50,
                    Some("torso".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    torso,
                    heart,
                    Joint {
                        relation: JointRelation::Inside,
                        ..Default::default()
                    },
                );

                let liver = body.as_mut().unwrap().add_node(Part::new(
                    "Liver",
                    50,
                    Some("torso".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    torso,
                    liver,
                    Joint {
                        relation: JointRelation::Inside,
                        ..Default::default()
                    },
                );

                let spleen = body.as_mut().unwrap().add_node(Part::new(
                    "Spleen",
                    20,
                    Some("torso".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    torso,
                    spleen,
                    Joint {
                        relation: JointRelation::Inside,
                        ..Default::default()
                    },
                );

                let stomach = body.as_mut().unwrap().add_node(Part::new(
                    "Stomach",
                    50,
                    Some("torso".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    torso,
                    stomach,
                    Joint {
                        relation: JointRelation::Inside,
                        ..Default::default()
                    },
                );

                let int = body.as_mut().unwrap().add_node(Part::new(
                    "Intestines",
                    550,
                    Some("torso".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    torso,
                    int,
                    Joint {
                        relation: JointRelation::Inside,
                        ..Default::default()
                    },
                );

                let r_upper_arm = body.as_mut().unwrap().add_node(Part::new(
                    "Right Upper Arm",
                    350,
                    Some("rarm".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    r_upper_arm,
                    torso,
                    Joint {
                        relation: JointRelation::Right | JointRelation::Outside,
                        ..Default::default()
                    },
                );
                let l_upper_arm = body.as_mut().unwrap().add_node(Part::new(
                    "Left Upper Arm",
                    350,
                    Some("larm".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    l_upper_arm,
                    torso,
                    Joint {
                        relation: JointRelation::Left | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let r_lower_arm = body.as_mut().unwrap().add_node(Part::new(
                    "Right Lower Arm",
                    350,
                    Some("rarm".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    r_lower_arm,
                    r_upper_arm,
                    Joint {
                        relation: JointRelation::Bottom | JointRelation::Outside,
                        ..Default::default()
                    },
                );
                let l_lower_arm = body.as_mut().unwrap().add_node(Part::new(
                    "Left Lower Arm",
                    350,
                    Some("larm".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    l_lower_arm,
                    l_upper_arm,
                    Joint {
                        relation: JointRelation::Bottom | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let r_hand = body.as_mut().unwrap().add_node(Part::new(
                    "Right Hand",
                    50,
                    Some("rarm".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    r_hand,
                    r_lower_arm,
                    Joint {
                        relation: JointRelation::Bottom | JointRelation::Outside,
                        ..Default::default()
                    },
                );
                let l_hand = body.as_mut().unwrap().add_node(Part::new(
                    "Left Hand",
                    50,
                    Some("larm".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    l_hand,
                    l_lower_arm,
                    Joint {
                        relation: JointRelation::Bottom | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let r_thigh = body.as_mut().unwrap().add_node(Part::new(
                    "Right Upper Leg",
                    550,
                    Some("rleg".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    r_thigh,
                    torso,
                    Joint {
                        relation: JointRelation::Right
                            | JointRelation::Bottom
                            | JointRelation::Outside,
                        ..Default::default()
                    },
                );
                let l_thigh = body.as_mut().unwrap().add_node(Part::new(
                    "Left Upper Leg",
                    550,
                    Some("lleg".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    l_thigh,
                    torso,
                    Joint {
                        relation: JointRelation::Left
                            | JointRelation::Bottom
                            | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let r_calf = body.as_mut().unwrap().add_node(Part::new(
                    "Right Lower leg",
                    450,
                    Some("rleg".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    r_calf,
                    r_thigh,
                    Joint {
                        relation: JointRelation::Bottom | JointRelation::Outside,
                        ..Default::default()
                    },
                );
                let l_calf = body.as_mut().unwrap().add_node(Part::new(
                    "Left Lower Leg",
                    450,
                    Some("lleg".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    l_calf,
                    l_thigh,
                    Joint {
                        relation: JointRelation::Bottom | JointRelation::Outside,
                        ..Default::default()
                    },
                );

                let r_foot = body.as_mut().unwrap().add_node(Part::new(
                    "Right Foot",
                    75,
                    Some("rleg".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    r_foot,
                    r_calf,
                    Joint {
                        relation: JointRelation::Bottom | JointRelation::Outside,
                        ..Default::default()
                    },
                );
                let l_foot = body.as_mut().unwrap().add_node(Part::new(
                    "Left Foot",
                    75,
                    Some("lleg".to_string()),
                    &[],
                ));
                body.as_mut().unwrap().add_edge(
                    l_foot,
                    l_calf,
                    Joint {
                        relation: JointRelation::Bottom | JointRelation::Outside,
                        ..Default::default()
                    },
                );
            }
        }

        details.sprite = Some(SpriteRef {
            source: SpriteSource::Sheet("default_map".to_string()),
            index: 16,
            ..Default::default()
        });

        details.dimensions = Some(Average {
            mean: Dimensions::Cube {
                x: 430,
                y: 1760,
                z: 200,
            },
            deviation: Dimensions::Cube {
                x: 20,
                y: 700,
                z: 50,
            },
        });
        details.mass = Some(Average {
            mean: 69000,
            deviation: 200,
        });

        let serialized = ron::ser::to_string_pretty(
            &vec![details],
            ron::ser::PrettyConfig::new()
                .with_depth_limit(10)
                .with_separate_tuple_members(false)
                .with_enumerate_arrays(false)
                .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME),
        )
        .unwrap();
        log::trace!("{}", serialized);

        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open("../resources/defs/bodies/humanoid.ron")?
            .write_all(serialized.as_bytes())
    }
}
