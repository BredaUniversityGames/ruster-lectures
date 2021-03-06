use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};
use std::path::Path;
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
            normal: glam::vec3(0.0, 0.0, 1.0),
            color: glam::vec3(0.0, 1.0, 1.0),
            uv: glam::vec2(0.0, 0.0),
        };
        let v1 = Vertex {
            position: glam::vec4(100.0, 400.0, 0.0, 1.0),
            normal: glam::vec3(0.0, 0.0, 1.0),
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
                        let normal = bary.x * v0.normal + bary.y * v1.normal + bary.z * v2.normal;
                        let normal = normal * correction;
                        let n_dot_l = normal.dot(Vec3::ONE.normalize());
                        let color = bary.x * v0.color + bary.y * v1.color + bary.z * v2.color;
                        let mut color = color * correction;
                        if let Some(tex) = texture {
                            let tex_coords = bary.x * v0.uv + bary.y * v1.uv + bary.z * v2.uv;
                            let tex_coords = tex_coords * correction;
                            color = tex.argb_at_uvf(tex_coords.x, tex_coords.y).yzw();
                        }
                        let ambient = glam::vec3(0.2, 0.2, 0.2);
                        color = color * n_dot_l + ambient;
                        let out_color = to_argb8(
                            255,
                            (color.x * 255.0) as u8,
                            (color.y * 255.0) as u8,
                            (color.z * 255.0) as u8,
                        );
                        buffer[pixel_id] = out_color;
                    }
                }
            }
        }
    }
}

