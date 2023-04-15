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
mod mesh_mapper;
pub mod pipe;
pub mod ser;
pub mod svg;
pub mod maptri;

use pipe::TryCollector;

use crate::{
    intextgrouper::IntExtGrouper,
    pipe::{CloneSplit, ConsumeLeft, Pipe, Producer},
    svg::{PointSampler, SvgReader},
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
    sampler: svg::LineStringSampler,
) -> Result<(), Error> {
    let mut buff = String::with_capacity(4096);

    let mut pipes = ::svg::open(path, &mut buff)
        .unwrap()
        .feed(
            SvgReader::default()
                .pipe(sampler)
                .pipe(IntExtGrouper::new())
                .pipe(mesh_mapper::MeshMapper {}),
        )
        .producer()
        .feed(
            TryCollector::new()
                .pipe(CloneSplit::new())
                .pipe(ConsumeLeft::new(ser::WriteGeojson::new(output)))
                .pipe(crate::maptri::MapTri::new()),
        );

    Ok(std::iter::from_fn(|| pipes.produce()).for_each(drop))
}
