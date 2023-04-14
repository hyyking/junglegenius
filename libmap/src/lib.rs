#![feature(get_many_mut, let_chains, iterator_try_collect)]

use std::io::Write;

pub mod intextgrouper;
pub mod mapreader;
pub mod parse;
pub mod pipe;
pub mod sampler;
pub mod ser;

use crate::{
    intextgrouper::IntExtGrouper,
    mapreader::SvgReader,
    pipe::{CloneSplit, ConsumeLeft, Pipe, Producer},
    sampler::PointSampler,
};

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

    let mut pipes = svg::open(path, &mut buff).unwrap().feed(
        SvgReader::default()
            .pipe(sampler)
            .pipe(IntExtGrouper::new())
            .pipe(ser::WriteGeojson::new(output)), // .pipe(CloneSplit::new())
                                                   // .pipe(ConsumeLeft::new(ser::WriteGeojson::new(output)))
    );

    Ok(std::iter::from_fn(|| pipes.produce()).for_each(drop))
}
