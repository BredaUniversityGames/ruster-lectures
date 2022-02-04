use glam::Vec2;
//clockwise
pub fn edge_function(v0: Vec2, v1: Vec2, p: Vec2) -> f32 {
    (p.x - v0.x) * (v1.y - v0.y) - (p.y - v0.y) * (v1.x - v0.x)
}

pub fn index_to_coords(p: usize, width: usize) -> (usize, usize) {
    (p % width, p / width)
}

pub fn to_argb8(a: u8, r: u8, g: u8, b: u8) -> u32 {
    let mut argb: u32 = a as u32; //a
    argb = (argb << 8) + r as u32; //r
    argb = (argb << 8) + g as u32; //g
    argb = (argb << 8) + b as u32; //b
    argb
}
