use minifb::{Key, Window, WindowOptions};
use std::path::Path;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

use ruster::*;

pub fn process_input_camera(window: &Window, camera: &mut Camera) {
    let mut axis = glam::vec2(0.0, 0.0);
    // we will make registering later

    if window.is_key_down(Key::A) {
        axis.x -= 1.0;
    }
    if window.is_key_down(Key::D) {
        axis.x += 1.0;
    }
    if window.is_key_down(Key::W) {
        axis.y += 1.0;
    }
    if window.is_key_down(Key::S) {
        axis.y -= 1.0;
    }
    camera.transform.translation += camera.transform.right() * camera.speed * axis.x
        + camera.transform.forward() * camera.speed * axis.y;
    //camera.transform.translation += Vec3::new(axis.x, 0.0, axis.y) * camera.speed;
}

fn main() {
    let mut window = Window::new(
        "Going 3D - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    //https://github.com/KhronosGroup/glTF-Sample-Models
    let texture = Texture::load(Path::new("../../assets/damagedhelmet/Default_albedo.jpg"));
    let mesh = load_gltf(Path::new("../../assets/damagedhelmet/damagedhelmet.gltf"));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut z_buffer = vec![f32::INFINITY; WIDTH * HEIGHT];
    let window_size = glam::vec2(WIDTH as f32, HEIGHT as f32);

    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;

    let mut camera = Camera {
        aspect_ratio,
        transform: Transform::from_translation(glam::vec3(0.0, 0.0, 8.0)),
        frustum_near: 4.0,
        frustum_far: 100.0,
        ..Default::default()
    };

    let mut rot = std::f32::consts::FRAC_PI_4;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        clear_buffer(&mut buffer, 0);
        clear_buffer(&mut z_buffer, f32::INFINITY);
        process_input_camera(&window, &mut camera);

        let parent_local =
            Transform::from_rotation(glam::Quat::from_euler(glam::EulerRot::XYZ, rot, 0.0, 0.0))
                .local();
        let view = camera.view();
        let proj = camera.projection();

        raster_mesh(
            &mesh,
            &parent_local,
            &(proj * view * parent_local),
            Some(&texture),
            &mut buffer,
            &mut z_buffer,
            window_size,
        );
        rot += 0.05;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
