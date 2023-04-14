#![feature(
    get_many_mut,
    let_chains,
    iterator_try_collect,
    try_trait_v2,
    try_trait_v2_residual
)]

#[macro_use]
extern crate tracing;

use std::io::Write;

pub mod intextgrouper;
pub mod svg;
pub mod pipe;
pub mod ser;

use pipe::TryCollector;

use crate::{
    intextgrouper::IntExtGrouper,
    svg::{SvgReader, PointSampler},
    pipe::{CloneSplit, ConsumeLeft, Pipe, Producer},
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

    let mut pipes = ::svg::open(path, &mut buff)
        .unwrap()
        .feed(
            SvgReader::default()
                .pipe(sampler)
                .pipe(IntExtGrouper::new()),
        )
        .producer()
        .feed(
            TryCollector::new()
                .pipe(CloneSplit::new())
                .pipe(ConsumeLeft::new(ser::WriteGeojson::new(output))),
        );

    Ok(std::iter::from_fn(|| pipes.produce()).for_each(drop))
}
