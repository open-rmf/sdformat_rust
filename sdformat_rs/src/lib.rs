extern crate yaserde_derive;
use std::io::{Read, Write};

use yaserde_derive::{YaSerialize, YaDeserialize};
use yaserde::{YaDeserialize, YaSerialize, ser, de};
use yaserde::xml;
use yaserde::xml::attribute::OwnedAttribute;
use yaserde::xml::namespace::Namespace;
use nalgebra::*;

include!(concat!(env!("OUT_DIR"), "/sdf.rs"));

/*impl pose {
    pub fn get_pose(&self) -> String
    {
        if let Some(degrees) = self._degrees {
            if degrees {
                // TODO(arjo): Pose parsing code
            }
        }

        self.data.clone()
    }
}*/

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vector3d {
    pub data: Vector3<f64>
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
                        return Ok(Vector3d{data: Vector3::new(x, y, z)});
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