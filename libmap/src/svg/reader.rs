use std::{collections::HashMap, ops::Deref};

use lyon_path::{Path, traits::{SvgPathBuilder, Build}};

use crate::{pipe::Pipe, Error, svg::parse::Operation};

pub enum SvgOperation<S> {
    StartNewGroup(String),
    NewPath(S, HashMap<String, svg::node::Value>),
    EndNewGroup,
    NotSupported,
}

#[derive(Default)]
pub struct SvgReader<'a>(std::marker::PhantomData<&'a ()>);

impl<'a> Pipe for SvgReader<'a> {
    type Input = svg::parser::Event<'a>;
    type Output = SvgOperation<lyon_path::Path>;

    type Error = Error;

    fn process(&mut self, event: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        match event {
            svg::parser::Event::Tag("g", svg::node::element::tag::Type::Start, attrs) => {
                Ok(Some(SvgOperation::StartNewGroup(
                    attrs
                        .get("inkscape:label")
                        .or(attrs.get("id"))
                        .unwrap()
                        .to_string(),
                )))
            }
            svg::parser::Event::Tag("g", svg::node::element::tag::Type::End, _) => {
                Ok(Some(SvgOperation::EndNewGroup))
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
                Ok(Some(SvgOperation::NewPath(path, attrs)))
            }
            _ => Ok(Some(SvgOperation::NotSupported)),
        }
    }
}



pub fn build_path(
    path_string: &str,
    builder: impl SvgPathBuilder + Build<PathType = Path> + 'static,
) -> Result<Path, nom::Err<nom::error::Error<&str>>> {
    crate::svg::parse::path_to_operations(path_string).map(|(_, ops)| {
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
