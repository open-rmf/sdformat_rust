extern crate yaserde_derive;
use std::io::{Read, Write};
use std::string::ParseError;

use nalgebra::*;
use yaserde::xml;
use yaserde::xml::attribute::OwnedAttribute;
use yaserde::xml::namespace::Namespace;
use yaserde::{de, ser, YaDeserialize, YaSerialize};
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
        let digits_raw = self.data.split_whitespace().map(|dig| dig.parse::<f64>());

        let mut digits = vec![];
        for dig in digits_raw {
            if let Ok(dig) = dig {
                digits.push(dig);
            } else {
                return Err("Failed to parse Isometry from ".to_string());
            }
        }

        let mut frame = "".to_string();

        if let Some(fr) = &self.relative_to {
            frame = fr.clone();
        }

        if digits.len() == 6 {
            let translation = Vector3::new(digits[0], digits[1], digits[2]);
            if let Some(degrees) = self.degrees {
                let rot = if degrees {
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
                    translation: translation,
                    rotation: rot,
                    relative_to: frame,
                });
            } else {
                let rot = Rotation3::from_euler_angles(digits[3], digits[4], digits[5]);

                return Ok(Pose {
                    translation: translation,
                    rotation: rot,
                    relative_to: frame,
                });
            }
        } else if digits.len() == 7 {
            let translation = Vector3::new(digits[0], digits[1], digits[2]);
            let (_norm, half_angle, axis) =
                Quaternion::new(digits[3], digits[4], digits[5], digits[6]).polar_decomposition();
            let rot = if let Some(axis) = axis {
                Rotation3::from_axis_angle(&axis, half_angle * 2.0)
            } else {
                Rotation3::from_axis_angle(&Vector3::y_axis(), half_angle * 2.0)
            };
            return Ok(Pose {
                translation: translation,
                rotation: rot,
                relative_to: frame,
            });
        }
        Err("Failed to parse pose".to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vector3d {
    pub data: Vector3<f64>,
}

impl YaDeserialize for Vector3d {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        // deserializer code
        reader.next_event()?;
        if let Ok(xml::reader::XmlEvent::Characters(v)) = reader.peek() {
            let sz: Vec<&str> = v.split_whitespace().collect();
            if sz.len() != 3 {
                return Err("Expected 3 items in Vec3 field".to_string());
            }

            let x = sz[0].parse::<f64>();
            let y = sz[1].parse::<f64>();
            let z = sz[2].parse::<f64>();

            if let Ok(x) = x {
                if let Ok(y) = y {
                    if let Ok(z) = z {
                        return Ok(Vector3d {
                            data: Vector3::new(x, y, z),
                        });
                    }
                }
            }
            return Err("Unable to parse Vector3 into floats".to_string());
        } else {
            return Err("String of elements not found while parsing Vec3".to_string());
        }
    }
}

impl YaSerialize for Vector3d {
    fn serialize<W: Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        // serializer code
        Err("Not yet implemented".to_string())
    }

    fn serialize_attributes(
        &self,
        attributes: Vec<OwnedAttribute>,
        namespace: Namespace,
    ) -> Result<(Vec<OwnedAttribute>, Namespace), String> {
        Ok((attributes, namespace))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vector3i {
    pub data: Vector3<i64>,
}

impl YaDeserialize for Vector3i {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        // deserializer code
        reader.next_event()?;
        if let Ok(xml::reader::XmlEvent::Characters(v)) = reader.peek() {
            let sz: Vec<&str> = v.split_whitespace().collect();
            if sz.len() != 3 {
                return Err("Expected 3 items in Vec3 field".to_string());
            }

            let x = sz[0].parse::<i64>();
            let y = sz[1].parse::<i64>();
            let z = sz[2].parse::<i64>();

            if let Ok(x) = x {
                if let Ok(y) = y {
                    if let Ok(z) = z {
                        return Ok(Vector3i {
                            data: Vector3::new(x, y, z),
                        });
                    }
                }
            }
            return Err("Unable to parse Vector3 into floats".to_string());
        } else {
            return Err("String of elements not found while parsing Vec3".to_string());
        }
    }
}

impl YaSerialize for Vector3i {
    fn serialize<W: Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        // serializer code
        Err("Not yet implemented".to_string())
    }

    fn serialize_attributes(
        &self,
        attributes: Vec<OwnedAttribute>,
        namespace: Namespace,
    ) -> Result<(Vec<OwnedAttribute>, Namespace), String> {
        Ok((attributes, namespace))
    }
}
