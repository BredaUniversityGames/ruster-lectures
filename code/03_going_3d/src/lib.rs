use glam::{Mat4, Vec2, Vec4};

pub mod camera;
pub mod geometry;
pub mod texture;
pub mod transform;
pub mod utils;
pub use {
    camera::Camera,
    geometry::{Mesh, Vertex},
    texture::Texture,
    transform::{Transform, TransformInitialParams},
    utils::*,
};

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
    model: &Mat4,
    view: &Mat4,
    projection: &Mat4,
    texture: Option<&Texture>,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {
    let mvp = *projection * *view * *model;

    let clip0 = mvp * Vec4::from((v0.position, 1.0));
    let clip1 = mvp * Vec4::from((v1.position, 1.0));
    let clip2 = mvp * Vec4::from((v2.position, 1.0));

    let rec0 = 1.0 / clip0.w;
    let rec1 = 1.0 / clip1.w;
    let rec2 = 1.0 / clip2.w;

    let uv0 = v0.uv * rec0;
    let uv1 = v1.uv * rec1;
    let uv2 = v2.uv * rec2;

    let color0 = v0.color * rec0;
    let color1 = v1.color * rec1;
    let color2 = v2.color * rec2;

    // This would be the output of the vertex shader (clip space)
    // then we perform perspective division to transform in ndc
    // now x,y,z componend of ndc are between -1 and 1
    let ndc0 = clip0 * rec0;
    let ndc1 = clip1 * rec1;
    let ndc2 = clip2 * rec2;

    // screeen coordinates remapped to window
    let sc0 = glam::vec2(
        map_to_range(ndc0.x, -1.0, 1.0, 0.0, viewport_size.x),
        map_to_range(-ndc0.y, -1.0, 1.0, 0.0, viewport_size.y),
    );
    let sc1 = glam::vec2(
        map_to_range(ndc1.x, -1.0, 1.0, 0.0, viewport_size.x),
        map_to_range(-ndc1.y, -1.0, 1.0, 0.0, viewport_size.y),
    );
    let sc2 = glam::vec2(
        map_to_range(ndc2.x, -1.0, 1.0, 0.0, viewport_size.x),
        map_to_range(-ndc2.y, -1.0, 1.0, 0.0, viewport_size.y),
    );

    for (i, pixel) in buffer.iter_mut().enumerate() {
        let coords = index_to_coords(i, viewport_size.y as usize);
        // center of the pixel
        let coords = glam::vec2(coords.0 as f32, coords.1 as f32) + 0.5;

        let area = edge_function(sc0, sc1, sc2);

        if let Some(bary) = barycentric_coordinates(coords, sc0, sc1, sc2, area) {
            let correction = bary.x * rec0 + bary.y * rec1 + bary.z * rec2;
            let correction = 1.0 / correction;
            let depth = bary.x * ndc0.z + bary.y * ndc1.z + bary.z * ndc2.z;
            if depth < z_buffer[i] {
                z_buffer[i] = depth;
                let color = bary.x * color0 + bary.y * color1 + bary.z * color2;
                let color = color * correction;
                let mut color = to_argb8(
                    255,
                    (color.x * 255.0) as u8,
                    (color.y * 255.0) as u8,
                    (color.z * 255.0) as u8,
                );
                if let Some(tex) = texture {
                    let tex_coords = bary.x * uv0 + bary.y * uv1 + bary.z * uv2;
                    let tex_coords = tex_coords * correction;

                    color = tex.argb_at_uv(tex_coords.x, tex_coords.y);
                }

                *pixel = color;
            }
        }
    }
}

pub fn raster_mesh(
    mesh: &Mesh,
    model: &Mat4,
    view: &Mat4,
    projection: &Mat4,
    texture: Option<&Texture>,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {
    for triangle in mesh.triangles() {
        let vertices = mesh.get_vertices_from_triangle(*triangle);
        raster_triangle(
            *vertices[0],
            *vertices[1],
            *vertices[2],
            model,
            view,
            projection,
            texture,
            buffer,
            z_buffer,
            viewport_size,
        );
    }
}
