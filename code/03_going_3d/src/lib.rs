use glam::{Vec2, Vec3Swizzles};

pub mod geometry;
pub mod texture;
pub mod transform;
pub mod utils;
pub use {geometry::Vertex, texture::Texture, transform::Transform, utils::*};

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
    window_size: Vec2,
) {
    for (i, pixel) in buffer.iter_mut().enumerate() {
        let coords = index_to_coords(i, window_size.y as usize);
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
