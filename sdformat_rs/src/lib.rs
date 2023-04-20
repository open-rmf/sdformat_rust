extern crate yaserde_derive;
use yaserde_derive::{YaSerialize, YaDeserialize};
use yaserde::{YaDeserialize, YaSerialize, ser, de};
use nalgebra::*;

include!(concat!(env!("OUT_DIR"), "/sdf.rs"));

impl pose {
    pub fn get_pose(&self) -> String
    {
        if let Some(degrees) = self._degrees {
            if degrees {
                // TODO(arjo): Pose parsing code
            }
        }

        self.data.clone()
    }
}
