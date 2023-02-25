use crate::bitmap::Bitmap;
use crate::color::Color;
use crate::complex::Complex;
use crate::dim::Dim;
use rayon::prelude::*;

#[allow(dead_code)]
pub fn calculate_pixels<B: Bitmap>(bmp: &mut B) {
    let dim = bmp.dim();
    dim.into_iter()
        .map(|(x, y)| calculate_pixel(&dim, x, y))
        .for_each(|(x, y, color)| bmp.set_pixel(x, y, color));
}

pub fn calculate_pixels_par<B: Bitmap>(bmp: &mut B) {
    let dim = bmp.dim();
    dim.into_par_iter()
        .map(|(x, y)| calculate_pixel(&dim, x, y))
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|(x, y, color)| bmp.set_pixel(x, y, color));
}

fn calculate_pixel(dim: &Dim, x: i32, y: i32) -> (i32, i32, Color) {
    let c = translate(dim.width, dim.height, x, y);
    let mut z = c;

    let max_iter = 400;
    let mut iter = 0;
    let mut inside = true;
    loop {
        let new_z = z * z + c;
        if new_z.abs() >= 2.0 {
            inside = false;
            break;
        }

        z = new_z;
        if iter > max_iter {
            break;
        }
        iter += 1;
    }

    let color = if inside {
        Color::BLACK
    } else {
        color_iter(iter)
    };

    (x, y, color)
}

fn translate(width: i32, height: i32, x: i32, y: i32) -> Complex {
    let third_width = width / 3;
    let half_height = height / 2;

    let unit = i32::min(third_width, half_height) as f64;

    let value_x = (x - third_width - third_width) as f64 / unit;
    let value_y = (y - half_height) as f64 / unit;

    Complex::new(value_x, value_y)
}

fn color_iter(iter: i32) -> Color {
    if iter <= 100 {
        let it_rel = (iter as f64) / 100.0;
        Color::blend(Color::BLUE, Color::WHITE, it_rel)
    } else if iter <= 200 {
        let it_rel = (iter - 100) as f64 / 100.0;
        Color::blend(Color::WHITE, Color::RED, it_rel)
    } else if iter <= 300 {
        let it_rel = (iter - 200) as f64 / 100.0;
        Color::blend(Color::RED, Color::YELLOW, it_rel)
    } else {
        let it_rel = (iter - 300) as f64 / 100.0;
        Color::blend(Color::YELLOW, Color::CYAN, it_rel)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use crate::bitmap::InMemBitmap;
    use test::Bencher;

    #[bench]
    fn bench_pixel_calc(b: &mut Bencher) {
        let mut bmp = InMemBitmap::<60, 40>::new();
        b.iter(|| calculate_pixels(&mut bmp))
    }

    #[bench]
    fn bench_pixel_calc_par(b: &mut Bencher) {
        let mut bmp = InMemBitmap::<60, 40>::new();
        b.iter(|| calculate_pixels_par(&mut bmp))
    }
}
