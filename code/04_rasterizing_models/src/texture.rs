use crate::utils::*;
use glam::Vec4;
use stb_image;
use std::path::Path;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
    pub depth: usize,
}

impl Texture {
    pub fn load(path: &Path) -> Self {
        let decoded_image = stb_image::image::load(path);
        if let stb_image::image::LoadResult::ImageU8(image) = decoded_image {
            let data;
            if image.depth == 4 {
                data = (0..image.data.len() / 4)
                    .map(|id| {
                        to_argb8(
                            image.data[id * 4 + 3],
                            image.data[id * 4],
                            image.data[id * 4 + 1],
                            image.data[id * 4 + 2],
                        )
                    })
                    .collect();
            } else {
                data = (0..image.data.len() / 3)
                    .map(|id| {
                        to_argb8(
                            255,
                            image.data[id * 3],
                            image.data[id * 3 + 1],
                            image.data[id * 3 + 2],
                        )
                    })
                    .collect();
            }
            Self {
                width: image.width,
                height: image.height,
                data,
                depth: image.depth,
            }
        } else {
            panic!("Unsupported texture type");
        }
    }

    pub fn uv_to_index(&self, u: f32, v: f32) -> usize {
        let (u, v) = (u * self.width as f32, v * self.height as f32);
        coords_to_index(
            (u as usize) % self.width,
            (v as usize) % self.height,
            self.width,
        )
    }

    pub fn argb_at_uv(&self, u: f32, v: f32) -> u32 {
        let id = self.uv_to_index(u, v);
        if id < self.data.len() {
            self.data[id]
        } else {
            to_argb8(255, 255, 0, 255)
        }
    }

    pub fn argb_at_uvf(&self, u: f32, v: f32) -> Vec4 {
        let id = self.uv_to_index(u, v);
        if id < self.data.len() {
            let color = from_argb8(self.data[id]);
            Vec4::new(
                (color.0 as f32) / 255.0,
                (color.1 as f32) / 255.0,
                (color.2 as f32) / 255.0,
                (color.3 as f32) / 255.0,
            )
        } else {
            Vec4::new(1.0, 1.0, 0.0, 1.0)
        }
    }
}
