extern crate image;

#[cfg(feature="gui")]
pub mod gui;

// use std::fmt;
use std::fs::File;
use std::io::Write;

use image::GenericImage;

/// Basic characteristics of the halftone image we're making.  Linear dimension in mm
struct OutputParameters {
    dot_spacing_x: f32,
    dot_spacing_y: f32,
    min_dot_diam: f32,
    overall_x: f32,
    overall_y: f32,
}

fn parse_cli_args(args: Vec<String>) -> Result<(String, String), &'static str> {
    if args.len() != 3 {
        return Err("Expected kicad_halftone source dest");
    }

    let source = args[1].clone();
    let dest = args[2].clone();

    Ok((source, dest))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("Got {} args: {:?}", args.len(), args);

    if args.len() == 1 {
        #[cfg(feature="gui")] {
            println!("Start GUI version");
            gui::launch_gui();
        }
        #[cfg(not(feature="gui"))] {
            println!("GUI not built in, sorry...");
        }

    } else {
        println!("Start CLI version");
        let (source_filename, dest_filename) = parse_cli_args(args).unwrap();
    }
    // writeln!(std::io::stderr(), ).unwrap();
    // std::process::exit(1);


    // let source_image = image::open(source_filename).expect(
    //     &format!("Failed to open source image \"{}\"", source_filename));

    // let dims = source_image.dimensions();
    // println!("Source image has size {}x{}", dims.0, dims.1);
}
