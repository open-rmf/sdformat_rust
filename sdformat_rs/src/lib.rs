extern crate yaserde_derive;
use std::collections::HashMap;
use std::io::{Read, Write};

use nalgebra::*;
use yaserde::xml;
use yaserde::xml::attribute::OwnedAttribute;
use yaserde::xml::namespace::Namespace;
use yaserde::{YaDeserialize, YaSerialize};
use yaserde_derive::{YaDeserialize, YaSerialize};

// Most of the structs are generated automatically from the
include!(concat!(env!("OUT_DIR"), "/sdf.rs"));

#[derive(Default, PartialEq, Clone, Debug)]
pub enum ElementData {
    #[default]
    Empty,
    Integer(i64),
    Double(f64),
    String(String),
    Nested(HashMap<String, XmlElement>),
}

impl ElementData {
    fn to_string(&self) -> String {
        match self {
            ElementData::Empty => "".into(),
            ElementData::Integer(val) => val.to_string(),
            ElementData::Double(val) => val.to_string(),
            ElementData::String(val) => val.clone(),
            ElementData::Nested(_) => unreachable!(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct XmlElement {
    pub attributes: HashMap<String, ElementData>,
    pub data: ElementData,
}

// Manually declare plugin
#[derive(Default, PartialEq, Clone, Debug)]
pub struct SdfPlugin {
    pub name: String,
    pub filename: String,
    pub elements: HashMap<String, XmlElement>,
}

fn parse_data(val: &str) -> ElementData {
    if val.is_empty() {
        ElementData::Empty
    } else if let Ok(val) = str::parse::<i64>(val) {
        ElementData::Integer(val)
    } else if let Ok(val) = str::parse::<f64>(val) {
        ElementData::Double(val)
    } else {
        ElementData::String(val.into())
    }
}

fn deserialize_element<R: Read>(
    reader: &mut yaserde::de::Deserializer<R>,
) -> Result<(String, XmlElement), String> {
    let (name, attributes) = match reader.next_event()? {
        xml::reader::XmlEvent::StartElement {
            name, attributes, ..
        } => (name.local_name, attributes),
        _ => return Err("Unexpected event found when deserializing plugin element".to_string()),
    };
    let mut element = XmlElement::default();
    for attr in attributes.iter() {
        element
            .attributes
            .insert(attr.name.local_name.clone(), parse_data(&attr.value));
    }
    match reader.peek()? {
        xml::reader::XmlEvent::Characters(value) => {
            element.data = parse_data(value);
            reader.next_event()?;
            // Discard the next element, it should be an end event
            reader.next_event()?;
        }
        xml::reader::XmlEvent::StartElement { .. } => {
            let mut elements = HashMap::new();
            // TODO(luca) make sure we are ending this element
            while !matches!(reader.peek(), Ok(xml::reader::XmlEvent::EndElement { .. })) {
                let (name, data) = deserialize_element(reader)?;
                elements.insert(name, data);
            }
            element.data = ElementData::Nested(elements);
            // Discard the next element, it is an end event
            reader.next_event()?;
        }
        xml::reader::XmlEvent::EndElement { .. } => {
            reader.next_event()?;
            element.data = ElementData::Empty
        }
        _ => return Err("Unexpected event found when deserializing plugin data".to_string()),
    };
    Ok((name, element))
}

impl YaDeserialize for SdfPlugin {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        let mut plugin = SdfPlugin::default();
        let read_attribute = |attributes: &Vec<OwnedAttribute>, name: &str| {
            attributes
                .iter()
                .find(|attr| attr.name.local_name == name)
                .map(|attr| attr.value.clone())
                .unwrap_or_default()
        };
        // deserializer code
        if let Ok(xml::reader::XmlEvent::StartElement { attributes, .. }) = reader.next_event() {
            plugin.name = read_attribute(&attributes, "name");
            plugin.filename = read_attribute(&attributes, "filename");
            while !matches!(reader.peek()?, xml::reader::XmlEvent::EndElement { .. }) {
                let (name, data) = deserialize_element(reader)?;
                plugin.elements.insert(name, data);
            }
            Ok(plugin)
        } else {
            Err("Element not found when parsing plugin".to_string())
        }
    }
}

fn serialize_element<W: Write>(
    name: &str,
    elem: &XmlElement,
    serializer: &mut yaserde::ser::Serializer<W>,
) -> Result<(), String> {
    let mut builder = xml::writer::XmlEvent::start_element(name);
    let converted = elem
        .attributes
        .iter()
        .map(|(name, data)| (name, data.to_string()))
        .collect::<Vec<_>>();
    for (name, data) in converted.iter() {
        builder = builder.attr(xml::name::Name::local(name), data);
    }
    serializer.write(builder).map_err(|e| e.to_string())?;
    match &elem.data {
        ElementData::Empty
        | ElementData::Integer(_)
        | ElementData::Double(_)
        | ElementData::String(_) => {
            serializer
                .write(xml::writer::XmlEvent::Characters(&elem.data.to_string()))
                .map_err(|e| e.to_string())?;
        }
        ElementData::Nested(elements) => {
            for (name, data) in elements.iter() {
                serialize_element(name, data, serializer)?;
            }
        }
    }
    serializer
        .write(xml::writer::XmlEvent::end_element())
        .map_err(|e| e.to_string())?;
    Ok(())
}

impl YaSerialize for SdfPlugin {
    fn serialize<W: Write>(
        &self,
        serializer: &mut yaserde::ser::Serializer<W>,
    ) -> Result<(), String> {
        serializer
            .write(
                xml::writer::XmlEvent::start_element("plugin")
                    .attr("name", &self.name)
                    .attr("filename", &self.filename),
            )
            .map_err(|e| e.to_string())?;
        for (name, data) in &self.elements {
            serialize_element(name, data, serializer)?;
        }
        serializer
            .write(xml::writer::XmlEvent::end_element())
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
        serializer
            .write(xml::writer::XmlEvent::Characters(&format!(
                "{} {} {}",
                self.0.x, self.0.y, self.0.z
            )))
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
        serializer
            .write(xml::writer::XmlEvent::Characters(&format!(
                "{} {} {}",
                self.0.x, self.0.y, self.0.z
            )))
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
