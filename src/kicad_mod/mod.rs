/// Currently, just used for writing a limited subset of KiCad footprint files
use std::io::Write;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct XYCoord {
    pub x: f32,
    pub y: f32,
}

impl std::ops::Add for XYCoord {
    type Output = XYCoord;
    fn add(self, rhs:Self) -> Self {
        XYCoord {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl std::ops::Sub for XYCoord {
    type Output = XYCoord;
    fn sub(self, rhs:Self) -> Self {
        XYCoord {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

pub enum Layer {
    FrontSilkscreen,
    // FrontMask,
    FrontCopper,
    FrontFabrication,
    // BackCopper,
    // BackMask,
    // BaskSilkscreen,
}

impl Layer {
    pub fn to_string(&self) -> &'static str {
        match self {
            Layer::FrontSilkscreen => "F.SilkS",
            Layer::FrontCopper => "F.Cu",
            Layer::FrontFabrication => "F.Fab",
        }
    }
}

pub enum Geometry {
    Circle { center: XYCoord, radius: f32},
    Line { ends: [XYCoord; 2] },
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

    pub fn line(start: XYCoord, end: XYCoord, width: f32, layer: Layer) -> Self {
        Shape {
            layer,
            geom: Geometry::Line {
                ends: [start, end]
            },
            thickness: width
        }
    }

    pub fn write(&self, out: &mut dyn Write) -> std::io::Result<()> {
        let mut s = String::new();

        match self.geom {
            Geometry::Circle{ref center, ref radius} => {
                s.push_str(&format!(" (fp_circle (center {} {}) (end {} {})",
                    center.x, center.y, center.x + radius, center.y));
            },
            Geometry::Line{ref ends} => {
                s.push_str(&format!(" (fp_line (start {} {}) (end {} {})",
                    ends[0].x, ends[0].y, ends[1].x, ends[1].y));
            }
        }

        s.push_str(&format!(" (layer {})", self.layer.to_string()));

        s.push_str(&format!(" (width {}))\n", self.thickness));

        out.write(s.as_bytes())?;

        Ok(())
    }
}

pub fn write(shapes: &Vec<Shape>, out: &mut dyn Write) -> std::io::Result<()> {
    // TODO Name, timestamp, etc
    out.write(b"(module test (layer F.Cu) (tedit 5BDB7444)\n")?;

    for shape in shapes {
        shape.write(out)?;
    }

    out.write(b")\n")?;

    Ok(())
}