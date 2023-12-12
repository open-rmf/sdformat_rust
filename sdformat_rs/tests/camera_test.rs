use nalgebra::Vector;
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

use sdformat_rs::{SdfLight, Vector3d};
#[test]
fn test_light_direction_pose() {
    let light = SdfLight {
        direction: Vector3d::new(4.0, 5.0, 6.0),
        ..Default::default()
    };
    let serialized = yaserde::ser::to_string(&light);
    dbg!(&serialized);
    panic!();
}
