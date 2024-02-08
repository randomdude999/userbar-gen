use crate::font_data;

// renders the given text. returns in a somewhat weird format:
// vec of columns. in each column values are from top down, 0=no text 1=yes text
pub fn render(text: &str) -> Vec<[u8; 9]> {
    let mut out: Vec<[u8; 9]> = Vec::new();
    let default_char = font_data::FONT[&0x7f];
    for c in text.chars() {
        let char = font_data::FONT.get(&(c as u32)).unwrap_or(&default_char);
        for col in 0..char.0 - 1 {
            let real_col = char.0 - col - 2;
            out.push(char.1.map(|x| ((x >> real_col) & 1) as u8));
        }
        out.push([0; 9]);
    }
    // remove the padding after the last character
    out.pop();
    out
}
