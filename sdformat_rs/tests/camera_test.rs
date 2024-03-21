use yaserde::de::from_str;

use sdformat_rs::SdfCamera;

#[test]
fn test_camera_fragment() {
    use yaserde::de::from_str;
    let test_syntax = r#"<camera>
            <horizontal_fov>1.047</horizontal_fov>
            <image>
                <width>320</width>
                <height>240</height>
            </image>
            <clip>
                <near>0.1</near>
                <far>100</far>
            </clip>
        </camera>"#;
    let fr = from_str::<SdfCamera>(test_syntax);
    assert!(matches!(fr, Ok(_)));
}

use sdformat_rs::SdfPose;
#[test]
fn test_pose_fragment() {
    let test_syntax = "<pose>1 0 0 0 0 0</pose>";
    let fr = from_str::<SdfPose>(test_syntax);
    assert!(matches!(fr, Ok(_)));

    if let Ok(pose) = fr {
        let pose = pose.get_pose();
        assert!(matches!(pose, Ok(_)));
        assert_eq!(pose.unwrap().translation, Vector3::new(1.0, 0.0, 0.0));
    }
}

use nalgebra::Vector3;
use sdformat_rs::SdfBoxShape;
#[test]
fn test_box_fragment() {
    let test_syntax = "<box><size>0 0 1</size></box>";
    let fr = from_str::<SdfBoxShape>(test_syntax);
    assert!(matches!(fr, Ok(_)));

    if let Ok(box_shape) = fr {
        assert!(
            (box_shape.size.0 - Vector3::<f64>::new(0.0, 0.0, 1.0))
                .norm()
                .abs()
                < 0.000001
        );
    }
}

use sdformat_rs::SdfGeometry;
#[test]
fn test_geometry_enum() {
    let test_syntax = "<geometry><box><size>0 0 1</size></box></geometry>";
    let fr = from_str::<SdfGeometry>(test_syntax);
    assert!(matches!(fr, Ok(_)));
    assert!(matches!(fr.unwrap(), SdfGeometry::Box(_)));
}

use sdformat_rs::{ElementData, SdfPlugin};
#[test]
fn test_plugin() {
    let test_plugin_content = |fr: &SdfPlugin| {
        assert_eq!(fr.name, "hello");
        assert_eq!(fr.filename, "world.so");
        assert_eq!(fr.elements.all().len(), 1);
        let box_elem = fr.elements.all().iter().next().unwrap();
        assert_eq!(&*box_elem.name, "box");
        assert_eq!(box_elem.attributes.len(), 1);
        let (attr_name, attr_value) = box_elem.attributes.iter().next().unwrap();
        assert_eq!((attr_name, attr_value), (&"name".into(), &"boxy".into()));
        match &box_elem.data {
            ElementData::Nested(data) => {
                assert_eq!(data.all().len(), 1);
                let size_elem = data.all().iter().next().unwrap();
                assert_eq!(&*size_elem.name, "size");
                assert_eq!(size_elem.data.clone().try_into(), Ok(42));
            }
            _ => panic!("Expected nested element"),
        }
    };
    let test_syntax = "<plugin name=\"hello\" filename=\"world.so\"><box name=\"boxy\"><size>42</size><!-- A comment --></box></plugin>";
    let fr = from_str::<SdfPlugin>(test_syntax).unwrap();
    test_plugin_content(&fr);
    // Serialize back
    let to = yaserde::ser::to_string(&fr);
    // Deserialize again and check that it's OK
    let fr = from_str::<SdfPlugin>(test_syntax).unwrap();
    test_plugin_content(&fr);
    assert!(to.is_ok());

    // Test accessing and mutating API
    let test_syntax = "<plugin name=\"hello\" filename=\"world.so\"><size>42</size></plugin>";
    let mut plugin = from_str::<SdfPlugin>(test_syntax).unwrap();
    let size = plugin.elements.get("size").unwrap();
    assert_eq!(size.data, ElementData::String("42".to_string()));
    let size = plugin.elements.get_mut("size").unwrap();
    size.data = ElementData::String("hello".to_string());

    let size = plugin.elements.get("size").unwrap();
    assert_eq!(size.data, ElementData::String("hello".to_string()));
    // test for_each and for_each_mut
    let test_syntax = "<plugin name=\"hello\" filename=\"world.so\"><box name=\"boxy\"></box><box name=\"boxy\"></box></plugin>";
    let mut plugin = from_str::<SdfPlugin>(test_syntax).unwrap();
    plugin.elements.for_each("box", |elem| {
        assert_eq!(elem.attributes.values().next(), Some(&"boxy".to_string()));
    });
    plugin.elements.for_each_mut("box", |elem| {
        elem.attributes
            .insert("hello".to_string(), "world".to_string());
    });
    plugin.elements.for_each("box", |elem| {
        assert_eq!(elem.attributes.get("hello"), Some(&"world".to_string()));
    });
}

use sdformat_rs::SdfLight;
#[test]
fn test_light_direction_pose_serdeser() {
    let test_syntax = "<?xml version=\"1.0\" encoding=\"utf-8\"?><light name=\"test\" type=\"point\"><direction>0 0 1</direction></light>";
    let fr = from_str::<SdfLight>(test_syntax);
    let serialized = yaserde::ser::to_string(&fr.unwrap()).unwrap();
    assert_eq!(test_syntax.to_string(), serialized);
}

use sdformat_rs::SdfModel;
#[test]
fn test_nested_model() {
    let test_syntax = "<?xml version=\"1.0\" encoding=\"utf-8\"?><model name=\"top\"><model name=\"nested\" /></model>";
    let fr = from_str::<SdfModel>(test_syntax);
    let serialized = yaserde::ser::to_string(&fr.unwrap()).unwrap();
    assert_eq!(test_syntax.to_string(), serialized);
}
