extern crate argparse;
extern crate image;

use argparse::{ArgumentParser, Store};
use std::path::Path;

#[cfg(feature="gui")]
pub mod gui;

pub mod kicad_mod;
use kicad_mod::{Shape, XYCoord, Layer};

// use std::fmt;

// use image::GenericImage;

/// Basic characteristics of the halftone image we're making.  Linear dimension in mm
struct HalftoneParameters {
    dot_spacing: f32,
    min_dot_diam: f32,
    max_dot_diam: f32,
    output_width: f32,
    output_height: f32,
}

fn main() {
    let default_output_extension = "kicad_mod";

    let mut input_filename = String::new(); // At some stage, these should be replaceable with PathBuf's
    let mut output_filename = String::new();

    let mut halftone_params = HalftoneParameters {
        dot_spacing: 0.0,
        min_dot_diam: 0.0,
        max_dot_diam: 0.0,
        output_width: 0.0,
        output_height: 0.0,
    };

    let launch_gui = false;

    { // Block is to control scope of borrows in refer() calls
        let mut parser = ArgumentParser::new();
        parser.set_description("Generate KiCad footprints from bitmaps, using halftone");
        // TODO Add long description, explain that we need at least one of output width and height.
        parser.refer(&mut input_filename).required().
            add_argument("Input filename", Store, "Raster image source");
        parser.refer(&mut output_filename).
            add_option(&["-o", "--output"], Store, "Output file name");
        parser.refer(&mut halftone_params.output_width).
            add_option(&["-w", "--width"], Store, "Output width");
        parser.refer(&mut halftone_params.output_height).
            add_option(&["-h", "--height"], Store, "Output height");

        parser.parse_args_or_exit();

        // TODO Ensure we've got at least one of output_width and output_height
    }

    // Currently (November 2018), it seems that the Rust standard library doesn't have traits like
    // FromStr, so we need to use Strings for the command line parsing, then build Paths explicitly
    let input_path = Path::new(&input_filename);
    if !input_path.is_file() {
        println!("Couldn't read {}", &input_filename);
        return; // TODO Error
    }
    let output_path = if output_filename.is_empty() {
            input_path.with_extension(&default_output_extension)
        } else {
            Path::new(&output_filename).to_path_buf()
        };

    println!("Output {} x {}", halftone_params.output_width, halftone_params.output_height);

    if launch_gui {
        #[cfg(feature="gui")] {
            println!("Start GUI version");
            gui::launch_gui();
        }
        #[cfg(not(feature="gui"))] {
            println!("GUI not built in, sorry...");
            std::process::exit(1);
        }

    } else {
        println!("Start CLI version");

    }

    // Want to end up here knowing the HalftoneParameters, source, and destination filenames

    // let source_image = image::open(source_filename).expect(
    //     &format!("Failed to open source image \"{}\"", source_filename));

    // let dims = source_image.dimensions();
    // println!("Source image has size {}x{}", dims.0, dims.1);

    let mut shapes = Vec::<Shape>::new();

    // TODO: Iterate over the image here
    let coord = XYCoord {
        x : 0.0,
        y : 1.27
    };

    let s = Shape::filled_circle(coord, 0.2, Layer::FrontSilkscreen);

    shapes.push(s);

    kicad_mod::write(&shapes, &output_path).unwrap();
}
