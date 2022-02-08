use glam::{Mat4, Vec2};

pub mod camera;
pub mod geometry;
pub mod texture;
pub mod transform;
pub mod utils;
pub use {
    camera::Camera,
    geometry::*,
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
            position: glam::vec4(100.0, 100.0, 0.0, 1.0),
            color: glam::vec3(0.0, 1.0, 1.0),
            uv: glam::vec2(0.0, 0.0),
        };
        let v1 = Vertex {
            position: glam::vec4(100.0, 400.0, 0.0, 1.0),
            color: glam::vec3(1.0, 0.0, 0.0),
            uv: glam::vec2(0.0, 1.0),
        };

        let interpolated = lerp(v0, v1, 0.5);
        assert_eq!(interpolated.uv.y, 0.5);
    }
}

pub fn raster_clipped_triangle(
    clip_triangle: &Triangle,
    texture: Option<&Texture>,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {
    let rec0 = 1.0 / clip_triangle.v0.position.w;
    let rec1 = 1.0 / clip_triangle.v1.position.w;
    let rec2 = 1.0 / clip_triangle.v2.position.w;

    // This would be the output of the vertex shader (clip space)
    // then we perform perspective division to transform in ndc
    // now x,y,z componend of ndc are between -1 and 1
    let ndc0 = clip_triangle.v0.position * rec0;
    let ndc1 = clip_triangle.v1.position * rec1;
    let ndc2 = clip_triangle.v2.position * rec2;

    // perspective division on all attributes
    let v0 = clip_triangle.v0 * rec0;
    let v1 = clip_triangle.v1 * rec1;
    let v2 = clip_triangle.v2 * rec2;

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

    if let Some(bb) = triangle_screen_bounding_box(&[sc0, sc1, sc2], viewport_size) {
        for y in (bb.top as usize)..=bb.bottom as usize {
            for x in (bb.left as usize)..=bb.right as usize {
                let coords = glam::vec2(x as f32, y as f32) + 0.5;
                let pixel_id = coords_to_index(x, y, viewport_size.x as usize);
                let area = edge_function(sc0, sc1, sc2);

                if let Some(bary) = barycentric_coordinates(coords, sc0, sc1, sc2, area) {
                    let correction = bary.x * rec0 + bary.y * rec1 + bary.z * rec2;
                    let correction = 1.0 / correction;
                    let depth = bary.x * ndc0.z + bary.y * ndc1.z + bary.z * ndc2.z;
                    if depth < z_buffer[pixel_id] {
                        z_buffer[pixel_id] = depth;
                        let color = bary.x * v0.color + bary.y * v1.color + bary.z * v2.color;
                        let color = color * correction;
                        let mut color = to_argb8(
                            255,
                            (color.x * 255.0) as u8,
                            (color.y * 255.0) as u8,
                            (color.z * 255.0) as u8,
                        );
                        if let Some(tex) = texture {
                            let tex_coords = bary.x * v0.uv + bary.y * v1.uv + bary.z * v2.uv;
                            let tex_coords = tex_coords * correction;
                            color = tex.argb_at_uv(tex_coords.x, tex_coords.y);
                        }
                        buffer[pixel_id] = color;
                    }
                }
            }
        }
    }
}

pub fn raster_triangle(
    vertices: &[&Vertex; 3],
    mvp: &Mat4,
    texture: Option<&Texture>,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {
    let triangle = Triangle {
        v0: *vertices[0],
        v1: *vertices[1],
        v2: *vertices[2],
    };
    let clip_tri = triangle.transform(mvp);

    match cull_triangle_view_frustum(&clip_tri) {
        ClipResult::None => {
            println!("fully clipped!");
        }
        ClipResult::One(tri) => {
            raster_clipped_triangle(&tri, texture, buffer, z_buffer, viewport_size);
        }
    }
}

pub fn raster_mesh(
    mesh: &Mesh,
    mvp: &Mat4,
    texture: Option<&Texture>,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {
    for triangle in mesh.triangles() {
        let vertices = mesh.get_vertices_from_triangle(*triangle);
        raster_triangle(&vertices, mvp, texture, buffer, z_buffer, viewport_size);
    }
}

// this takes care of raster clipping
pub fn triangle_screen_bounding_box(
    positions: &[Vec2; 3],
    viewport_size: Vec2,
) -> Option<BoundingBox2D> {
    let bb = get_triangle_bounding_box_2d(positions);

    if bb.left >= viewport_size.x || bb.right < 0.0 || bb.bottom >= viewport_size.y || bb.top < 0.0
    {
        None
    } else {
        let left = bb.left.max(0.0);
        let right = bb.right.min(viewport_size.x - 1.0);
        let bottom = bb.bottom.max(0.0);
        let top = bb.top.min(viewport_size.y - 1.0);

        Some(BoundingBox2D {
            left,
            right,
            top,
            bottom,
        })
    }
}

pub enum ClipResult {
    None,
    One(Triangle),
}

//View Frustum Culling
pub fn cull_triangle_view_frustum(triangle: &Triangle) -> ClipResult {
    // cull tests against the 6 planes
    if triangle.v0.position.x > triangle.v0.position.w
        && triangle.v1.position.x > triangle.v1.position.w
        && triangle.v2.position.x > triangle.v2.position.w
    {
        return ClipResult::None;
    }
    if triangle.v0.position.x < -triangle.v0.position.w
        && triangle.v1.position.x < -triangle.v1.position.w
        && triangle.v2.position.x < -triangle.v2.position.w
    {
        return ClipResult::None;
    }
    if triangle.v0.position.y > triangle.v0.position.w
        && triangle.v1.position.y > triangle.v1.position.w
        && triangle.v2.position.y > triangle.v2.position.w
    {
        return ClipResult::None;
    }
    if triangle.v0.position.y < -triangle.v0.position.w
        && triangle.v1.position.y < -triangle.v1.position.w
        && triangle.v2.position.y < -triangle.v2.position.w
    {
        return ClipResult::None;
    }
    if triangle.v0.position.z > triangle.v0.position.w
        && triangle.v1.position.z > triangle.v1.position.w
        && triangle.v2.position.z > triangle.v2.position.w
    {
        return ClipResult::None;
    }
    if triangle.v0.position.z < 0.0 && triangle.v1.position.z < 0.0 && triangle.v2.position.z < 0.0
    {
        return ClipResult::None;
    }

    ClipResult::One(*triangle)
}
