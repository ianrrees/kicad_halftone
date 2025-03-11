extern crate clap;
use clap::{Arg, Command};

extern crate image;

use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(feature="gui")]
pub mod gui;

pub mod kicad_mod;
use self::kicad_mod::{Shape, XYCoord, Layer};

use image::{DynamicImage, GenericImageView, Pixel};
use image::imageops::{resize, colorops, FilterType};

/// Basic characteristics of the halftone image we're making.  Linear dimension in mm
struct HalftoneParameters {
    dot_spacing: f32,
    dot_min_diam: f32,
    dot_max_diam: f32,
    output_width: f32,
    output_height: f32,
    invert: bool,
}

enum ProgramSettings {
    Cli {source_image: DynamicImage,         output_path: PathBuf, params: HalftoneParameters},
    #[cfg(feature="gui")]
    Gui {source_image: Option<DynamicImage>, output_path: PathBuf, params: HalftoneParameters},
}

/// Parses command line arguments
fn parse_command_line() -> Result<ProgramSettings, String> {
    let default_output_extension = "kicad_mod";

    let cli_base = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("Generate KiCad footprints from bitmaps, using halftone technique")
        .long_about(
"Generate KiCad footprints from bitmaps, using halftone technique.  At least one of output width \
and output height needs to be specified.  If one is specified, then the input image's aspect ratio \
will be preserved, but if both are specified the image will be scaled to fit.")
        .arg(Arg::new("INPUT")
           .help("Raster image source")
           .index(1))
        .arg(Arg::new("OUTPUT")
           .help("Output file name - defaults input base name")
           .index(2))
        .arg(Arg::new("dot_spacing") //.takes_value(true)
            .help("Spacing between dots [mm]")
            .short('s').long("spacing"))
        .arg(Arg::new("dot_min_diam") //.takes_value(true)
            .help("Minimum diameter of dots [mm]")
            .short('d').long("dot-min"))
        .arg(Arg::new("dot_max_diam") //.takes_value(true)
            .help("Maximum diameter of dots [mm]")
            .short('D').long("dot-max"))
        .arg(Arg::new("output_width") //.takes_value(true)
            .help("Output width [mm]")
            .short('w').long("width"))
        // .disable_help_flag(true) // Bad design - masks help short argument
        .arg(Arg::new("output_height") //.takes_value(true)
            .help("Output height [mm]")
            .long("height"))
        .arg(Arg::new("invert")
            .help("Invert image brightness")
            .short('i').long("invert"));

    let cli;
    #[cfg(feature="gui")] {
        cli = cli_base
            .arg(Arg::new("gui")
                .help("Starts the graphical interface")
                .short("g").long("gui"))
            .get_matches();
    }
    #[cfg(not(feature="gui"))] {
        cli = cli_base.get_matches();
    }

    let mut params = HalftoneParameters {
        dot_spacing:   *cli.get_one::<f32>("dot_spacing").unwrap_or(&1.1),
        dot_min_diam:  *cli.get_one::<f32>("dot_min_diam").unwrap_or(&0.15), // From dirtypcbs.com
        dot_max_diam:  *cli.get_one::<f32>("dot_max_diam").unwrap_or(&1.2),
        output_width:  *cli.get_one::<f32>("output_width").unwrap_or(&0.0),
        output_height: *cli.get_one::<f32>("output_height").unwrap_or(&0.0),
        invert: cli.contains_id("invert"),
    };

    // INPUT is required for CLI, not for GUI
    if !cli.contains_id("INPUT") {
        #[cfg(feature="gui")] {
            if cli.is_present("gui") {
                let mut default_output_name: String = "output".to_owned();
                default_output_name.push_str(&default_output_extension);

                let output_path = Path::new(&cli.value_of("OUTPUT").
                    unwrap_or(&default_output_name))
                    .to_path_buf();

                return Ok(ProgramSettings::Gui{
                    source_image: None,
                    output_path,
                    params
                });
            }
            return Err("Input file name is required if --gui is not specified".to_string());
        }

        #[cfg(not(feature="gui"))] {
            return Err("Input file name is required".to_string());
        }
    }

    // Currently (November 2018), it seems that the Rust Path library doesn't have traits like
    // FromStr, so we need to use Strings for the command line parsing, then build Paths explicitly
    let input_filename = cli.get_one::<String>("INPUT").expect("Input file name is required");
    let input_path = Path::new(&input_filename);
    if !input_path.is_file() {
        return Err(format!("Couldn't read {}", &input_filename));
    }

    let output_path = if cli.contains_id("OUTPUT") {
            Path::new(&cli.get_one::<String>("OUTPUT").unwrap_or(&"".to_string())).to_path_buf()
        } else {
            match input_path.with_extension(&default_output_extension).file_name() {
                Some(name) => PathBuf::from(name),
                // I don't think this is possible, but...
                None => ["output.", &default_output_extension].iter().collect(),
            }
        };


    match image::open(&input_path) {
        Err(e) => {
            return Err(e.to_string());
        },
        Ok(source_image) => {
            let source_image_dims = source_image.dimensions();

            // Prevent possible divide-by-0
            if source_image_dims.0 == 0 || source_image_dims.1 == 0 {
                return Err("Command line parsing failed: \
                    Source image has no area; width and/or height is 0".to_string());
            }

            if params.output_width < 0.0 || params.output_height < 0.0 {
                return Err("Can't use negative values for width or height!".to_string());
            }

            // Ensure both output width and height are set in halftone_params:
            // At least one of them needs to be set from CLI or GUI
            if params.output_width == 0.0 &&
               params.output_height == 0.0 {
                return Err("Command line parsing failed: \
                    Need at least one of output width and height specified".to_string());
            } else if params.output_width != 0.0 &&
                      params.output_height == 0.0 {
                // Width specified, calculate height based on image
                params.output_height = params.output_width *
                    source_image_dims.1 as f32 / source_image_dims.0 as f32;

            } else if params.output_width == 0.0 &&
                      params.output_height != 0.0 {
                // Height specified, calculate width based on image
                params.output_width = params.output_height *
                    source_image_dims.0 as f32 / source_image_dims.1 as f32;
            }

            #[cfg(feature="gui")] {
                if cli.is_present("gui") {
                    return Ok(ProgramSettings::Gui{
                        source_image: Some(source_image),
                        output_path,
                        params
                    });
                }
            }

            return Ok(ProgramSettings::Cli{
                source_image,
                output_path,
                params
            });
        } // Ok(...
    } // match image::open(&input_path) {
} // end of parse_command_line()

