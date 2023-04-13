use std::{collections::HashMap, ops::Deref};

use lyon_path::Path;

use crate::{build_path, sampler::PathSampler, Error, Pipe};

pub enum MapOperation<S> {
    StartNewGroup(String),
    NewPath(S, HashMap<String, svg::node::Value>),
    EndNewGroup,
    NotSupported,
}


impl<S> Pipe for S
where
    S: PathSampler,
{
    type Input = MapOperation<lyon_path::Path>;
    type Output = MapOperation<S::Sample>;
    type Error = Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        Ok(Some(match input {
            MapOperation::NewPath(path, attrs) => MapOperation::NewPath(self.sample(path), attrs),
            MapOperation::StartNewGroup(g) => MapOperation::StartNewGroup(g),
            MapOperation::EndNewGroup => MapOperation::EndNewGroup,
            MapOperation::NotSupported => MapOperation::NotSupported,
        }))
    }
}

#[derive(Default)]
pub struct SvgMapReader<'a>(std::marker::PhantomData<&'a ()>);

impl<'a> Pipe for SvgMapReader<'a>
where
    Self: 'a,
{
    type Input = svg::parser::Event<'a>;
    type Output = MapOperation<lyon_path::Path>;

    type Error = Error;

    fn process(&mut self, event: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        match event {
            svg::parser::Event::Tag("g", svg::node::element::tag::Type::Start, attrs) => {
                Ok(Some(MapOperation::StartNewGroup(
                    attrs
                        .get("inkscape:label")
                        .or(attrs.get("id"))
                        .unwrap()
                        .to_string(),
                )))
            }
            svg::parser::Event::Tag("g", svg::node::element::tag::Type::End, _) => {
                Ok(Some(MapOperation::EndNewGroup))
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
                Ok(Some(MapOperation::NewPath(path, attrs)))
            }
            _ => Ok(Some(MapOperation::NotSupported)),
        }
    }
}
