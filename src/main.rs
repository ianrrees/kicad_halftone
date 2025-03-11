use clap::Parser;

extern crate image;

use std::fs::File;
use std::path::{Path, PathBuf};

pub mod kicad_mod;
use self::kicad_mod::{Layer, Shape, XYCoord};

use image::imageops::{colorops, resize, FilterType};
use image::{DynamicImage, GenericImageView, Pixel};

/// Basic characteristics of the halftone image we're making.  Linear dimension in mm
struct HalftoneParameters {
    dot_spacing: f32,
    dot_min_diam: f32,
    dot_max_diam: f32,
    output_width: f32,
    output_height: f32,
    invert: bool,
}

struct ProgramSettings {
    source_image: DynamicImage,
    output_path: PathBuf,
    params: HalftoneParameters,
}

#[derive(Parser, Debug)]
#[command(
    version=clap::crate_version!(),
    about="Generate KiCad footprints from bitmaps, using halftone technique",
    long_about="
Generate KiCad footprints from bitmaps, using halftone technique.  At least one of output width \
and output height needs to be specified.  If one is specified, then the input image's aspect ratio \
will be preserved, but if both are specified the image will be scaled to fit.")]
struct Args {
    #[arg(help = "Raster image source")]
    input: String,
    #[arg(help = "Output file name - defaults input base name")]
    output: Option<String>,
    #[arg(
        short = 's',
        long = "spacing",
        default_value_t = 1.1,
        help = "Spacing between dots [mm]"
    )]
    dot_spacing: f32,
    // Minimum diameter from dirtypcbs.com
    #[arg(
        short = 'd',
        long = "dot-min",
        default_value_t = 0.15,
        help = "Minimum diameter of dots [mm]"
    )]
    dot_min_diam: f32,
    #[arg(
        short = 'D',
        long = "dot-max",
        default_value_t = 1.2,
        help = "Maximum diameter of dots [mm]"
    )]
    dot_max_diam: f32,
    #[arg(
        short = 'w',
        long = "width",
        default_value_t = 0.0,
        help = "Output width [mm]"
    )]
    output_width: f32,
    // nb the dumb choices for argument names means -h would clobber --help
    #[arg(long = "height", default_value_t = 0.0, help = "Output height [mm]")]
    output_height: f32,
    #[arg(short = 'i', long = "invert", help = "Invert image brightness")]
    invert: bool,
}

