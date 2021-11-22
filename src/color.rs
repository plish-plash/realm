// TODO use a color library (palette?)

#[derive(Clone, Copy)]
pub struct Color([f32; 4]);

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color([r, g, b, 1.0])
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> [f32; 4] {
        color.0
    }
}