   # ffmpeg-loudnorm-helper

[![License](https://img.shields.io/badge/license-GPLv3-blue.svg)](https://github.com/Indiscipline/ffmpeg-loudnorm-helper/blob/master/LICENSE.md)

Command line helper for performing linear audio loudness normalization using ffmpeg's loudnorm audio filter. Performs the loudness scanning pass of the given file and outputs the string of desired loudnorm options to be included in ffmpeg arguments.
The program expects `ffmpeg` to be in your PATH.

This program is a much simpler substitute for a [ffmpeg-normalize](https://github.com/slhck/ffmpeg-normalize) Python script.

Developed using the wonderful [Clap](https://github.com/kbknapp/clap-rs) crate.


## Usage
ffmpeg-loudnorm-helper is designed to work using your shell's command substitution capability.

Bash example:
```
ffmpeg -i input.mov -c:v copy -c:a libopus $(ffmpeg-lh input.mov) normalized.mkv
```

Windows CMD:
```
for /f "tokens=*" %i in ('ffmpeg-lh input.mov') do ffmpeg -i input.mov -c:v copy -c:a libopus %i normalized.mkv
```

Full help available on `--help` switch.


## How to build
Developed with stable Rust.

To build the code, go to the project directory and run:

```
$ cargo build --release
```

The executable will be `target/release/ffmpeg-lh`.


## Contributing ##
This is a small helper utility which achieves its intended functionality, but if you know how to improve it, file a bug report via [Issues](https://github.com/Indiscipline/ffmpeg-loudnorm-helper/issues).

## License ##
**ffmpeg-loudnorm-helper** licensed under GNU General Public License version 3;

See `LICENSE.md` for full details.