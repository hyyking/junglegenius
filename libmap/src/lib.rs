#![feature(get_many_mut, let_chains)]

use std::io::Write;

use geojson::{feature::Id, Feature, JsonValue};

use lyon_path::{builder::SvgPathBuilder, traits::Build, Path};

pub mod intextgrouper;
pub mod mapreader;
pub mod parse;
pub mod sampler;

use crate::{
    intextgrouper::IntExtGrouper, mapreader::SvgMapReader, parse::Operation, sampler::PointSampler,
};

pub trait Pipe {
    type Input;
    type Output;

    fn pipe<T: Pipe>(other: T) -> Pipe<T, Self>;
}

struct Pipe<I, O>
where
    I: Pipe,
    O: Pipe, {}

pub fn build_path(
    path_string: &str,
    builder: impl SvgPathBuilder + Build<PathType = Path> + 'static,
) -> Result<Path, nom::Err<nom::error::Error<&str>>> {
    parse::path_to_operations(path_string).map(|(_, ops)| {
        ops.into_iter()
            .flatten()
            .fold(builder, |mut builder, op| {
                match op {
                    Operation::MoveTo(to) => SvgPathBuilder::move_to(&mut builder, to),
                    Operation::LineTo(to) => SvgPathBuilder::line_to(&mut builder, to),
                    Operation::QuadBezierTo { ctrl, to } => {
                        SvgPathBuilder::quadratic_bezier_to(&mut builder, ctrl, to)
                    }
                    Operation::ArcTo {
                        radii,
                        x_rotation,
                        flags,
                        to,
                    } => SvgPathBuilder::arc_to(&mut builder, radii, x_rotation, flags, to),

                    Operation::RelMoveTo(to) => SvgPathBuilder::relative_move_to(&mut builder, to),
                    Operation::RelQuadBezierTo { ctrl, to } => {
                        SvgPathBuilder::relative_quadratic_bezier_to(&mut builder, ctrl, to)
                    }
                    Operation::RelLineTo(to) => SvgPathBuilder::relative_line_to(&mut builder, to),
                    Operation::RelArcTo {
                        radii,
                        x_rotation,
                        flags,
                        to,
                    } => {
                        SvgPathBuilder::relative_arc_to(&mut builder, radii, x_rotation, flags, to)
                    }
                    Operation::VerticalLineTo(to) => {
                        SvgPathBuilder::vertical_line_to(&mut builder, to)
                    }
                    Operation::HorizontalLineTo(to) => {
                        SvgPathBuilder::horizontal_line_to(&mut builder, to)
                    }
                    Operation::RelCubicBezierTo { ctrl1, ctrl2, to } => {
                        SvgPathBuilder::relative_cubic_bezier_to(&mut builder, ctrl1, ctrl2, to)
                    }

                    Operation::Close => SvgPathBuilder::close(&mut builder),
                    Operation::RelVerticalLineTo(dy) => {
                        SvgPathBuilder::relative_vertical_line_to(&mut builder, dy)
                    }
                    Operation::RelHorizontalLineTo(dx) => {
                        SvgPathBuilder::relative_horizontal_line_to(&mut builder, dx)
                    }
                    Operation::CubicBezierTo { ctrl1, ctrl2, to } => {
                        SvgPathBuilder::cubic_bezier_to(&mut builder, ctrl1, ctrl2, to)
                    }
                };
                builder
            })
            .build()
    })
}

#[derive(Debug)]
pub enum Error {
    SVGOpen,
    GetRGB,
    ParseRGB,
    GetPath,
    GeoJsonWrite,
}
pub fn svg2geojson(
    path: impl AsRef<std::path::Path>,
    output: impl Write,
    sampler: PointSampler,
) -> Result<(), Error> {
    let mut buff = String::with_capacity(4096);
    let reader = SvgMapReader::open_with(path, sampler, &mut buff).unwrap();

    let objects = IntExtGrouper::new(reader)
        .map(|sample| Feature {
            bbox: None,
            geometry: Some(geojson::Value::Polygon(sample.poly).into()),
            id: Some(Id::String(sample.id)),
            properties: dbg!(serde_json::to_value(sample.properties)
                .ok()
                .as_ref()
                .and_then(JsonValue::as_object)
                .cloned()),
            foreign_members: None,
        })
        .collect::<Vec<_>>();
    dbg!(objects.len());
    geojson::ser::to_feature_collection_writer(output, &objects).map_err(|_| Error::GeoJsonWrite)
}
