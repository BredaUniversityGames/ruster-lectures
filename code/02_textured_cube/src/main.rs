use glam::{Vec2, Vec3, Vec3Swizzles};
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

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
}

pub fn raster_triangle(
    v0: Vertex,
    v1: Vertex,
    v2: Vertex,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
) {
    for (i, pixel) in buffer.iter_mut().enumerate() {
        let coords = index_to_coords(i, HEIGHT);
        // shadowing a variable
        let coords = glam::vec2(coords.0 as f32, coords.1 as f32);

        let area = edge_function(v0.position.xy(), v1.position.xy(), v2.position.xy());

        if let Some(bary) = barycentric_coordinates(
            coords,
            v0.position.xy(),
            v1.position.xy(),
            v2.position.xy(),
            area,
        ) {
            let depth = bary.x * v0.position.z + bary.y * v1.position.z + bary.z * v2.position.z;
            if depth < z_buffer[i] {
                z_buffer[i] = depth;
                let color = bary.x * v0.color + bary.y * v1.color + bary.z * v2.color;

                *pixel = to_argb8(
                    255,
                    (color.x * 255.0) as u8,
                    (color.y * 255.0) as u8,
                    (color.z * 255.0) as u8,
                );
            }
        }
    }
}

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

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut z_buffer = vec![f32::INFINITY; WIDTH * HEIGHT];

    let v0 = Vertex {
        position: glam::vec3(100.0, 100.0, 1.0),
        color: glam::vec3(0.0, 1.0, 1.0),
    };
    let v1 = Vertex {
        position: glam::vec3(250.0, 400.0, 1.0),
        color: glam::vec3(0.0, 1.0, 1.0),
    };
    let v2 = Vertex {
        position: glam::vec3(400.0, 100.0, 1.0),
        color: glam::vec3(0.0, 1.0, 1.0),
    };

    let v3 = Vertex {
        position: glam::vec3(0.0, 0.0, 0.0),
        color: glam::vec3(1.0, 1.0, 0.0),
    };
    let v4 = Vertex {
        position: glam::vec3(150.0, 300.0, 0.0),
        color: glam::vec3(1.0, 1.0, 0.0),
    };
    let v5 = Vertex {
        position: glam::vec3(300.0, 0.0, 0.0),
        color: glam::vec3(1.0, 1.0, 0.0),
    };

    raster_triangle(v0, v1, v2, &mut buffer, &mut z_buffer);
    raster_triangle(v3, v4, v5, &mut buffer, &mut z_buffer);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