pub fn raster_triangle(
    vertices: &[&Vertex; 3],
    model: &Mat4,
    mvp: &Mat4,
    texture: Option<&Texture>,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {
    let cof_mat = cofactor(model);
    let triangle = Triangle {
        v0: *vertices[0],
        v1: *vertices[1],
        v2: *vertices[2],
    };
    let mut clip_tri = triangle.transform(mvp);
    clip_tri.v0.normal = (cof_mat * clip_tri.v0.normal.extend(0.0)).xyz();
    clip_tri.v1.normal = (cof_mat * clip_tri.v1.normal.extend(0.0)).xyz();
    clip_tri.v2.normal = (cof_mat * clip_tri.v2.normal.extend(0.0)).xyz();

    match clip_cull_triangle(&clip_tri) {
        ClipResult::None => {}
        ClipResult::One(tri) => {
            raster_clipped_triangle(&tri, texture, buffer, z_buffer, viewport_size);
        }
        ClipResult::Two(tri) => {
            raster_clipped_triangle(&tri.0, texture, buffer, z_buffer, viewport_size);
            raster_clipped_triangle(&tri.1, texture, buffer, z_buffer, viewport_size);
        }
    }
}

pub fn raster_mesh(
    mesh: &Mesh,
    model: &Mat4,
    mvp: &Mat4,
    texture: Option<&Texture>,
    buffer: &mut Vec<u32>,
    z_buffer: &mut Vec<f32>,
    viewport_size: Vec2,
) {
    for triangle in mesh.triangles() {
        let vertices = mesh.get_vertices_from_triangle(*triangle);
        raster_triangle(
            &vertices,
            model,
            mvp,
            texture,
            buffer,
            z_buffer,
            viewport_size,
        );
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
    Two((Triangle, Triangle)),
}

//View Frustum Culling
pub fn cull_triangle_view_frustum(triangle: &Triangle) -> bool {
    // cull tests against the 6 planes
    if triangle.v0.position.x > triangle.v0.position.w
        && triangle.v1.position.x > triangle.v1.position.w
        && triangle.v2.position.x > triangle.v2.position.w
    {
        return true;
    }
    if triangle.v0.position.x < -triangle.v0.position.w
        && triangle.v1.position.x < -triangle.v1.position.w
        && triangle.v2.position.x < -triangle.v2.position.w
    {
        return true;
    }
    if triangle.v0.position.y > triangle.v0.position.w
        && triangle.v1.position.y > triangle.v1.position.w
        && triangle.v2.position.y > triangle.v2.position.w
    {
        return true;
    }
    if triangle.v0.position.y < -triangle.v0.position.w
        && triangle.v1.position.y < -triangle.v1.position.w
        && triangle.v2.position.y < -triangle.v2.position.w
    {
        return true;
    }
    if triangle.v0.position.z > triangle.v0.position.w
        && triangle.v1.position.z > triangle.v1.position.w
        && triangle.v2.position.z > triangle.v2.position.w
    {
        return true;
    }
    if triangle.v0.position.z < 0.0 && triangle.v1.position.z < 0.0 && triangle.v2.position.z < 0.0
    {
        return true;
    }

    false
}

pub fn clip_triangle_two(triangle: &Triangle) -> (Triangle, Triangle) {
    // calculate alpha values for getting adjusted vertices
    let alpha_a = (-triangle.v0.position.z) / (triangle.v1.position.z - triangle.v0.position.z);
    let alpha_b = (-triangle.v0.position.z) / (triangle.v2.position.z - triangle.v0.position.z);

    // interpolate to get v0a and v0b
    let v0_a = lerp(triangle.v0, triangle.v1, alpha_a);
    let v0_b = lerp(triangle.v0, triangle.v2, alpha_b);

    // draw triangles
    let mut result_a = *triangle;
    let mut result_b = *triangle;

    result_a.v0 = v0_a;

    result_b.v0 = v0_a;
    result_b.v1 = v0_b;

    let green = Vec3::new(0.0, 1.0, 0.0);
    let blue = Vec3::new(0.0, 0.0, 1.0);

    result_a.v0.color = green;
    result_a.v1.color = green;
    result_a.v2.color = green;
    result_b.v0.color = blue;
    result_b.v1.color = blue;
    result_b.v2.color = blue;

    (result_a, result_b)
}

pub fn clip_triangle_one(triangle: &Triangle) -> Triangle {
    // calculate alpha values for getting adjusted vertices
    let alpha_a = (-triangle.v0.position.z) / (triangle.v2.position.z - triangle.v0.position.z);
    let alpha_b = (-triangle.v1.position.z) / (triangle.v2.position.z - triangle.v1.position.z);

    // interpolate to get v0a and v0b
    let mut v0 = lerp(triangle.v0, triangle.v2, alpha_a);
    let mut v1 = lerp(triangle.v1, triangle.v2, alpha_b);

    let mut v2 = triangle.v2;

    let red = Vec3::new(1.0, 0.0, 0.0);

    v0.color = red;
    v1.color = red;
    v2.color = red;

    //println!("out tri: {:?}, {:?}, {:?},", v0, v1, v2);
    // draw triangles
    Triangle { v0, v1, v2 }
}

pub fn cull_triangle_backface(triangle: &Triangle) -> bool {
    let normal = (triangle.v1.position.xyz() - triangle.v0.position.xyz())
        .cross(triangle.v2.position.xyz() - triangle.v0.position.xyz());
    // any is vertex valid
    let view_dir = -Vec3::Z;
    // also we don't care about normalizing
    // if negative facing the camera
    normal.dot(view_dir) >= 0.0
}

pub fn clip_cull_triangle(triangle: &Triangle) -> ClipResult {
    if cull_triangle_backface(triangle) {
        return ClipResult::None;
    }
    if cull_triangle_view_frustum(triangle) {
        ClipResult::None
    } else {
        // clipping routines
        if triangle.v0.position.z < 0.0 {
            if triangle.v1.position.z < 0.0 {
                ClipResult::One(clip_triangle_one(triangle))
            } else if triangle.v2.position.z < 0.0 {
                ClipResult::One(clip_triangle_one(&triangle.reorder(VerticesOrder::ACB)))
            } else {
                ClipResult::Two(clip_triangle_two(&triangle.reorder(VerticesOrder::ACB)))
            }
        } else if triangle.v1.position.z < 0.0 {
            if triangle.v2.position.z < 0.0 {
                ClipResult::One(clip_triangle_one(&triangle.reorder(VerticesOrder::BCA)))
            } else {
                ClipResult::Two(clip_triangle_two(&triangle.reorder(VerticesOrder::BAC)))
            }
        } else if triangle.v2.position.z < 0.0 {
            ClipResult::Two(clip_triangle_two(&triangle.reorder(VerticesOrder::CBA)))
        } else {
            // no near clipping necessary
            //return original
            ClipResult::One(*triangle)
        }
    }
}

pub fn load_gltf(path: &Path) -> Mesh {
    // handle loading textures, cameras, meshes here
    let (document, buffers, _images) = gltf::import(path).unwrap();

    for scene in document.scenes() {
        for node in scene.nodes() {
            println!(
                "Node #{} has {} children, camera: {:?}, mesh: {:?}, transform: {:?}",
                node.index(),
                node.children().count(),
                node.camera(),
                node.mesh().is_some(),
                node.transform(),
            );
            println!(
                "Node #{} has transform: trans {:?}, rot {:?}, scale {:?},",
                node.index(),
                node.transform().decomposed().0,
                node.transform().decomposed().1,
                node.transform().decomposed().2,
            );
            if let Some(mesh) = node.mesh() {
                return Mesh::load_from_gltf(&mesh, &buffers);
            }
        }
    }

    Mesh::new()
}
