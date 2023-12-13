extern crate yaserde_derive;
use std::io::{Read, Write};

use nalgebra::*;
use yaserde::xml;
use yaserde::xml::attribute::OwnedAttribute;
use yaserde::xml::namespace::Namespace;

use yaserde::{YaDeserialize, YaSerialize};
use yaserde_derive::{YaDeserialize, YaSerialize};

// Most of the structs are generated automatically from the
include!(concat!(env!("OUT_DIR"), "/sdf.rs"));

// Manually declare plugin
#[derive(Default, PartialEq, Clone, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "plugin")]
pub struct SdfPlugin {}

// Frame is another wierdo. For some reason it refuses to serialize/deserialize automatically
// Hence the manual definition
// Todo(arjo): Actually implement Frame.
#[derive(Default, PartialEq, Clone, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "frame")]
pub struct SdfFrame {}

// Geometry should really be an enum rather than a list of Options, redefine it here
/// The shape of the visual or collision object.
#[derive(Default, PartialEq, Clone, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "geometry")]
pub enum SdfGeometry {
    #[yaserde(child, rename = "empty")]
    #[default]
    Empty,
    #[yaserde(child, rename = "box")]
    r#Box(SdfBoxShape),
    #[yaserde(child, rename = "capsule")]
    Capsule(SdfCapsuleShape),
    #[yaserde(child, rename = "cylinder")]
    Cylinder(SdfCylinderShape),
    #[yaserde(child, rename = "ellipsoid")]
    Ellipsoid(SdfEllipsoidShape),
    #[yaserde(child, rename = "heightmap")]
    Heightmap(SdfHeightmapShape),
    #[yaserde(child, rename = "image")]
    Image(SdfImageShape),
    #[yaserde(child, rename = "mesh")]
    Mesh(SdfMeshShape),
    #[yaserde(child, rename = "plane")]
    Plane(SdfPlaneShape),
    #[yaserde(child, rename = "polyline")]
    Polyline(SdfPolylineShape),
    #[yaserde(child, rename = "sphere")]
    Sphere(SdfSphereShape),
}

/// Simple implementation of pose
pub struct Pose {
    /// Translation vector
    pub translation: Vector3<f64>,
    /// Rotation
    pub rotation: Rotation3<f64>,
    /// Relative pose
    pub relative_to: String,
}

impl SdfPose {
    /// Lazily retrieve the pose as an Isometry
    /// In the event the pose is not parseable it returns a String based error.
    pub fn get_pose(&self) -> Result<Pose, String> {
        let digits = self
            .data
            .split_whitespace()
            .map(|dig| dig.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| format!("Failed to parse pose values from {:?}", self.data))?;

        let relative_to = self.relative_to.clone().unwrap_or_default();

        if digits.len() == 6 {
            let translation = Vector3::new(digits[0], digits[1], digits[2]);
            if let Some(degrees) = self.degrees {
                let rotation = if degrees {
                    // TODO(arjo): Pose parsing code
                    use std::f64::consts::PI;
                    Rotation3::from_euler_angles(
                        digits[3] / PI * 180.0,
                        digits[4] / PI * 180.0,
                        digits[5] / PI * 180.0,
                    )
                } else {
                    Rotation3::from_euler_angles(digits[3], digits[4], digits[5])
                };
                return Ok(Pose {
                    translation,
                    rotation,
                    relative_to,
                });
            } else {
                let rotation = Rotation3::from_euler_angles(digits[3], digits[4], digits[5]);

                return Ok(Pose {
                    translation,
                    rotation,
                    relative_to,
                });
            }
        } else if digits.len() == 7 {
            let translation = Vector3::new(digits[0], digits[1], digits[2]);
            let (_norm, half_angle, axis) =
                Quaternion::new(digits[3], digits[4], digits[5], digits[6]).polar_decomposition();
            let rotation = if let Some(axis) = axis {
                Rotation3::from_axis_angle(&axis, half_angle * 2.0)
            } else {
                Rotation3::from_axis_angle(&Vector3::y_axis(), half_angle * 2.0)
            };
            return Ok(Pose {
                translation,
                rotation,
                relative_to,
            });
        }
        Err("Failed to parse pose".to_string())
    }
}

