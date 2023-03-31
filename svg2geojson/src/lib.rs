use std::{io::Write, ops::Deref};

use geojson::{feature::Id, Feature, LineStringType};
use lyon_algorithms::walk::{RegularPattern, WalkerEvent};
use lyon_path::{builder::SvgPathBuilder, Path};
use parse::Operation;

pub mod parse;

pub fn build_path(svg: &str) -> Path {
    let mut builder = Path::svg_builder().transformed(lyon_geom::Transform::scale(
        14980.0 / 512.0,
        14980.0 / 512.0,
    )); // TODO: make this dependent on variables
        // .transformed(Translation::new(-120.0, -120.0));

    for op in parse::path_to_operations(svg) {
        match op {
            Operation::M(to) => SvgPathBuilder::move_to(&mut builder, to),
            Operation::L(to) => SvgPathBuilder::line_to(&mut builder, to),
            Operation::Q { ctrl, to } => {
                SvgPathBuilder::quadratic_bezier_to(&mut builder, ctrl, to)
            }
            Operation::A {
                radii,
                x_rotation,
                flags,
                to,
            } => SvgPathBuilder::arc_to(&mut builder, radii, x_rotation, flags, to),
            Operation::Close => SvgPathBuilder::close(&mut builder),
        };
    }

    builder.build()
}

pub fn sample_path(path: &Path) -> LineStringType {
    let mut samples = vec![];

    let mut pattern = RegularPattern {
        callback: &mut |event: WalkerEvent| {
            samples.push(vec![event.position.x as f64, event.position.y as f64]);
            true
        },
        interval: 256.0,
    };
    lyon_algorithms::walk::walk_along_path(path, 0.0, 0.0, &mut pattern);
    samples
}

#[derive(Debug)]
pub enum Error {
    SVGOpen,
    GetRGB,
    ParseRGB,
    GetPath,
    GeoJsonWrite,
}

pub fn svg2geojson_filter_rgb(
    svg: &std::path::Path,
    output: impl Write,
    filter: impl Fn(parse::RGB) -> bool,
) -> Result<(), Error> {
    let mut buf = String::with_capacity(1024);
    let mut objects = vec![];

    for (i, event) in svg::open(svg, &mut buf)
        .map_err(|_| Error::SVGOpen)?
        .enumerate()
    {
        match event {
            svg::parser::Event::Tag("path", _, attrs) => {
                let fill = attrs.get("fill").ok_or(Error::GetRGB)?.deref();
                let (_, rgb) = parse::parse_rgb(fill).map_err(|_| Error::ParseRGB)?;

                if filter(rgb) {
                    continue;
                }

                let v = attrs.get("d").ok_or(Error::GetPath)?.deref();

                let path = build_path(v);
                let samples: LineStringType = sample_path(&path);

                objects.push(Feature {
                    id: Some(Id::String(format!("{i}"))),
                    bbox: None,
                    geometry: Some(geojson::Value::LineString(samples).into()),
                    properties: None,
                    foreign_members: None,
                });
            }
            _ => {}
        }
    }

    geojson::ser::to_feature_collection_writer(output, &objects).map_err(|_| Error::GeoJsonWrite)
}