/// Parses command line arguments
fn parse_command_line() -> Result<ProgramSettings, String> {
    let default_output_extension = "kicad_mod";

    let cli = Args::parse();

    let mut params = HalftoneParameters {
        dot_spacing: cli.dot_spacing,
        dot_min_diam: cli.dot_min_diam,
        dot_max_diam: cli.dot_max_diam,
        output_width: cli.output_width,
        output_height: cli.output_height,
        invert: cli.invert,
    };

    // Currently (November 2018), it seems that the Rust Path library doesn't have traits like
    // FromStr, so we need to use Strings for the command line parsing, then build Paths explicitly
    let input_path = Path::new(&cli.input);
    if !input_path.is_file() {
        return Err(format!("Couldn't read {}", &cli.input));
    }

    let output_path = if let Some(output_name) = cli.output {
        Path::new(&output_name).to_path_buf()
    } else {
        match input_path
            .with_extension(&default_output_extension)
            .file_name()
        {
            Some(name) => PathBuf::from(name),
            // I don't think this is possible, but...
            None => ["output.", &default_output_extension].iter().collect(),
        }
    };

    match image::open(&input_path) {
        Err(e) => {
            return Err(e.to_string());
        }
        Ok(source_image) => {
            let source_image_dims = source_image.dimensions();

            // Prevent possible divide-by-0
            if source_image_dims.0 == 0 || source_image_dims.1 == 0 {
                return Err("Command line parsing failed: \
                    Source image has no area; width and/or height is 0"
                    .to_string());
            }

            if params.output_width < 0.0 || params.output_height < 0.0 {
                return Err("Can't use negative values for width or height!".to_string());
            }

            // Ensure both output width and height are set in halftone_params:
            // At least one of them needs to be supplied
            if params.output_width == 0.0 && params.output_height == 0.0 {
                return Err("Command line parsing failed: \
                    Need at least one of output width and height specified"
                    .to_string());
            } else if params.output_width != 0.0 && params.output_height == 0.0 {
                // Width specified, calculate height based on image
                params.output_height =
                    params.output_width * source_image_dims.1 as f32 / source_image_dims.0 as f32;
            } else if params.output_width == 0.0 && params.output_height != 0.0 {
                // Height specified, calculate width based on image
                params.output_width =
                    params.output_height * source_image_dims.0 as f32 / source_image_dims.1 as f32;
            }

            return Ok(ProgramSettings {
                source_image,
                output_path,
                params,
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

    let usable_width = halftone_params.output_width - halftone_params.dot_max_diam;
    let usable_height = halftone_params.output_height - halftone_params.dot_max_diam;

    let num_cols = (usable_width / half_dot_space).floor() as usize;
    let num_rows = (usable_height / row_spacing).floor() as usize;

    // intensity is in range [0, 1]
    let radius_from_intensity = |intensity: f32| -> f32 {
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
    let px_per_mm = 5.0; // input pixels, per output mm

    // Scale image to match the halftone grid
    let image = colorops::grayscale(&resize(
        &source_image,
        (halftone_params.output_width * px_per_mm).ceil() as u32,
        (halftone_params.output_height * px_per_mm).ceil() as u32,
        FilterType::Lanczos3,
    ));

    let mut shapes = Vec::<Shape>::new();

    // Just used to shift the footprint so it's centered on the origin
    let center = XYCoord {
        x: (num_cols as f32) / 2.0 * half_dot_space + max_dot_radius,
        y: (num_rows as f32) / 2.0 * row_spacing + max_dot_radius,
    };

    // Draw approximate bounds of input image on the fab layer
    let half_width = center.x;
    let half_height = center.y;
    shapes.push(Shape::line(
        XYCoord {
            x: -half_width,
            y: half_height,
        },
        XYCoord {
            x: half_width,
            y: half_height,
        },
        0.15,
        Layer::FrontFabrication,
    ));
    shapes.push(Shape::line(
        XYCoord {
            x: -half_width,
            y: -half_height,
        },
        XYCoord {
            x: half_width,
            y: -half_height,
        },
        0.15,
        Layer::FrontFabrication,
    ));
    shapes.push(Shape::line(
        XYCoord {
            x: half_width,
            y: -half_height,
        },
        XYCoord {
            x: half_width,
            y: half_height,
        },
        0.15,
        Layer::FrontFabrication,
    ));
    shapes.push(Shape::line(
        XYCoord {
            x: -half_width,
            y: -half_height,
        },
        XYCoord {
            x: -half_width,
            y: half_height,
        },
        0.15,
        Layer::FrontFabrication,
    ));

    for row in 0..num_rows {
        for col in 0..num_cols {
            // Make diagonal grid pattern by skipping half of positions
            if (row & 1) != (col & 1) {
                continue;
            }

            let coord = XYCoord {
                x: col as f32 * half_dot_space + max_dot_radius,
                y: row as f32 * row_spacing + max_dot_radius,
            };

            // Compute dot diam based on image intensity near point
            let mut score: u64 = 0;
            let mut max_score: u64 = 0;
            let left_px = ((coord.x - max_dot_radius) * px_per_mm).floor() as u32;
            let top_px = ((coord.y - max_dot_radius) * px_per_mm).floor() as u32;
            let diam_px = (2.0 * max_dot_radius).ceil() as u32;
            for y_px in top_px..(top_px + diam_px) {
                for x_px in left_px..(left_px + diam_px) {
                    // Explicitly make px a u8, so that we do the right thing
                    // with max_score if get_pixel() bit depth improves
                    let px: u8 = image.get_pixel(x_px, y_px).channels()[0];
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
                Layer::FrontSilkscreen,
            ));
        }
    }

    shapes
}

fn main() {
    match parse_command_line() {
        Ok(ProgramSettings {
            source_image,
            output_path,
            params,
        }) => {
            let mut out_file = File::create(output_path).unwrap(); // TODO check for existence first

            let shapes = make_halftone(source_image, params);

            kicad_mod::write(&shapes, &mut out_file).unwrap();
        }

        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
}
