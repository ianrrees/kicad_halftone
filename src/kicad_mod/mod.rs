/// Currently, just used for writing a limited subset of KiCad footprint files

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

pub struct XYCoord {
    pub x: f32,
    pub y: f32,
}

pub enum Layer {
    FrontSilkscreen,
    // FrontMask,
    // FrontCopper,
    // BackCopper,
    // BackMask,
    // BaskSilkscreen,
}

impl Layer {
    pub fn to_string(&self) -> String {
        match self {
            Layer::FrontSilkscreen => String::from("F.SilkS"),
        }
    }
}

pub enum Geometry {
    Circle { center: XYCoord, radius: f32},
    // Poly { points: Vec<XYCoord>},
    // Text { location: XYCoord, text: String},
}

pub struct Shape {
    pub layer: Layer,
    pub geom: Geometry,
    pub thickness: f32 // Move this down in to Geometry?
}

impl Shape {
    /// Circle, thickness and radius make a filled circle as specified
    pub fn filled_circle(center: XYCoord, radius: f32, layer: Layer) -> Self {
        Shape {
            layer,
            geom: Geometry::Circle {
                center,
                radius: radius/2.0
            },
            thickness: radius
        }
    }

    pub fn write_to_file(&self, out_file: &mut File) -> std::io::Result<()> {
        let mut s = String::new();

        match self.geom {
            Geometry::Circle{ref center, ref radius} => {
                s.push_str(&format!(" (fp_circle (center {} {}) (end {} {})",
                    center.x, center.y, center.x + radius, center.y));
            },
        }

        s.push_str(&format!(" (layer {})", self.layer.to_string()));

        s.push_str(&format!(" (width {}))\n", self.thickness));

        out_file.write(s.as_bytes())?;

        Ok(())
    }
}

pub fn write(shapes: &Vec<Shape>, out_path: &Path) -> std::io::Result<()> {

    let mut out_file = File::create(out_path)?;

    // TODO Name, timestamp, etc
    out_file.write(b"(module test (layer F.Cu) (tedit 5BDB7444)\n")?;

    for shape in shapes {
        shape.write_to_file(&mut out_file)?;
    }

    out_file.write(b")\n")?;

    Ok(())
}