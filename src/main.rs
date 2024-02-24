use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn readimg(name: &str) -> anyhow::Result<(Vec<u8>, usize, usize)> {
    let decoder = png::Decoder::new(File::open(name)?);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let h = reader.next_frame(&mut buf)?;
    anyhow::ensure!(
        h.color_type == png::ColorType::Rgba,
        "Input image must be RGBA!"
    );
    anyhow::ensure!(
        h.bit_depth == png::BitDepth::Eight,
        "Input image must have 8 bits per channel!"
    );
    Ok((buf, h.width as usize, h.height as usize))
}

fn color_from_hex(hex: &str) -> anyhow::Result<libuserbar::Color> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16)? * 0x11;
            let g = u8::from_str_radix(&hex[1..2], 16)? * 0x11;
            let b = u8::from_str_radix(&hex[2..3], 16)? * 0x11;
            Ok(libuserbar::Color(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)?;
            let g = u8::from_str_radix(&hex[2..4], 16)?;
            let b = u8::from_str_radix(&hex[4..6], 16)?;
            Ok(libuserbar::Color(r, g, b))
        }
        _ => anyhow::bail!("Expected RGB hex color to have 3 or 6 digits"),
    }
}

fn colora_from_hex(hex: &str) -> anyhow::Result<libuserbar::ColorA> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    match hex.len() {
        3 | 6 => {
            let c = color_from_hex(hex)?;
            Ok(libuserbar::ColorA(c.0, c.1, c.2, 255))
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1], 16)? * 0x11;
            let g = u8::from_str_radix(&hex[1..2], 16)? * 0x11;
            let b = u8::from_str_radix(&hex[2..3], 16)? * 0x11;
            let a = u8::from_str_radix(&hex[3..4], 16)? * 0x11;
            Ok(libuserbar::ColorA(r, g, b, a))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16)?;
            let g = u8::from_str_radix(&hex[2..4], 16)?;
            let b = u8::from_str_radix(&hex[4..6], 16)?;
            let a = u8::from_str_radix(&hex[6..8], 16)?;
            Ok(libuserbar::ColorA(r, g, b, a))
        }
        _ => anyhow::bail!("Expected RGBA hex color to have 3, 4, 6 or 8 digits"),
    }
}

fn parse_placement(s: &str) -> anyhow::Result<libuserbar::Placement> {
    use libuserbar::AxisPlacement;
    use libuserbar::AxisAnchor;
    let parts: Vec<_> = s.split(',').collect();
    let parts: [&str; 2] = parts
        .try_into()
        .or(Err(anyhow::anyhow!("expected 2 components")))?;
    let do_part = |part: &str| -> Option<AxisPlacement> {
        Some(match part {
            "auto" => AxisPlacement { anchor: AxisAnchor::Auto, offset: 0 },
            "center" => AxisPlacement { anchor: AxisAnchor::Center, offset: 0 },
            x => {
                if let Some(x_strip) = x.strip_prefix('-') {
                    let d = x_strip.parse::<isize>().ok()?;
                    AxisPlacement { anchor: AxisAnchor::End, offset: d }
                } else {
                    let d = x.parse::<isize>().ok()?;
                    AxisPlacement { anchor: AxisAnchor::Start, offset: d }
                }
            }
        })
    };
    let horz = do_part(parts[0]).ok_or(anyhow::anyhow!("bad horizontal placement"))?;
    let vert = do_part(parts[1]).ok_or(anyhow::anyhow!("bad vertical placement"))?;
    Ok(libuserbar::Placement { horz, vert })
}

