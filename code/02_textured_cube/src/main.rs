use glam::{Vec2, Vec3};
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

//accessible from other modules
pub mod utils;
pub use utils::*;

pub fn barycentric_coordinates(
    point: Vec2,
    v0: Vec2,
    v1: Vec2,
    v2: Vec2,
    area: f32,
) -> Option<Vec3> {
    let m0 = edge_function(point, v1, v2);
    let m1 = edge_function(point, v2, v0);
    let m2 = edge_function(point, v0, v1);
    // instead of 3 divisions we can do 1/area *
    let a = 1.0 / area;
    if m0 >= 0.0 && m1 >= 0.0 && m2 >= 0.0 {
        Some(glam::vec3(m0 * a, m1 * a, m2 * a))
    } else {
        None
    }
}

pub struct Vertex {
    pub position: Vec2,
    pub color: Vec3,
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let v0 = Vertex {
        position: glam::vec2(100.0, 100.0),
        color: glam::vec3(1.0, 0.0, 0.0),
    };
    let v1 = Vertex {
        position: glam::vec2(250.0, 400.0),
        color: glam::vec3(0.0, 1.0, 0.0),
    };
    let v2 = Vertex {
        position: glam::vec2(400.0, 100.0),
        color: glam::vec3(0.0, 0.0, 1.0),
    };

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    for (i, pixel) in buffer.iter_mut().enumerate() {
        let coords = index_to_coords(i, WIDTH);
        let coords = glam::vec2(coords.0 as f32, coords.1 as f32);

        // Triangle area
        let area = edge_function(v0.position, v1.position, v2.position);

        if let Some(bary) =
            barycentric_coordinates(coords, v0.position, v1.position, v2.position, area)
        {
            let color = bary.x * v0.color + bary.y * v1.color + bary.z * v2.color;
            *pixel = to_argb8(
                255,
                (color.x * 255.0) as u8,
                (color.y * 255.0) as u8,
                (color.z * 255.0) as u8,
            );
        } else {
            *pixel = 0;
        }
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
