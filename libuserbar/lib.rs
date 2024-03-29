mod drawing;
mod font;
mod font_data;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum AxisAnchor {
    Auto,
    Center,
    // leave `offset` pixel gap with left/top edge
    Start,
    // leave `offset` pixel gap with right/bottom edge
    End,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct AxisPlacement {
    pub offset: isize,
    pub anchor: AxisAnchor,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub horz: AxisPlacement,
    pub vert: AxisPlacement,
}

impl AxisPlacement {
    fn to_offset(self, auto_v: AxisPlacement, inner_sz: isize, outer_sz: isize) -> isize {
        let val = if let AxisAnchor::Auto = self.anchor {
            auto_v
        } else {
            self
        };
        match val.anchor {
            AxisAnchor::Auto => panic!(),
            AxisAnchor::Center => (outer_sz - inner_sz) / 2,
            AxisAnchor::Start => val.offset,
            AxisAnchor::End => outer_sz - inner_sz - val.offset,
        }
    }
}

// RGB tuple.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct Color(pub u8, pub u8, pub u8);
// RGBA tuple (not premultiplied).
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct ColorA(pub u8, pub u8, pub u8, pub u8);

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct StripePattern {
    // originally only black.
    pub color: ColorA,
    pub on_main_diagonal: bool,
    // width of the pattern (there are spacing-1 blank pixels between stripes).
    // original generator supported 3 or 4.
    pub spacing: usize,
}

#[wasm_bindgen]
impl StripePattern {
    #[wasm_bindgen(constructor)]
    pub fn new(color: ColorA, on_main_diagonal: bool, spacing: usize) -> Self {
        Self { color, on_main_diagonal, spacing }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct BgImage {
    pub width: usize,
    pub height: usize,
    // data must be bytes in RGBA order, 8 bits per channel.
    pub data: Vec<u8>,
    // placement of image, defaults to top left
    pub placement: Placement,
}

#[wasm_bindgen]
impl BgImage {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize, data: Vec<u8>, placement: Placement) -> Self {
        Self { width, height, data, placement }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct Options {
    pub width: usize,
    pub height: usize,
    pub text: String,
    pub text_placement: Placement,
    pub bg_top_color: Color,
    pub bg_bottom_color: Color,
    pub text_color: ColorA,
    pub text_outline_color: ColorA,
    pub ellipse_color: Option<ColorA>,
    // the old generator rendered the ellipse over the text, but i feel that can make the text less
    // legible.
    pub text_over_ellipse: bool,
    pub border_color: Option<ColorA>,
    // these were called "scanlines" in the original generator.
    pub diag_stripes: Option<StripePattern>,
    pub bg_image: Option<BgImage>,
}

#[wasm_bindgen]
pub fn make_color(r: u8, g: u8, b: u8) -> Color { Color(r, g, b) }
#[wasm_bindgen]
pub fn make_colora(r: u8, g: u8, b: u8, a: u8) -> ColorA { ColorA(r, g, b, a) }
#[wasm_bindgen]
pub fn make_placement(horz_anchor: AxisAnchor, horz_off: isize, vert_anchor: AxisAnchor, vert_off: isize) -> Placement {
    Placement { horz: AxisPlacement { offset: horz_off, anchor: horz_anchor }, vert: AxisPlacement { offset: vert_off, anchor: vert_anchor } }
}

#[wasm_bindgen]
impl Options {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            text: String::new(),
            width: 350,
            height: 19,
            bg_top_color: Color(0, 0, 255),
            bg_bottom_color: Color(128, 255, 255),
            text_color: ColorA(255, 255, 255, 255),
            text_outline_color: ColorA(0, 0, 0, 255),
            ellipse_color: Some(ColorA(255, 255, 255, 40)),
            text_over_ellipse: false,
            border_color: Some(ColorA(0, 0, 0, 255)),
            text_placement: Placement {
                horz: AxisPlacement { anchor: AxisAnchor::Auto, offset: 0 },
                vert: AxisPlacement { anchor: AxisAnchor::Auto, offset: 0 },
            },
            diag_stripes: Some(StripePattern {
                color: ColorA(0, 0, 0, 180),
                on_main_diagonal: false,
                spacing: 4,
            }),
            bg_image: None,
        }
    }
}

// returns flat RGB buffer: array of [r, g, b, r, g, b, ...] with length width*height*3.
#[wasm_bindgen]
pub fn generate(opts: &Options) -> Vec<u8> {
    let width = opts.width;
    let height = opts.height;
    let mut canvas = drawing::Canvas::new(opts.width, opts.height);
    fn to_af_color(c: ColorA) -> drawing::ColorAF {
        drawing::ColorAF::from_srgb(c.0, c.1, c.2, c.3)
    }
    canvas.vert_gradient(
        0,
        width - 1,
        0,
        height - 1,
        opts.bg_top_color,
        opts.bg_bottom_color,
    );
    if let Some(StripePattern {
        color,
        on_main_diagonal,
        spacing,
    }) = opts.diag_stripes
    {
        let color = to_af_color(color);
        for x in 0..width as isize {
            for y in 0..height as isize {
                let off = if on_main_diagonal { x - y } else { x + y };
                if off % (spacing as isize) == 0 {
                    canvas.draw_px(x as usize, y as usize, color);
                }
            }
        }
    }

    if let Some(img) = &opts.bg_image {
        let im_offx = img.placement.horz.to_offset(
            AxisPlacement { anchor: AxisAnchor::Start, offset: 0 },
            img.width as isize,
            width as isize,
        );
        let im_offy = img.placement.vert.to_offset(
            AxisPlacement { anchor: AxisAnchor::Start, offset: 0 },
            img.height as isize,
            height as isize,
        );
        for x in 0..width as isize {
            for y in 0..height as isize {
                let ix = x - im_offx;
                let iy = y - im_offy;
                if ix >= 0 && (ix as usize) < img.width && iy >= 0 && (iy as usize) < img.height {
                    let offset = (img.width * iy as usize + ix as usize) * 4;
                    let buf = &img.data[offset..offset + 4];
                    let col = to_af_color(ColorA(buf[0], buf[1], buf[2], buf[3]));
                    canvas.draw_px(x as usize, y as usize, col);
                }
            }
        }
    }

    let do_ellipse = |canvas: &mut drawing::Canvas| {
        if let Some(x) = opts.ellipse_color {
            canvas.ellipse(
                width as f32 / 2.,
                0.,
                width as f32 / 2.,
                height as f32 / 2.,
                to_af_color(x),
            );
        }
    };

    if opts.text_over_ellipse {
        do_ellipse(&mut canvas);
    }

    let rendered = font::render(&opts.text);
    let textw = rendered.len() + 2;
    // +1 because this is the offset of the "main" text, but we computed it with the shadow
    let text_horz_offset =
        opts.text_placement
            .horz
            .to_offset(AxisPlacement { anchor: AxisAnchor::End, offset: 6 }, textw as isize, width as isize)
            + 1;
    // the bounding box is technically 9px tall,
    // but the height of most letters is only 5px.
    // so we have +1 for the shadow and -2 for the box height diff
    let text_vert_offset =
        opts.text_placement
            .vert
            .to_offset(AxisPlacement { anchor: AxisAnchor::Center, offset: 0 }, 7, height as isize)
            - 1;

    // draw the shadow of the text first
    let text_outline_color = to_af_color(opts.text_outline_color);
    for (x, column) in rendered.iter().enumerate() {
        for y in (0..9).filter(|&y| column[y] == 1) {
            for (dx, dy) in itertools::iproduct!(-1..=1, -1..=1) {
                let x = text_horz_offset + x as isize + dx;
                let y = text_vert_offset + y as isize + dy;
                if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
                    canvas.draw_px(x as usize, y as usize, text_outline_color);
                }
            }
        }
    }
    // now draw the main text
    let text_color = to_af_color(opts.text_color);
    for (x, column) in rendered.iter().enumerate() {
        for y in (0..9).filter(|&y| column[y] == 1) {
            let x = text_horz_offset + x as isize;
            let y = text_vert_offset + y as isize;
            if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
                canvas.draw_px(x as usize, y as usize, text_color);
            }
        }
    }

    if !opts.text_over_ellipse {
        do_ellipse(&mut canvas);
    }

    if let Some(x) = opts.border_color {
        let col = to_af_color(x);
        // don't wanna draw the corners twice, in case the color is transparent
        canvas.horz_line(1, width - 1, 0, col);
        canvas.vert_line(1, height - 1, width - 1, col);
        if width > 1 { canvas.horz_line(0, width - 2, height - 1, col); }
        if height > 1 { canvas.vert_line(0, height - 2, 0, col); }
    }
    canvas.get_buf()
}

// im so lazy lol
#[wasm_bindgen]
pub fn generate_rgba(o: &Options) -> Vec<u8> {
    let v = generate(o);
    let pixs = v.len() / 3;
    let mut out = vec![0; pixs * 4];
    for i in 0..pixs {
        out[i*4 + 0] = v[i*3 + 0];
        out[i*4 + 1] = v[i*3 + 1];
        out[i*4 + 2] = v[i*3 + 2];
        out[i*4 + 3] = 255;
    }
    out
}
