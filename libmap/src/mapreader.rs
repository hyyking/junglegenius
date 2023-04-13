use std::{collections::HashMap, ops::Deref};

use lyon_path::Path;
use svg::Parser;

use crate::{
    build_path,
    sampler::{IdSampler, PathSampler},
    Error,
};

pub struct SvgMapReader<'a, S = IdSampler>
where
    S: PathSampler,
{
    parser: Parser<'a>,
    sampler: S,
}

impl<'a> SvgMapReader<'a, IdSampler> {
    pub fn open(
        path: impl AsRef<std::path::Path>,
        buff: &'a mut String,
    ) -> Result<Self, std::io::Error> {
        svg::open(path.as_ref(), buff).map(|parser| Self {
            parser,
            sampler: IdSampler,
        })
    }
}
impl<'a, S> SvgMapReader<'a, S>
where
    S: PathSampler,
{
    pub fn open_with(
        path: impl AsRef<std::path::Path>,
        sampler: S,
        buff: &'a mut String,
    ) -> Result<Self, std::io::Error> {
        svg::open(path.as_ref(), buff).map(|parser| Self { parser, sampler })
    }
}

pub enum MapOperation<S: PathSampler> {
    StartNewGroup(String),
    NewPath(S::Sample, HashMap<String, svg::node::Value>),
    EndNewGroup,
    NotSupported,
}

impl<S> Iterator for SvgMapReader<'_, S>
where
    S: PathSampler,
{
    type Item = MapOperation<S>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next().map(|event| match event {
            svg::parser::Event::Tag("g", svg::node::element::tag::Type::Start, attrs) => {
                MapOperation::StartNewGroup(
                    attrs
                        .get("inkscape:label")
                        .or(attrs.get("id"))
                        .unwrap()
                        .to_string(),
                )
            }
            svg::parser::Event::Tag("g", svg::node::element::tag::Type::End, _) => {
                MapOperation::EndNewGroup
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
