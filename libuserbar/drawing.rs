pub struct Canvas {
    width: usize,
    height: usize,
    data: Vec<f32>,
}

// color with alpha, floating. linear colorspace. not premultiplied.
#[derive(Clone, Copy)]
pub struct ColorAF(f32, f32, f32, f32);

impl ColorAF {
    pub fn from_f_srgb(r: f32, g: f32, b: f32, a: f32) -> Self {
        fn gamma(x: f32) -> f32 {
            // return x; // linear for testing
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        }
        Self(gamma(r), gamma(g), gamma(b), a)
    }
    pub fn from_srgb(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_f_srgb(
            r as f32 / 255.,
            g as f32 / 255.,
            b as f32 / 255.,
            a as f32 / 255.,
        )
    }
    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self(r, g, b, 1.0)
    }
}
impl std::ops::Add for ColorAF {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
            self.3 + rhs.3,
        )
    }
}
impl std::ops::Mul<f32> for ColorAF {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs, self.3 * rhs)
    }
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0.; width * height * 3],
        }
    }

    pub fn draw_px(&mut self, x: usize, y: usize, c: ColorAF) {
        let ind = (y * self.width + x) * 3;
        let col = ColorAF::from_f32(self.data[ind], self.data[ind + 1], self.data[ind + 2]);
        let old_alpha = 1. - c.3;
        let new_alpha = c.3;
        let new_col = (col * old_alpha) + c * new_alpha;
        self.data[ind + 0] = new_col.0;
        self.data[ind + 1] = new_col.1;
        self.data[ind + 2] = new_col.2;
    }

    pub fn horz_line(&mut self, start: usize, end: usize, y: usize, c: ColorAF) {
        for i in start..=end {
            self.draw_px(i, y, c)
        }
    }
    pub fn vert_line(&mut self, start: usize, end: usize, x: usize, c: ColorAF) {
        for i in start..=end {
            self.draw_px(x, i, c)
        }
    }

    pub fn vert_gradient(
        &mut self,
        startx: usize,
        endx: usize,
        starty: usize,
        endy: usize,
        startc: crate::Color,
        endc: crate::Color,
    ) {
        for y in starty..=endy {
            // how much of the start color and how much of the end color to take
            let denom = endy - starty;
            let frac_e = (y - starty) as f32 / denom as f32;
            let frac_s = 1. - frac_e;
            // this could use oklab or whatever Fancy Color Space in theory
            let mix = |x: u8, y: u8| x as f32 / 255. * frac_s + y as f32 / 255. * frac_e;
            let col = ColorAF::from_f_srgb(
                mix(startc.0, endc.0),
                mix(startc.1, endc.1),
                mix(startc.2, endc.2),
                1.,
            );

            self.horz_line(startx, endx, y, col);
        }
    }

    pub fn ellipse(&mut self, centerx: f32, centery: f32, a: f32, b: f32, col: ColorAF) {
        let a2 = (a * a).recip();
        let b2 = (b * b).recip();
        for x in 0..self.width {
            for y in 0..self.height {
                let dx = (x as f32) - centerx;
                let dy = (y as f32) - centery;
                // optimization to not do subsampling except when on the edge
                // which quadrant are we in?
                let off_x = if dx < 0. { 1 } else { -1 };
                let off_y = if dy < 0. { 1 } else { -1 };
                let outer_corner_x = dx - (3 * off_x) as f32 / 7.;
                let outer_corner_y = dy - (3 * off_y) as f32 / 7.;
                // if we are fully inside the ellipse:
                if outer_corner_x.powi(2) * a2 + outer_corner_y.powi(2) * b2 <= 1. {
                    self.draw_px(x, y, col);
                    continue;
                }
                let inner_corner_x = dx + (3 * off_x) as f32 / 7.;
                let inner_corner_y = dy + (3 * off_y) as f32 / 7.;
                // if we are fully outside the ellipse:
                if inner_corner_x.powi(2) * a2 + inner_corner_y.powi(2) * b2 > 1. {
                    continue;
                }

                // otherwise, subsample.
                let mut num_inside = 0_usize;
                for sub_x in -3..=3 {
                    for sub_y in -3..=3 {
                        let nx = dx + sub_x as f32 / 7.;
                        let ny = dy + sub_y as f32 / 7.;
                        if nx * nx * a2 + ny * ny * b2 <= 1. {
                            num_inside += 1;
                        }
                    }
                }
                let new_alpha = num_inside as f32 / 49.;
                let mut new_col = col;
                new_col.3 *= new_alpha;
                self.draw_px(x, y, new_col);
            }
        }
    }

    pub fn get_buf(self) -> Vec<u8> {
        fn degamma(x: f32) -> u8 {
            let corrected = if x <= 0.0031308 {
                x * 12.92
            } else {
                1.055 * (x.powf(1. / 2.4)) - 0.055
            };
            // let corrected = x; // linear for testing
            (corrected * 255.) as u8
        }
        self.data.into_iter().map(degamma).collect()
    }
}
