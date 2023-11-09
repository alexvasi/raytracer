use crate::Color3;
use anyhow::Result;
use glam::UVec2;
use std::path::Path;

pub struct Canvas {
    size: UVec2,
    data: Vec<u8>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        Canvas {
            size: UVec2::new(width, height),
            data: vec![0; width as usize * height as usize * 3],
        }
    }

    pub fn draw(&mut self, x: u32, y: u32, color: Color3) {
        let idx = (y * self.size.x + x) as usize * 3;
        for (num, c) in color.to_array().iter().enumerate() {
            let c = Self::linear_to_gamma_2(*c).clamp(0.0, 1.0);
            self.data[idx + num] = (c * 255.9999) as u8;
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        use std::fs::File;
        use std::io::BufWriter;

        let file = File::create(path)?;
        let w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, self.size.x, self.size.y);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;

        writer.write_image_data(&self.data)?;
        Ok(())
    }

    fn linear_to_gamma_2(component: f32) -> f32 {
        component.sqrt()
    }
}
