use yaserde::de::from_str;

/*use sdformat_rs::camera;

#[test]
fn test_camera_fragment()
{
    use yaserde::de::from_str;
    let test_syntax =
        r#"<camera>
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
    let fr = from_str::<camera>(test_syntax);
    assert!(matches!(fr, Ok(_)));
}*/

/*use sdformat_rs::pose;
#[test]
fn test_pose_fragment()
{
    let test_syntax = "<pose>0 0 0</pose>";
    let fr = from_str::<pose>(test_syntax);
    assert!(matches!(fr, Ok(_)));

    if let Ok(pose) = fr {
        assert_eq!(pose.get_pose(), "0 0 0".to_string());
    }
}*/
use sdformat_rs::SdfBoxShape;
use nalgebra::Vector3;
#[test]
fn test_camera_fragment()
{
    let test_syntax = "<box><size>0 0 1</size></box>";
    let fr = from_str::<SdfBoxShape>(test_syntax);
    assert!(matches!(fr, Ok(_)));

    if let Ok(box_shape) = fr {
        assert!((box_shape.size.data - Vector3::<f64>::new(0.0,0.0,1.0)).norm().abs() < 0.000001);
    }
}
