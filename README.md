# ffmpeg-loudnorm-helper <img src="ffmpeg-loudnorm-helper.svg" align="right" alt="ffmpeg-loudnorm-helper logo" width="20%"/>
[![License](https://img.shields.io/badge/license-GPLv3-blue.svg)](https://github.com/Indiscipline/ffmpeg-loudnorm-helper/blob/master/LICENSE.md)

Command line helper for performing audio loudness normalization with ffmpeg's `loudnorm`[^1] audio filter. 

Performs the loudness scanning pass of the given file and outputs the string of desired loudnorm options to be included in ffmpeg arguments. This automates the two-pass workflow that is necessary for the loudnorm filter to apply a uniform linear normalization that is desirable for keeping the original audio dynamics intact.

This program is a simpler alternative to the [ffmpeg-normalize](https://github.com/slhck/ffmpeg-normalize) Python script.

## Requirements

The program relies on `ffmpeg` of version >= 3.1 for its functions and expects it to be accessible from the environment.

Output of the `--resample` option implies ffmpeg is compiled with the SOX resampler support (libsoxr), though it is only printed and never executed. 

## Usage

**Summary**: Perform a single pass with `ffmpeg-lh input.wav` or use command substitution to combine both passes.

ffmpeg-loudnorm-helper is designed to work using your shell's command substitution capability. It launches an `ffmpeg` instance to measure the original audio loudness and then outputs the properly formatted copy-pasteable audio filter string to perform the required normalization. If the requested loudness target is provably unattainable with linear normalization, the program will warn the user. However, loudnorm is a bit picky, so there's no guarantee it will not fall back onto dynamic normalization. 

Full help available with the `--help` switch.

### Bash example
```
ffmpeg -i input.mov -c:v copy -c:a libopus $(ffmpeg-lh input.mov) normalized.mkv
```

### Windows CMD example
```
for /f "tokens=*" %i in ('ffmpeg-lh input.mov') do ffmpeg -i input.mov -c:v copy -c:a libopus %i normalized.mkv
```

## How to build

Developed with stable Rust with the minimal number of direct dependencies (Serde and clap).

To build the program, go to the project directory and run:

```
$ cargo build --release
```

The executable will be located at `target/release/ffmpeg-lh`.


## Contributing

The project is open for contributions. Open an [issue](https://github.com/Indiscipline/ffmpeg-loudnorm-helper/issues) for bugs, ideas and feature requests.

## License ##

**ffmpeg-loudnorm-helper** is licensed under GNU General Public License version 3;

See [`LICENSE.md`](LICENSE.md) for full details.

----

[^1]: An informative description of the [filter's inner-workings](https://k.ylo.ph/2016/04/04/loudnorm.html) by its author, Kyle Swanson.
