extern crate nalgebra as na;
use na::{Vector3, Vector2,
    geometry::{Isometry3, Quaternion}};
use std::collections::HashMap;

enum Value {
    TString(String),
    TBool(bool),
    TInt(i64),
    TDouble(f64)
}


trait TypeSerializationTrait {
    type Item;

    fn keys() -> Vec<(String, String)> {
        vec!()
    }

    fn parse_args(&self, special_values: &HashMap<String, Value>) {

    }

    fn typename<'a>(&self) -> &'a str;

    fn parse(&self, istr:  &str) -> Option<Self::Item>;

    fn to_string(&self, item: &Self::Item) -> String;
}

struct Vec3Serialization {
}

impl TypeSerializationTrait for Vec3Serialization {
    type Item = Vector3<f64>;

    fn typename<'a>(&self) -> &'a str {
        "vector3"
    }

    fn parse(&self, istr:  &str) -> Option<Self::Item> {
        let sz: Vec<&str> = istr.split_whitespace().collect();
        if sz.len() != 3 {
            return None;
        }

        let x = sz[0].parse::<f64>();
        let y = sz[1].parse::<f64>();
        let z = sz[2].parse::<f64>();

        if let Ok(x) = x {
            if let Ok(y) = y {
                if let Ok(z) = z {
                    return Some(Vector3::<f64>::new(x, y, z));
                }
            }
        }

        None
    }

    fn to_string(&self, item: &Self::Item) -> String {
        format!("{} {} {}", item.x, item.y, item.z)
    }
}


struct Vec2Serialization {
}

impl TypeSerializationTrait for Vec2Serialization {
    type Item = Vector2<f64>;

    fn typename<'a>(&self) -> &'a str {
        "vector2d"
    }

    fn parse(&self, istr:  &str) -> Option<Self::Item> {
        let sz: Vec<&str> = istr.split_whitespace().collect();
        if sz.len() != 2 {
            return None;
        }

        let x = sz[0].parse::<f64>();
        let y = sz[1].parse::<f64>();

        if let Ok(x) = x {
            if let Ok(y) = y {
                return Some(Vector2::<f64>::new(x, y));
            }
        }

        None
    }

    fn to_string(&self, item: &Self::Item) -> String {
        format!("{} {}", item.x, item.y)
    }
}

struct Vec2iSerialization {
}

impl TypeSerializationTrait for Vec2iSerialization {
    type Item = Vector2<i64>;

    fn typename<'a>(&self) -> &'a str {
        "vector2i"
    }

    fn parse(&self, istr:  &str) -> Option<Self::Item> {
        let sz: Vec<&str> = istr.split_whitespace().collect();
        if sz.len() != 2 {
            return None;
        }

        let x = sz[0].parse::<i64>();
        let y = sz[1].parse::<i64>();

        if let Ok(x) = x {
            if let Ok(y) = y {
                return Some(Vector2::<i64>::new(x, y));
            }
        }

        None
    }

    fn to_string(&self, item: &Self::Item) -> String {
        format!("{} {}", item.x, item.y)
    }
}


enum QuaternionInputType {
    Degrees,
    Radians
}

struct QuaternionSerialization {
    inputType: QuaternionInputType
}

impl TypeSerializationTrait for QuaternionSerialization {
    type Item = Quaternion<f64>;

    fn keys() -> Vec<(String, String)> {
        vec!(("degrees".to_string(), "bool".to_string()))
    }

    fn typename<'a>(&self) -> &'a str {
        "quaternion"
    }

    fn parse_args(&self, special_values: &HashMap<String, Value>) {
        if let Some(key) = special_values.get("degrees")
        {
           
        }
    }

    fn parse(&self, istr:  &str) -> Option<Self::Item> {
        let sz: Vec<&str> = istr.split_whitespace().collect();
        if sz.len() == 3 {
            // This means euler RPY
            let x = sz[0].parse::<f64>();
            let y = sz[1].parse::<f64>();
            let z = sz[2].parse::<f64>();

            if let Ok(x) = x {
                if let Ok(y) = y {
                    if let Ok(z) = z {
                        match self.inputType {
                            QuaternionInputType::Degrees => {

                            }
                            QuaternionInputType::Radians => {

                            }
                        }
                    }
                }
            }

        }
        else  if sz.len() == 4 {
            let i = sz[0].parse::<f64>();
            let j = sz[1].parse::<f64>();
            let k = sz[2].parse::<f64>();
            let w = sz[3].parse::<f64>();

            if let Ok(i) = i {
                if let Ok(j) = j {
                    if let Ok(k) = k {
                        if let Ok(w) = w {
                            return Some(Quaternion::new(w, i, j, k));
                        }
                    }
                }
            }
        }

        None
    }

    fn to_string(&self, item: &Self::Item) -> String {
        format!("{} {} {} {}", item.i, item.j, item.k, item.w)
    }
}

enum PoseInputType {
    Degrees,
    Radians
}

struct PoseSerialization {
    inputType: PoseInputType
}

impl TypeSerializationTrait for PoseSerialization {
    type Item = Isometry3<f64>;

    fn keys() -> Vec<(String, String)> {
        vec!(("degrees".to_string(), "bool".to_string()))
    }

    fn typename<'a>(&self) -> &'a str {
        "pose"
    }

    fn parse(&self, istr:  &str) -> Option<Self::Item> {
        let sz: Vec<&str> = istr.split_whitespace().collect();
        todo!("Implement it");

        None
    }

    fn to_string(&self, item: &Self::Item) -> String {
        //format!("{} {} {}", item.x, item.y, item.z)
        "".to_string()
    }
}