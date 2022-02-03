//with extern we can make it availble everywhere
use glam::Vec2;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

//clockwise
pub fn edge_function(v0: Vec2, v1: Vec2, p: Vec2) -> f32 {
    (p.x - v0.x) * (v1.y - v0.y) - (p.y - v0.y) * (v1.x - v0.x)
}

//num items in a row
pub fn index_to_coords(p: usize, width: usize) -> (usize, usize) {
    (p % width, p / width)
}

//https://doc.rust-lang.org/book/ch03-02-data-types.html

pub fn to_argb8(a: u8, r: u8, g: u8, b: u8) -> u32 {
    let mut argb: u32 = a as u32; //a
    argb = (argb << 8) + r as u32; //r
    argb = (argb << 8) + g as u32; //g
    argb = (argb << 8) + b as u32; //b
    argb
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
            *pixel = to_argb8(
                255,
                (m2 * 255.0) as u8,
                0,
                0,
            );
        } else {
            *pixel = 0;
        }
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
