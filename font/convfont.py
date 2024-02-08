import dewinfont
import sys

f = open(sys.argv[1], "rb").read()
font_list = dewinfont.dofon(f)

print("pub static FONT: phf::Map<u32, (u8, [u8; 9])> = phf::phf_map! {")

for ind, char in enumerate(font_list[0].chars):
    if char.width != 0:
        try:
            char_ind = ord(bytes([ind]).decode('cp1252'))
        except UnicodeDecodeError:
            # all the glyphs this happens for are the filler glyph anyways
            continue
        # rightmost column is always empty - use this as it allows squeezing the font data into bytes instead of u16s
        assert all(x % 2 == 0 for x in char.data)
        newdata = [x // 2 for x in char.data]
        print(f"    {char_ind}_u32 => ({char.width}, {newdata}),")

print("};")
