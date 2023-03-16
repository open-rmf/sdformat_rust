use sdformat_rs::camera;

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
}