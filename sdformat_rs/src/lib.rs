extern crate yaserde_derive;
use yaserde_derive::{YaSerialize, YaDeserialize};
use yaserde::{YaDeserialize, YaSerialize, ser, de};

include!(concat!(env!("OUT_DIR"), "/sdf.rs"));
