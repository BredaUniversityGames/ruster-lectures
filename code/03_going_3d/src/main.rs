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
        position: glam::vec3(-1.0, -1.0, 1.0),
        color: glam::vec3(0.0, 1.0, 1.0),
        uv: glam::vec2(0.0, 1.0),
    };
    let v1 = Vertex {
        position: glam::vec3(-1.0, 1.0, 1.0),
        color: glam::vec3(1.0, 0.0, 0.0),
        uv: glam::vec2(0.0, 0.0),
    };
    let v2 = Vertex {
        position: glam::vec3(1.0, 1.0, 1.0),
        color: glam::vec3(0.0, 1.0, 0.0),
        uv: glam::vec2(1.0, 0.0),
    };
    let v3 = Vertex {
        position: glam::vec3(1.0, -1.0, 1.0),
        color: glam::vec3(0.0, 1.0, 1.0),
        uv: glam::vec2(1.0, 1.0),
    };

    let triangles = vec![glam::uvec3(2, 1, 0), glam::uvec3(3, 2, 0)];
    let vertices = vec![v0, v1, v2, v3];

    let mesh = Mesh::from_vertices(&triangles, &vertices);

    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;

    let camera = Camera {
        aspect_ratio,
        transform: Transform::from_translation(glam::vec3(0.0, 0.0, 8.0)),
        frustum_far: 100.0,
        ..Default::default()
    };
    //+z
    let transform0 = Transform::IDENTITY;
    //-z
    let transform1 = Transform::from_rotation(glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        -std::f32::consts::PI,
        0.0,
        0.0,
    ));
    //+y
    let transform2 = Transform::from_rotation(glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        std::f32::consts::FRAC_PI_2,
        0.0,
        0.0,
    ));
    //-y
    let transform3 = Transform::from_rotation(glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        -std::f32::consts::FRAC_PI_2,
        0.0,
        0.0,
    ));
    //+x
    let transform4 = Transform::from_rotation(glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        0.0,
        -std::f32::consts::FRAC_PI_2,
        0.0,
    ));
    //-x
    let transform5 = Transform::from_rotation(glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        0.0,
        std::f32::consts::FRAC_PI_2,
        0.0,
    ));

    let mut rot = std::f32::consts::FRAC_PI_4;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        clear_buffer(&mut buffer, 0);
        clear_buffer(&mut z_buffer, f32::INFINITY);
        let parent_local =
            Transform::from_rotation(glam::Quat::from_euler(glam::EulerRot::XYZ, rot, 0.0, 0.0))
                .local();

        raster_mesh(
            &mesh,
            &(parent_local * transform0.local()),
            &camera.view(),
            &camera.projection(),
            Some(&texture),
            &mut buffer,
            &mut z_buffer,
            window_size,
        );
        raster_mesh(
            &mesh,
            &(parent_local * transform1.local()),
            &camera.view(),
            &camera.projection(),
            Some(&texture),
            &mut buffer,
            &mut z_buffer,
            window_size,
        );
        raster_mesh(
            &mesh,
            &(parent_local * transform2.local()),
            &camera.view(),
            &camera.projection(),
            Some(&texture),
            &mut buffer,
            &mut z_buffer,
            window_size,
        );
        raster_mesh(
            &mesh,
            &(parent_local * transform3.local()),
            &camera.view(),
            &camera.projection(),
            Some(&texture),
            &mut buffer,
            &mut z_buffer,
            window_size,
        );
        raster_mesh(
            &mesh,
            &(parent_local * transform4.local()),
            &camera.view(),
            &camera.projection(),
            Some(&texture),
            &mut buffer,
            &mut z_buffer,
            window_size,
        );
        raster_mesh(
            &mesh,
            &(parent_local * transform5.local()),
            &camera.view(),
            &camera.projection(),
            Some(&texture),
            &mut buffer,
            &mut z_buffer,
            window_size,
        );
        rot += 0.05;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