fn main() -> anyhow::Result<()> {
    let mut opts = libuserbar::Options::new();
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        println!("usage: userbar [options]

options:
    --grad-top      Color of top of background gradient [default: #00f]
    --grad-bottom   Color of bottom of background gradient [default: #8ff]
    -w, --width     Output width [default: 350]
    -h, --height    Output height [default: 19]
    -o, --output    Filename of output (will be PNG format) [required]
    -i, --bg-image  Image to use as a background [default: no image]
    --bg-pos        Placement of BG image [default: top-left corner]
    -t, --text      Text to use [required]
    --text-pos      Placement of text [default: center-right]
    --text-color    Color of text [default: #fff]
    --text-outline-color  Color of text's outline [default: #000]
    --no-ellipse    Disable the ellipse for the \"glare\" effect
    --ellipse-color Color of the ellipse [default: #ffffff28]
    --text-over-ellipse   Draw the text above the ellipse, instead of below
    --no-border     Disable drawing a border
    --border-color  Color of border [default: #000]
    --no-scan       Disable drawing \"scanlines\"
    --scan-color    Color of scanlines [default: #000000b4]
    --scan-flip     Flip scanline direction
    --scan-width    Width of scanline pattern [default: 4]
");
        return Ok(());
    }
    if let Some(v) = args.opt_value_from_fn("--grad-top", color_from_hex)? {
        opts.bg_top_color = v;
    }
    if let Some(v) = args.opt_value_from_fn("--grad-bottom", color_from_hex)? {
        opts.bg_bottom_color = v;
    }
    if let Some(v) = args.opt_value_from_fn(["-w", "--width"], str::parse::<usize>)? {
        opts.width = v;
    }
    if let Some(v) = args.opt_value_from_fn(["-h", "--height"], str::parse::<usize>)? {
        opts.height = v;
    }
    let outname: String = args.value_from_str(["-o", "--output"])?;
    if let Some(v) = args.opt_value_from_str::<_, String>(["-i", "--bg-image"])? {
        let (buf, width, height) = readimg(&v)?;
        opts.bg_image = Some(libuserbar::BgImage {
            width,
            height,
            data: buf,
            placement: libuserbar::Placement {
                horz: libuserbar::AxisPlacement { anchor: libuserbar::AxisAnchor::Auto, offset: 0 },
                vert: libuserbar::AxisPlacement { anchor: libuserbar::AxisAnchor::Auto, offset: 0 },
            },
        });
    }
    if let Some(v) = args.opt_value_from_fn("--bg-pos", parse_placement)? {
        opts.bg_image
            .as_mut()
            .ok_or(anyhow::anyhow!("--bg-pos provided without --bg-image"))?
            .placement = v;
    }

    let text: String = args.value_from_str(["-t", "--text"])?;
    opts.text = text;
    if let Some(v) = args.opt_value_from_fn("--text-pos", parse_placement)? {
        opts.text_placement = v;
    }

    if let Some(v) = args.opt_value_from_fn("--text-color", colora_from_hex)? {
        opts.text_color = v;
    }
    if let Some(v) = args.opt_value_from_fn("--text-outline-color", colora_from_hex)? {
        opts.text_outline_color = v;
    }
    if args.contains("--no-ellipse") {
        opts.ellipse_color = None;
    } else {
        if let Some(v) = args.opt_value_from_fn("--ellipse-color", colora_from_hex)? {
            opts.ellipse_color = Some(v);
        }
        if args.contains("--text-over-ellipse") {
            opts.text_over_ellipse = true;
        }
    }
    if args.contains("--no-border") {
        opts.border_color = None;
    } else if let Some(v) = args.opt_value_from_fn("--border-color", colora_from_hex)? {
        opts.border_color = Some(v);
    }

    if args.contains("--no-scan") {
        opts.diag_stripes = None;
    } else {
        // this is the default, which is "yes stripes"
        let mut awawa = opts.diag_stripes.unwrap();
        if let Some(v) = args.opt_value_from_fn("--scan-color", colora_from_hex)? {
            awawa.color = v;
        }
        if args.contains("--scan-flip") {
            awawa.on_main_diagonal = true;
        }
        if let Some(v) = args.opt_value_from_fn("--scan-width", str::parse::<usize>)? {
            awawa.spacing = v;
        }
        opts.diag_stripes = Some(awawa);
    }
    let rest = args.finish();
    if rest.len() > 0 {
        anyhow::bail!(
            "Unrecognized options: {}",
            rest.iter()
                .map(|x| x.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }

    let out = libuserbar::generate(&opts);

    let path = Path::new(&outname);
    let file = File::create(path)?;
    let w = BufWriter::new(file);
    let mut enc = png::Encoder::new(w, opts.width as u32, opts.height as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_adaptive_filter(png::AdaptiveFilterType::Adaptive);
    enc.set_srgb(png::SrgbRenderingIntent::Perceptual);
    enc.write_header()?.write_image_data(&out)?;
    Ok(())
}
