/// Mostly placeholder at this stage - need to figure out how to display raster
/// graphics with IUI...


// DISREGARD THIS - NEW PLAN IS TO USE CONROD

extern crate iui;
use self::iui::prelude::*;
use self::iui::controls::{HorizontalBox, VerticalBox, Button};

pub fn launch_gui() {
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