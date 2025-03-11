# kicad_halftone
Turns raster images in to PCB silkscreen images using halftone technique

I've been meaning to learn Rust for ages; this is a mainly a toy project towards that.  See my [post on an earlier iteration](http://ianrrees.github.io/2018/06/20/pcb-graphics-with-kicad-+-gimp.html) for some background.

![Example input - photo of Rube Goldberg](examples/rube-photo.jpg)
![Example output - screenshot of KiCad footprint](examples/rube-kicad.png)
![Example board - photo of finished PCB with Rube Goldberg halftone image](examples/rube-pcb.jpg)

Still a work in progress, when I've got some free time and feel like programming...

## Building

  1. Clone this repo
  2. Install Rust and cargo
  3. `$cargo build`

## Using
```
➜  kicad_halftone git:(master) ✗ cargo run -- -h
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/kicad_halftone -h`
Generate KiCad footprints from bitmaps, using halftone technique

Usage: kicad_halftone [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   Raster image source
  [OUTPUT]  Output file name - defaults input base name

Options:
  -s, --spacing <DOT_SPACING>   Spacing between dots [mm] [default: 1.1]
  -d, --dot-min <DOT_MIN_DIAM>  Minimum diameter of dots [mm] [default: 0.15]
  -D, --dot-max <DOT_MAX_DIAM>  Maximum diameter of dots [mm] [default: 1.2]
  -w, --width <OUTPUT_WIDTH>    Output width [mm] [default: 0]
      --height <OUTPUT_HEIGHT>  Output height [mm] [default: 0]
  -i, --invert                  Invert image brightness
  -h, --help                    Print help (see more with '--help')
  -V, --version                 Print version
```
