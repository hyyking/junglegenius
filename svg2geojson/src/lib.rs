#![feature(get_many_mut)]

use std::{collections::HashMap, io::Write, ops::Deref};

use geojson::{feature::Id, Feature, JsonValue};

use lyon_path::{builder::SvgPathBuilder, traits::Build, Path};
use parse::{Operation, RGB};
use sampler::{IdSampler, PathSampler, PointSampler};
use svg::Parser;

pub mod parse;
pub mod sampler;

pub struct SvgMapOpReader<'a, S = IdSampler>
where
    S: PathSampler,
{
    parser: Parser<'a>,
    ignore_groups: usize,
    sampler: S,
}

impl<'a> SvgMapOpReader<'a, IdSampler> {
    pub fn open(
        path: impl AsRef<std::path::Path>,
        buff: &'a mut String,
    ) -> Result<Self, std::io::Error> {
        svg::open(path.as_ref(), buff).map(|parser| Self {
            parser,
            sampler: IdSampler,
            ignore_groups: 0,
        })
    }
}
impl<'a, S> SvgMapOpReader<'a, S>
where
    S: PathSampler,
{
    pub fn open_with(
        path: impl AsRef<std::path::Path>,
        sampler: S,
        buff: &'a mut String,
    ) -> Result<Self, std::io::Error> {
        svg::open(path.as_ref(), buff).map(|parser| Self {
            parser,
            sampler,
            ignore_groups: 0,
        })
    }
}

pub enum MapOperation<S: PathSampler> {
    StartNewGroup(String),
    NewPath(S::Sample, HashMap<String, svg::node::Value>),
    ChildrenInteriorOfParent,
    ChildExteriorOfParent,
    EndNewGroup,
    NotSupported,
}

impl<S> Iterator for SvgMapOpReader<'_, S>
where
    S: PathSampler,
{
    type Item = MapOperation<S>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next().map(|event| match event {
            svg::parser::Event::Tag("g", svg::node::element::tag::Type::Start, attrs) => {
                match attrs.get("inkscape:label").map(|n| n.deref()) {
                    Some("interior") => {
                        self.ignore_groups += 1;
                        MapOperation::ChildrenInteriorOfParent
                    }
                    Some("exterior") => {
                        self.ignore_groups += 1;
                        MapOperation::ChildExteriorOfParent
                    }
                    _ => MapOperation::StartNewGroup(attrs.get("id").unwrap().to_string()),
                }
            }
            svg::parser::Event::Tag("g", svg::node::element::tag::Type::End, _) => {
                if self.ignore_groups == 0 {
                    MapOperation::EndNewGroup
                } else {
                    self.ignore_groups = self.ignore_groups.saturating_sub(1);
                    MapOperation::NotSupported
                }
            }
            svg::parser::Event::Tag("path", _, attrs) => {
                let v = attrs.get("d").ok_or(Error::GetPath).unwrap().deref();
                let path = build_path(
                    v,
                    Path::svg_builder().transformed(lyon_geom::Transform::scale(
                        14980.0 / 512.0,
                        14980.0 / 512.0,
                    )),
                )
                .unwrap();
                MapOperation::NewPath(self.sampler.sample(path), attrs)
            }
            _ => MapOperation::NotSupported,
        })
    }
}

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
#[derive(Debug)]
pub enum AppendMode {
    Direct,
    AppendExterior,
    AppendInterior,
    BuildPoly,
}