/// The meat of this program - produces a bunch of dots and such from a raster graphic
fn make_halftone(source_image: DynamicImage, halftone_params: HalftoneParameters) -> Vec<Shape> {
    // Calculate number of rows and columns
    let half_dot_space = halftone_params.dot_spacing / 2.0;
    let max_dot_radius = halftone_params.dot_max_diam / 2.0;

    let row_spacing = halftone_params.dot_spacing * f32::to_radians(60.0).sin();

    let usable_width  = halftone_params.output_width -  halftone_params.dot_max_diam;
    let usable_height = halftone_params.output_height - halftone_params.dot_max_diam;

    let num_cols = (usable_width / half_dot_space).floor() as usize;
    let num_rows = (usable_height / row_spacing).floor() as usize;

    // intensity is in range [0, 1]
    let radius_from_intensity = |intensity:f32| -> f32 {
        use std::f32::consts::PI;

        // Derivation:
        // intensity = area_dot / area_max 
        // intensity * area_max = area_dot
        // intensity * area_max = pi * radius_dot^2
        // (intensity * area_max) / pi = radius_dot^2
        // ((intensity * area_max) / pi).sqrt() = radius_dot

        let area_max = PI * max_dot_radius.powi(2);
        let rad = ((intensity * area_max) / PI).sqrt();

        if rad.is_nan() {
            0.0
        } else {
            rad
        }
    };

    // There's bound to be a more elegant way to do this...  We're not scaling the output based on
    // the input image, but rather are scaling an intermediate raster image, which is then used to
    // generate the halftone.
    let px_per_mm = 5.0;  // input pixels, per output mm

    // Scale image to match the halftone grid
    let image = colorops::grayscale(&resize(&source_image,
        (halftone_params.output_width * px_per_mm).ceil() as u32,
        (halftone_params.output_height * px_per_mm).ceil() as u32,
        FilterType::Lanczos3));

    let mut shapes = Vec::<Shape>::new();

    // Just used to shift the footprint so it's centered on the origin
    let center = XYCoord {
        x : (num_cols as f32) / 2.0 * half_dot_space + max_dot_radius,
        y : (num_rows as f32) / 2.0 * row_spacing + max_dot_radius,
    };

    // Draw approximate bounds of input image on the fab layer
    let half_width = center.x;
    let half_height = center.y;
    shapes.push(Shape::line(XYCoord{x: -half_width, y: half_height},
                            XYCoord{x: half_width, y: half_height},
                            0.15, Layer::FrontFabrication));
    shapes.push(Shape::line(XYCoord{x: -half_width, y: -half_height},
                            XYCoord{x: half_width, y: -half_height},
                            0.15, Layer::FrontFabrication));
    shapes.push(Shape::line(XYCoord{x: half_width, y: -half_height},
                            XYCoord{x: half_width, y: half_height},
                            0.15, Layer::FrontFabrication));
    shapes.push(Shape::line(XYCoord{x: -half_width, y: -half_height},
                            XYCoord{x: -half_width, y: half_height},
                            0.15, Layer::FrontFabrication));

    for row in 0..num_rows {
        for col in 0..num_cols {
            // Make diagonal grid pattern by skipping half of positions
            if (row & 1) != (col & 1) {
                continue;
            }

            let coord = XYCoord {
                x : col as f32 * half_dot_space + max_dot_radius,
                y : row as f32 * row_spacing + max_dot_radius,
            };

            // Compute dot diam based on image intensity near point
            let mut score:u64 = 0;
            let mut max_score:u64 = 0;
            let left_px = ((coord.x - max_dot_radius) * px_per_mm).floor() as u32;
            let top_px = ((coord.y - max_dot_radius) * px_per_mm).floor() as u32;
            let diam_px = (2.0 * max_dot_radius).ceil() as u32;
            for y_px in top_px..(top_px + diam_px) {
                for x_px in left_px..(left_px + diam_px) {
                    // Explicitly make px a u8, so that we do the right thing
                    // with max_score if get_pixel() bit depth improves
                    let px:u8 = image.get_pixel(x_px, y_px).channels()[0];
                    score += px as u64;
                    max_score += u8::max_value() as u64;
                }
            }

            let intensity = if halftone_params.invert {
                    1.0 - (score as f32 / max_score as f32)
                } else {
                    score as f32 / max_score as f32
                };

            let radius = radius_from_intensity(intensity);
            if radius <= 0.0 || radius * 2.0 < halftone_params.dot_min_diam {
                continue;
            }

            shapes.push(Shape::filled_circle(
                coord - center,
                radius,
                Layer::FrontSilkscreen));
        }
    }

    shapes
}

fn main() {
    match parse_command_line() {
        #[cfg(feature="gui")]
        Ok(ProgramSettings::Gui{source_image, output_path, params}) => {
            println!("Start GUI version");
            gui::launch_gui();
        },

        Ok(ProgramSettings::Cli{source_image, output_path, params}) => {
            let mut out_file = File::create(output_path).unwrap(); // TODO check for existence first

            let shapes = make_halftone(source_image, params);


            kicad_mod::write(&shapes, &mut out_file).unwrap();
        },

        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
}
