#![feature(get_many_mut, let_chains, iterator_try_collect)]

use std::io::Write;

use geojson::Feature;
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

    type Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error>;

    fn pipe<T>(self, other: T) -> ChainedPipe<Self, T, Self::Output, Self::Error>
    where
        Self: Sized,
        T: for<'a> Pipe<Input = Self::Output, Error = Self::Error>,
    {
        ChainedPipe {
            input: self,
            output: other,
            _s: std::marker::PhantomData,
        }
    }

    fn close(&mut self) {}
}

pub trait Producer {
    type Item;
    fn produce(&mut self) -> Option<Self::Item>;

    fn feed<T>(self, other: T) -> ChainedPipe<Self, T, Self::Item, T::Error>
    where
        Self: Sized,
        T: for<'a> Pipe<Input = Self::Item>,
    {
        ChainedPipe {
            input: self,
            output: other,
            _s: std::marker::PhantomData,
        }
    }
}

impl<T> Producer for T
where
    T: Iterator,
{
    type Item = <T as Iterator>::Item;

    fn produce(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::next(self)
    }
}

pub struct ChainedPipe<I, O, Shared, Error> {
    input: I,
    output: O,
    _s: std::marker::PhantomData<(Shared, Error)>,
}

impl<I, O> Producer for ChainedPipe<I, O, I::Item, O::Error>
where
    I: Producer,
    O: for<'a> Pipe<Input = I::Item>,
{
    type Item = Result<O::Output, O::Error>;

    fn produce(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.input.produce() {
            match Pipe::process(&mut self.output, item) {
                output @ Ok(Some(_)) => return output.transpose(),
                err @ Err(_) => return err.transpose(),
                _ => {}
            }
        }
        dbg!("ehe");
        self.output.close();
        None
    }
}

impl<I, O, Shared, Error> Pipe for ChainedPipe<I, O, Shared, Error>
where
    I: Pipe<Output = Shared, Error = Error>,
    O: for<'a> Pipe<Input = Shared, Error = Error>,
{
    type Input = I::Input;
    type Output = O::Output;
    type Error = O::Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        self.input.process(input).and_then(|input| {
            input
                .map(|input| self.output.process(input))
                .transpose()
                .map(Option::flatten)
        })
    }

    
    fn close(&mut self) {
        self.input.close();
        self.output.close();
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

pub struct WriteGeojson<W, T> {
    writer: W,
    features: Vec<Feature>,
    _s: std::marker::PhantomData<T>,
}

impl<W, T> WriteGeojson<W, T> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            features: vec![],
            _s: std::marker::PhantomData,
        }
    }
}

impl<W, T> Pipe for WriteGeojson<W, T>
where
    W: Write,
    T: Into<Feature> + Clone,
{
    type Input = T;

    type Output = T;

    type Error = Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        self.features.push(input.clone().into());
        Ok(Some(input))
    }

    fn close(&mut self) {
        geojson::ser::to_feature_collection_writer(&mut self.writer, &self.features)
            .map_err(|_| Error::GeoJsonWrite)
            .unwrap()
    }
}

pub fn svg2geojson(
    path: impl AsRef<std::path::Path>,
    output: impl Write,
    sampler: PointSampler,
) -> Result<(), Error> {
    let mut buff = String::with_capacity(4096);

    let mut pipes = svg::open(path, &mut buff).unwrap().feed(
        SvgMapReader::default()
            .pipe(sampler)
            .pipe(IntExtGrouper::new())
            .pipe(WriteGeojson::new(output)),
    );

    Ok(std::iter::from_fn(|| pipes.produce()).for_each(drop))

    /*
        .map(|a| {
            a.map(|sample| Feature {
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
        })
        .try_collect()?;

    geojson::ser::to_feature_collection_writer(output, &objects).map_err(|_| Error::GeoJsonWrite)
     */
}
