# userbar generation library

The userbar style is copied from [Daniel15's Userbar Generator](http://www.dansoftaustralia.net/products/userbar.htm). The font used is Visitor. I used dewinfont from [here](https://github.com/juanitogan/mkwinfont) to convert the .FON into raw bitmaps for embedding inside the program.

the JS frontend (in userbar.html) needs coloris.min.js and coloris.min.css from [here](https://github.com/mdbassit/Coloris), and libuserbar.js and libuserbar_bg.wasm built using `wasm-pack` (by running `wasm-pack build --target web` in the libuserbar directory).
