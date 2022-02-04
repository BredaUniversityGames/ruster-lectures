use super::*;
// it is common to write unit testing in the module itself, but we can
// either create a different file and tag it
// or add files to /tests folder next to /src (this is done for libs)
#[test]
fn lerping() {
    let v0 = Vertex {
        position: glam::vec3(100.0, 100.0, 0.0),
        color: glam::vec3(0.0, 1.0, 1.0),
        uv: glam::vec2(0.0, 0.0),
    };
    let v1 = Vertex {
        position: glam::vec3(100.0, 400.0, 0.0),
        color: glam::vec3(1.0, 0.0, 0.0),
        uv: glam::vec2(0.0, 1.0),
    };
    let interpolated = utils::lerp(v0, v1, 0.5);
    assert_eq!(interpolated.uv.y, 0.5);
}
