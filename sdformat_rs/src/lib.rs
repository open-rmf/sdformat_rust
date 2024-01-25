use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::io::{Read, Write};
use std::sync::Arc;
use std::marker::PhantomData;

use nalgebra::*;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;

// Most of the structs are generated automatically from the
include!(concat!(env!("OUT_DIR"), "/sdf.rs"));

/*
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct SdfPlugin {}
*/

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum ElementData {
    #[serde(rename = "$text")]
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

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "name")]
pub struct XmlElement {
    #[serde(flatten)]
    pub attributes: HashMap<String, String>,
    pub name: String,
    #[serde(flatten)]
    #[serde(deserialize_with = "deserialize_element_data")]
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

// Custom deserializer for ValueEnum
fn deserialize_element_data<'de, D>(deserializer: D) -> Result<ElementData, D::Error>
where
    D: Deserializer<'de>,
{
    struct ElementDataVisitor;

    impl<'de> Visitor<'de> for ElementDataVisitor {
        type Value = ElementData;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a ElementData object")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error
        {
            Ok(ElementData::String(value.to_string()))
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            if let Ok(nested_element) = map.next_value::<ElementMap>() {
                return Ok(ElementData::Nested(nested_element));
            }
            Err(A::Error::missing_field("nested"))
        }
    }

    deserializer.deserialize_any(ElementDataVisitor)
}


#[derive(Default, PartialEq, Clone, Debug, Serialize)]
pub struct ElementMap {
    #[serde(skip)]
    indexes: HashMap<String, BTreeSet<usize>>,
    #[serde(flatten)]
    elements: Vec<XmlElement>,
}

// Manually declare plugin
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct SdfPlugin {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@filename")]
    pub filename: String,
    #[serde(rename = "$value")]
    pub elements: ElementMap,
}

/*
fn deserialize_element_map<'de, D>(deserializer: D) -> Result<ElementMap, D::Error>
where
    D: Deserialize<'de>
{
    Ok(ElementMap::default())
}
*/


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

/*
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
*/

/*
#[derive(Default)]
struct ElementMapVisitor {}

impl<'de> Visitor<'de> for ElementMapVisitor {
    type Value = ElementMap;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A map with string keys and string values")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = ElementMap::default();
        while let Some((key, value)) = access.next_entry::<String, String>()? {
            dbg!(&(key, value));
            //map.push(key, value);
        }

        Ok(map)
    }

}
*/

impl<'de> Deserialize<'de> for ElementMap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Default)]
        struct ElementMapVisitor(ElementMap);

        impl<'de> Visitor<'de> for ElementMapVisitor {
            type Value = ElementMap;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("A map with string keys and string values")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                dbg!(v);
                panic!("THERE");
                /*
                let mut map = ElementMap::default();
                while let Some((key, value)) = access.next_entry::<String, String>()? {
                    dbg!(&(key, value));
                    //map.push(key, value);
                }
                */
                //Ok(v)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                dbg!(v);
                panic!("HELLO");
                /*
                let mut map = ElementMap::default();
                while let Some((key, value)) = access.next_entry::<String, String>()? {
                    dbg!(&(key, value));
                    //map.push(key, value);
                }
                */
            }

            fn visit_map<M>(mut self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                if let Ok(string_entry) = access.next_entry::<String, String>() {
                    dbg!("Found attribute");
                } else if let Ok(map_entry) = access.next_entry::<String, HashMap<String, String>>() {
                    dbg!("Found map");
                    dbg!(&map_entry);
                } else {
                    dbg!("Unhandled type!");
                }
                /*
                while let Some((key, value)) = access.next_entry::<String, String>()? {
                    dbg!(&(key, value));
                    //map.push(key, value);
                }
                */
                Ok(self.0)
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>
            {

                Ok(self.0)
            }

            //fn visit_struct<

        }

        // Do manual parsing with quick-xml
        dbg!("Deserializing");
        //let s = String::deserialize(deserializer)?;
        let s = deserializer.deserialize_seq(ElementMapVisitor::default())?;
        dbg!("Done");
        Ok(s)
        /*
        dbg!(&s);
        let mut reader = Reader::from_str(&s);
        let mut buf = Vec::new();
        loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                // exits the loop when reaching end of file
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"tag1" => println!("attributes values: {:?}",
                                            e.attributes().map(|a| a.unwrap().value)
                                            .collect::<Vec<_>>()),
                        _ => (),
                    }
                }
                Ok(Event::Text(e)) => println!("Text {}", e.unescape().unwrap()),
                //Ok(Event::Text(e)) => txt.push(e.unescape().unwrap().into_owned()),

                // There are several other `Event`s we do not consider here
                _ => (),
            }
        }
        Ok(Self::default())
        */
        //deserializer.deserialize_map(ElementMapVisitor::default())
        /*
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
        */
    }
}

/*
fn serialize_element<S: SerializeMap>(
    elem: &XmlElement,
    serializer: &mut S,
) -> Result<(), S::Error> {
    // Rename each attribute to prepend a @
    for (name, data) in elem.attributes.iter() {
        let name = String::from("@") + name;
        serializer.serialize_entry(&name, data)?;
    }
    match &elem.data {
        ElementData::String(s) => {
            serializer.serialize_entry("$value", s)?;
        }
        ElementData::Nested(elements) => {
            for element in elements.all().iter() {
                serialize_element(element, serializer)?;
            }
        }
    }
    Ok(())
}

impl Serialize for ElementMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        let mut state = serializer.serialize_map(None)?;
        /*
        state.serialize_entry("@name", &self.name);
        state.serialize_entry("@filename", &self.filename);
        */
        for element in self.elements.iter() {
            serialize_element(element, &mut state)?;
        }
        state.end()
    }
}
*/

// Frame is another wierdo. For some reason it refuses to serialize/deserialize automatically
// Hence the manual definition
// Todo(arjo): Actually implement Frame.
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "frame")]
pub struct SdfFrame {}

// Geometry should really be an enum rather than a list of Options, redefine it here
/// The shape of the visual or collision object.
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "geometry")]
pub enum SdfGeometry {
    #[serde(rename = "empty")]
    #[default]
    Empty,
    #[serde(rename = "box")]
    r#Box(SdfBoxShape),
    #[serde(rename = "capsule")]
    Capsule(SdfCapsuleShape),
    #[serde(rename = "cylinder")]
    Cylinder(SdfCylinderShape),
    #[serde(rename = "ellipsoid")]
    Ellipsoid(SdfEllipsoidShape),
    #[serde(rename = "heightmap")]
    Heightmap(SdfHeightmapShape),
    #[serde(rename = "image")]
    Image(SdfImageShape),
    #[serde(rename = "mesh")]
    Mesh(SdfMeshShape),
    #[serde(rename = "plane")]
    Plane(SdfPlaneShape),
    #[serde(rename = "polyline")]
    Polyline(SdfPolylineShape),
    #[serde(rename = "sphere")]
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

impl<'de> Deserialize<'de> for Vector3d {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = String::deserialize(deserializer)?;
        val.try_into().map_err(|e| D::Error::custom(e))
    }
}

impl Serialize for Vector3d {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        serializer.serialize_str(&format!("{} {} {}", self.0.x, self.0.y, self.0.z))
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

impl<'de> Deserialize<'de> for Vector3i {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = String::deserialize(deserializer)?;
        val.try_into().map_err(|e| D::Error::custom(e))
    }
}

impl Serialize for Vector3i {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
    {
        serializer.serialize_str(&format!("{} {} {}", self.0.x, self.0.y, self.0.z))
    }
}
