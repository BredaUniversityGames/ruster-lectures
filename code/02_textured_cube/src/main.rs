use glam::Vec3Swizzles;
use minifb::{Key, Window, WindowOptions};
use std::path::Path;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

pub mod utils;
pub use utils::*;
pub mod geometry;
pub use geometry::Vertex;
pub mod texture;
pub use texture::Texture;

#[cfg(test)]
mod tests {
    use crate::geometry::Vertex;
    use crate::utils::*;

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

        let interpolated = lerp(v0, v1, 0.5);
        assert_eq!(interpolated.uv.y, 0.5);
    }
}

pub fn raster_triangle(
    v0: Vertex,
    v1: Vertex,
    v2: Vertex,
    texture: &Texture,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
) {
    for (i, pixel) in buffer.iter_mut().enumerate() {
        let coords = index_to_coords(i, HEIGHT);
        // center of the pixel
        let coords = glam::vec2(coords.0 as f32, coords.1 as f32) + 0.5;

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
                //let color = bary.x * v0.color + bary.y * v1.color + bary.z * v2.color;
                let tex_coords = bary.x * v0.uv + bary.y * v1.uv + bary.z * v2.uv;
                let color = texture.argb_at_uv(tex_coords.x, tex_coords.y);

                *pixel = color;
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

    let texture = Texture::load(Path::new("../../assets/bojan.jpg"));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut z_buffer = vec![f32::INFINITY; WIDTH * HEIGHT];

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

    raster_triangle(v0, v1, v2, &texture, &mut buffer, &mut z_buffer);
    raster_triangle(v0, v2, v3, &texture, &mut buffer, &mut z_buffer);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