pub use yaserde::de::from_str;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vector3d(pub Vector3<f64>);

impl Vector3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3d(Vector3::new(x, y, z))
    }
}

impl YaDeserialize for Vector3d {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        // deserializer code
        reader.next_event()?;
        if let Ok(xml::reader::XmlEvent::Characters(v)) = reader.peek() {
            let sz = v
                .split_whitespace()
                .map(|x| x.parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "Unable to parse Vector3 into floats".to_string())?;

            if sz.len() != 3 {
                return Err("Expected 3 items in Vec3 field".to_string());
            }

            Ok(Vector3d(Vector3::new(sz[0], sz[1], sz[2])))
        } else {
            Err("String of elements not found while parsing Vec3".to_string())
        }
    }
}

impl YaSerialize for Vector3d {
    fn serialize<W: Write>(
        &self,
        serializer: &mut yaserde::ser::Serializer<W>,
    ) -> Result<(), String> {
        // serializer code
        let Some(yaserde_label) = serializer.get_start_event_name() else {
            return Err("vector3d is a primitive".to_string());
        };
        let struct_start_event =
            yaserde::xml::writer::XmlEvent::start_element(yaserde_label.as_ref());

        serializer
            .write(struct_start_event)
            .map_err(|e| e.to_string())?;
        serializer
            .write(xml::writer::XmlEvent::Characters(&format!(
                "{} {} {}",
                self.0.x, self.0.y, self.0.z
            )))
            .map_err(|e| e.to_string())?;
        serializer
            .write(yaserde::xml::writer::XmlEvent::end_element())
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn serialize_attributes(
        &self,
        attributes: Vec<OwnedAttribute>,
        namespace: Namespace,
    ) -> Result<(Vec<OwnedAttribute>, Namespace), String> {
        println!("{:?}", namespace);
        Ok((attributes, namespace))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vector3i(pub Vector3<i64>);

impl YaDeserialize for Vector3i {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        // deserializer code
        reader.next_event()?;
        if let Ok(xml::reader::XmlEvent::Characters(v)) = reader.peek() {
            let sz = v
                .split_whitespace()
                .map(|x| x.parse::<i64>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "Unable to parse Vector3 into ints".to_string())?;

            if sz.len() != 3 {
                return Err("Expected 3 items in Vec3 field".to_string());
            }

            Ok(Vector3i(Vector3::new(sz[0], sz[1], sz[2])))
        } else {
            Err("String of elements not found while parsing Vec3".to_string())
        }
    }
}

impl YaSerialize for Vector3i {
    fn serialize<W: Write>(
        &self,
        serializer: &mut yaserde::ser::Serializer<W>,
    ) -> Result<(), String> {
        // serializer code
        let Some(yaserde_label) = serializer.get_start_event_name() else {
            return Err("vector3d is a primitive".to_string());
        };
        let struct_start_event =
            yaserde::xml::writer::XmlEvent::start_element(yaserde_label.as_ref());

        serializer
            .write(struct_start_event)
            .map_err(|e| e.to_string())?;
        serializer
            .write(xml::writer::XmlEvent::Characters(&format!(
                "{} {} {}",
                self.0.x, self.0.y, self.0.z
            )))
            .map_err(|e| e.to_string())?;
        serializer
            .write(yaserde::xml::writer::XmlEvent::end_element())
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn serialize_attributes(
        &self,
        attributes: Vec<OwnedAttribute>,
        namespace: Namespace,
    ) -> Result<(Vec<OwnedAttribute>, Namespace), String> {
        Ok((attributes, namespace))
    }
}
