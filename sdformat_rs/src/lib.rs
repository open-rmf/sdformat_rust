extern crate yaserde_derive;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use nalgebra::*;
use yaserde::xml;
use yaserde::xml::attribute::OwnedAttribute;
use yaserde::xml::namespace::Namespace;

use yaserde::{YaDeserialize, YaSerialize};
use yaserde_derive::{YaDeserialize, YaSerialize};

// Most of the structs are generated automatically from the
include!(concat!(env!("OUT_DIR"), "/sdf.rs"));

pub struct Boxed<T> {
    inner: Box<T>,
}

impl<T> From<T> for Boxed<T> {
    fn from(t: T) -> Self {
        Self {
            inner: Box::new(t),
        }
    }
}

impl<T: PartialEq> PartialEq for Boxed<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.inner == rhs.inner
    }
}

impl<T: YaDeserialize> YaDeserialize for Boxed<T> {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        Ok(Self {
            inner: Box::new(T::deserialize(reader)?),
        })
    }
}

impl<T: YaSerialize> YaSerialize for Boxed<T> {
    fn serialize<W: Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        self.inner.as_ref().serialize(writer)
    }

    fn serialize_attributes(
        &self,
        attributes: Vec<OwnedAttribute>,
        namespace: Namespace,
    ) -> Result<(Vec<OwnedAttribute>, Namespace), String> {
        self.inner
            .as_ref()
            .serialize_attributes(attributes, namespace)
    }
}

impl<T: Default> Default for Boxed<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T: Clone> Clone for Boxed<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Debug> Debug for Boxed<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.as_ref().fmt(f)
    }
}

impl<T> Deref for Boxed<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Boxed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum ElementData {
    String(String),
    Nested(ElementMap),
}

impl Default for ElementData {
    fn default() -> Self {
        ElementData::String("".into())
    }
}

impl TryFrom<ElementData> for f64 {
    type Error = String;

    fn try_from(e: ElementData) -> Result<Self, Self::Error> {
        match e {
            ElementData::String(s) => s
                .parse::<f64>()
                .map_err(|_| "Unable to parse into f64".to_string()),
            ElementData::Nested(_) => Err("Nested element found".into()),
        }
    }
}

impl TryFrom<ElementData> for i64 {
    type Error = String;

    fn try_from(e: ElementData) -> Result<Self, Self::Error> {
        match e {
            ElementData::String(s) => s
                .parse::<i64>()
                .map_err(|_| "Unable to parse into i64".to_string()),
            ElementData::Nested(_) => Err("Nested element found".into()),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct XmlElement {
    pub attributes: HashMap<String, String>,
    pub name: Arc<str>,
    pub data: ElementData,
}

impl Default for XmlElement {
    fn default() -> Self {
        Self {
            attributes: Default::default(),
            name: "".into(),
            data: Default::default(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct ElementMap {
    indexes: HashMap<Arc<str>, BTreeSet<usize>>,
    elements: Vec<XmlElement>,
}

// Manually declare plugin
#[derive(Default, PartialEq, Clone, Debug)]
pub struct SdfPlugin {
    pub name: String,
    pub filename: String,
    pub elements: ElementMap,
}

impl ElementMap {
    pub fn get(&self, name: &str) -> Option<&XmlElement> {
        self.indexes
            .get(name)
            .and_then(|idxs| self.elements.get(*idxs.iter().next()?))
    }

    pub fn get_all(&self, name: &str) -> Option<Vec<&XmlElement>> {
        self.indexes
            .get(name)
            .and_then(|idxs| idxs.iter().map(|idx| self.elements.get(*idx)).collect())
    }

    pub fn push(&mut self, elem: XmlElement) {
        let idx = self.elements.len();
        let name = elem.name.clone();
        self.elements.push(elem);
        self.indexes.entry(name).or_default().insert(idx);
    }

    pub fn all(&self) -> &[XmlElement] {
        &self.elements
    }
}

fn deserialize_element<R: Read>(
    reader: &mut yaserde::de::Deserializer<R>,
) -> Result<XmlElement, String> {
    let (name, attributes) = match reader.next_event()? {
        xml::reader::XmlEvent::StartElement {
            name, attributes, ..
        } => (name.local_name, attributes),
        _ => return Err("Unexpected event found when deserializing plugin element".to_string()),
    };
    let mut element = XmlElement {
        name: name.into(),
        ..Default::default()
    };
    for attr in attributes.iter() {
        element
            .attributes
            .insert(attr.name.local_name.clone(), attr.value.to_owned());
    }
    match reader.peek()? {
        xml::reader::XmlEvent::Characters(value) => {
            element.data = ElementData::String(value.clone());
            reader.next_event()?;
            // Discard the next element, it should be an end event
            reader.next_event()?;
        }
        xml::reader::XmlEvent::StartElement { .. } => {
            let mut elements = ElementMap::default();
            // TODO(luca) make sure we are ending this element
            while !matches!(reader.peek(), Ok(xml::reader::XmlEvent::EndElement { .. })) {
                let elem = deserialize_element(reader)?;
                elements.push(elem);
            }
            element.data = ElementData::Nested(elements);
            // Discard the next element, it is an end event
            reader.next_event()?;
        }
        xml::reader::XmlEvent::EndElement { .. } => {
            reader.next_event()?;
            element.data = ElementData::default();
        }
        _ => return Err("Unexpected event found when deserializing plugin data".to_string()),
    };
    Ok(element)
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
                let elem = deserialize_element(reader)?;
                plugin.elements.push(elem);
            }
            Ok(plugin)
        } else {
            Err("Element not found when parsing plugin".to_string())
        }
    }
}

fn serialize_element<W: Write>(
    elem: &XmlElement,
    serializer: &mut yaserde::ser::Serializer<W>,
) -> Result<(), String> {
    let mut builder = xml::writer::XmlEvent::start_element(&*elem.name);
    for (name, data) in elem.attributes.iter() {
        builder = builder.attr(xml::name::Name::local(name), data);
    }
    serializer.write(builder).map_err(|e| e.to_string())?;
    match &elem.data {
        ElementData::String(s) => {
            serializer
                .write(xml::writer::XmlEvent::Characters(s))
                .map_err(|e| e.to_string())?;
        }
        ElementData::Nested(elements) => {
            for element in elements.all().iter() {
                serialize_element(element, serializer)?;
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
        for element in self.elements.elements.iter() {
            serialize_element(element, serializer)?;
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

impl TryFrom<String> for Vector3d {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let sz = s
            .split_whitespace()
            .map(|x| x.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Unable to parse Vector3 into floats".to_string())?;

        if sz.len() != 3 {
            return Err("Expected 3 items in Vec3 field".to_string());
        }

        Ok(Vector3d::new(sz[0], sz[1], sz[2]))
    }
}

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
            v.clone().try_into()
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

impl TryFrom<String> for Vector3i {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let sz = s
            .split_whitespace()
            .map(|x| x.parse::<i64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Unable to parse Vector3 into ints".to_string())?;

        if sz.len() != 3 {
            return Err("Expected 3 items in Vec3 field".to_string());
        }

        Ok(Vector3i::new(sz[0], sz[1], sz[2]))
    }
}

impl Vector3i {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self(Vector3::new(x, y, z))
    }
}

impl YaDeserialize for Vector3i {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        // deserializer code
        reader.next_event()?;
        if let Ok(xml::reader::XmlEvent::Characters(v)) = reader.peek() {
            v.clone().try_into()
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
