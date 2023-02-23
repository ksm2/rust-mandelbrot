use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Color {
    pub const BLACK: Color = Self {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };
    pub const WHITE: Color = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };
    pub const RED: Color = Self {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
    };
    pub const GREEN: Color = Self {
        red: 0.0,
        green: 1.0,
        blue: 0.0,
    };
    pub const BLUE: Color = Self {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
    };
    pub const YELLOW: Color = Self {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
    };
    pub const MAGENTA: Color = Self {
        red: 1.0,
        green: 0.0,
        blue: 1.0,
    };
    pub const CYAN: Color = Self {
        red: 0.0,
        green: 1.0,
        blue: 1.0,
    };

    pub fn new(red: f64, green: f64, blue: f64) -> Self {
        let red = Self::clamp(red);
        let green = Self::clamp(green);
        let blue = Self::clamp(blue);
        Self { red, green, blue }
    }

    fn clamp(val: f64) -> f64 {
        if val < 0.0 {
            0.0
        } else if val > 1.0 {
            1.0
        } else {
            val
        }
    }

    pub fn blend(self: Color, to: Color, value: f64) -> Color {
        self * (1.0 - value) + to * value
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.red + rhs.red,
            self.green + rhs.green,
            self.blue + rhs.blue,
        )
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self::new(self.red * rhs, self.green * rhs, self.blue * rhs)
    }
}

impl From<Color> for u32 {
    fn from(c: Color) -> Self {
        let red = (c.red * 255.0) as u32;
        let green = (c.green * 255.0) as u32;
        let blue = (c.blue * 255.0) as u32;

        red + (green << 8) + (blue << 16)
    }
}
