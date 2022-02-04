//with extern we can make it availble everywhere
use glam::Vec2;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

//accessible from other modules
pub mod utils;
pub use utils::*;

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

    let triangle = [
        glam::vec2(100.0, 100.0),
        glam::vec2(250.0, 400.0),
        glam::vec2(400.0, 100.0),
    ];

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    //let edge = (glam::vec2(0.0, 0.0), Vec2::new(width as f32, HEIGHT as f32));

    for (i, pixel) in buffer.iter_mut().enumerate() {
        let coords = index_to_coords(i, WIDTH);
        // shadowing a variable
        let coords = glam::vec2(coords.0 as f32, coords.1 as f32);
        let m2 = edge_function(coords, triangle[0], triangle[1]);

        let m0 = edge_function(coords, triangle[1], triangle[2]);
        let m1 = edge_function(coords, triangle[2], triangle[0]);
        // if m0 & m1 & m2 >= 0 we are inside the triangle
        if m0 >= 0.0 && m1 >= 0.0 && m2 >= 0.0 {
            *pixel = to_argb8(255, (m2 * 255.0) as u8, 0, 0);
        } else {
            *pixel = 0;
        }
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