pub fn svg2geojson(
    path: impl AsRef<std::path::Path>,
    output: impl Write,
    sampler: PointSampler,
) -> Result<(), Error> {
    let mut group_stack: Vec<JsonValue> = vec![];

    let mut state = vec![AppendMode::Direct];

    let mut curr_poly = Feature {
        id: None,
        bbox: None,
        geometry: None,
        properties: None,
        foreign_members: None,
    };

    let mut buff = String::with_capacity(1024);
    let reader = SvgMapOpReader::open_with(path, sampler, &mut buff).unwrap();

    let objects: Vec<Feature> = reader
        .filter_map(|op| {
            let mut build_poly = false;
            match op {
                MapOperation::StartNewGroup(id) => {
                    group_stack.push(geojson::JsonValue::String(id.clone()));
                    curr_poly.id = Some(geojson::feature::Id::String(id));
                    curr_poly.geometry = Some(geojson::Value::Polygon(vec![]).into());
                }
                MapOperation::ChildrenInteriorOfParent => state.push(AppendMode::AppendInterior),
                MapOperation::ChildExteriorOfParent => state.push(AppendMode::AppendExterior),

                MapOperation::NewPath(samples, attrs) => {
                    let id = attrs.get("id").map(ToString::to_string).unwrap_or_default();

                    let properties = build_properties(attrs, &group_stack).ok()?;

                    if samples.is_empty() {
                        println!("empty samples: {properties:#?} path: {id}");
                        return None;
                    }

                    match state.last() {
                        Some(AppendMode::Direct) => {
                            curr_poly.id = Some(Id::String(id));
                            curr_poly.geometry =
                                Some(geojson::Value::Polygon(vec![samples]).into());
                            curr_poly.properties = Some(properties);

                            build_poly = true;
                        }
                        Some(AppendMode::AppendInterior) => {
                            if let geojson::Value::Polygon(ref mut lines) =
                                curr_poly.geometry.as_mut().unwrap().value
                            {
                                lines.push(samples)
                            }
                        }
                        Some(AppendMode::AppendExterior) => {
                            if let geojson::Value::Polygon(ref mut lines) =
                                curr_poly.geometry.as_mut().unwrap().value
                            {
                                lines.insert(0, samples)
                            }
                        }
                        _ => {}
                    }
                }
                MapOperation::EndNewGroup => {
                    let _ = group_stack.pop();
                }
                MapOperation::NotSupported => {
                    use AppendMode::AppendExterior as E;
                    use AppendMode::AppendInterior as I;
                    let last_two = [state.len().saturating_sub(2), state.len().saturating_sub(1)];
                    if let Ok([I, E] | [E, I]) = state.get_many_mut(last_two) {
                        state.pop();
                        state.pop();

                        let mut properties = geojson::JsonObject::new();

                        if !group_stack.is_empty() {
                            properties.insert(
                                "groups".to_string(),
                                geojson::JsonValue::Array(group_stack.clone()),
                            );
                        }
                        curr_poly.properties = Some(properties);

                        build_poly = true;
                    }
                }
            }

            if build_poly {
                println!(
                    "new feature id: {:?} props: {:#?}",
                    curr_poly.id, curr_poly.properties
                );
                Some(std::mem::replace(&mut curr_poly, Feature::default()))
            } else {
                None
            }
        })
        .collect();

    geojson::ser::to_feature_collection_writer(output, &objects).map_err(|_| Error::GeoJsonWrite)
}

fn build_properties(
    attrs: HashMap<String, svg::node::Value>,
    group_stack: &Vec<JsonValue>,
) -> Result<geojson::JsonObject, Error> {
    let mut properties = geojson::JsonObject::new();
    if let Some(fill) = attrs.get("fill") {
        let rgb = nom::branch::alt((parse::parse_rgb, parse::parse_hex_rgb))(fill)
            .map_err(|_| Error::ParseRGB)?
            .1;

        properties.insert(
            "fill".to_string(),
            geojson::JsonValue::Object(geojson::JsonObject::from_iter([
                ("r".to_string(), rgb.r.into()),
                ("g".to_string(), rgb.g.into()),
                ("b".to_string(), rgb.b.into()),
            ])),
        );

        rgb
    } else {
        RGB { r: 0, g: 0, b: 0 }
    };

    if !group_stack.is_empty() {
        properties.insert(
            "groups".to_string(),
            geojson::JsonValue::Array(group_stack.clone()),
        );
    }

    Ok(properties)
}
