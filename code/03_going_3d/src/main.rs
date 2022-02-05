use minifb::{Key, Window, WindowOptions};
use std::path::Path;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

use going_3d::*;

fn main() {
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let texture = Texture::load(Path::new("../../assets/bojan.jpg"));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut z_buffer = vec![f32::INFINITY; WIDTH * HEIGHT];
    let window_size = glam::vec2(WIDTH as f32, HEIGHT as f32);

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
    let v2 = Vertex {
        position: glam::vec3(400.0, 400.0, 0.0),
        color: glam::vec3(0.0, 1.0, 0.0),
        uv: glam::vec2(1.0, 1.0),
    };
    let v3 = Vertex {
        position: glam::vec3(400.0, 100.0, 0.0),
        color: glam::vec3(0.0, 1.0, 1.0),
        uv: glam::vec2(1.0, 0.0),
    };

    let triangles = vec![glam::uvec3(0, 1, 2), glam::uvec3(0, 2, 3)];
    let vertices = vec![v0, v1, v2, v3];

    let mesh = Mesh::from_vertices(&triangles, &vertices);

    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;

    let _camera = Camera {
        aspect_ratio,
        transform: Transform::from_translation(glam::vec3(0.0, 0.0, 500.0)),
        frustum_far: 1000.0,
        ..Default::default()
    };

    raster_mesh(
        &mesh,
        Some(&texture),
        &mut buffer,
        &mut z_buffer,
        window_size,
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
