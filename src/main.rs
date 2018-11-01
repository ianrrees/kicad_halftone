extern crate image;

#[cfg(feature="gui")]
extern crate iui;
#[cfg(feature="gui")]
use iui::prelude::*;
#[cfg(feature="gui")]
use iui::controls::{HorizontalBox, VerticalBox, Button};

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

#[cfg(feature="gui")]
fn launch_gui() {
    let ui = UI::init().unwrap();

    let mut window = Window::new(&ui, "Halftone generator", 640, 480, WindowType::NoMenubar);
    let mut hbox = HorizontalBox::new(&ui);

    let mut open_button = Button::new(&ui, "Open Button!");
    let mut save_button = Button::new(&ui, "Save Button!");
    // let mut previewArea = Area::new(&ui);

    open_button.on_clicked(&ui, |_| {
        println!("open_button got clicked!");
        if let Some(path) = &window.open_file(&ui) {
            println!("Would open {:?}", &path);
            // let mut file = match File::create(&path) {
            //     Err(why) => { window.modal_err(&ui, "I/O Error", &format!("Could not open file {}: {}", path.display(), why.description())); return; }
            //     Ok(f) => f
            // };
            // match file.write_all(entry.value(&ui).as_bytes()) {
            //     Err(why) => { window.modal_err(&ui, "I/O Error", &format!("Could not write to file {}: {}", path.display(), why.description())); return; }
            //     Ok(_) => ()
            // };
        }
    });

    hbox.append(&ui, open_button, LayoutStrategy::Stretchy);
    hbox.append(&ui, save_button.clone(), LayoutStrategy::Stretchy);
    // hbox.append(&ui, previewArea.clone(), LayoutStrategy::Stretchy);

    window.set_child(&ui, hbox);
    window.show(&ui);

    // save_button.on_clicked(&ui, {
    //     println!("save_button got clicked!");
    //     let ui = ui.clone();
    //     move |_| {
    //         if let Some(path) = window.save_file(&ui) {
    //             // let mut file = match File::create(&path) {
    //             //     Err(why) => { window.modal_err(&ui, "I/O Error", &format!("Could not open file {}: {}", path.display(), why.description())); return; }
    //             //     Ok(f) => f
    //             // };
    //             // match file.write_all(entry.value(&ui).as_bytes()) {
    //             //     Err(why) => { window.modal_err(&ui, "I/O Error", &format!("Could not write to file {}: {}", path.display(), why.description())); return; }
    //             //     Ok(_) => ()
    //             // };
    //         }    
    //     }
    // });

    ui.main();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("Got {} args: {:?}", args.len(), args);

    if args.len() == 1 {
        #[cfg(feature="gui")] {
            println!("Start GUI version");
            launch_gui();
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
