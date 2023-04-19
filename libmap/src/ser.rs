use std::io::Write;
use geojson::Feature;
use serde::ser::Serialize;

use crate::{Error, maptri::refined::RefinedTesselation};

#[derive(Debug)]
pub struct WriteGeojson<W, T> {
    writer: W,
    _s: std::marker::PhantomData<T>,
}

impl<W, T> WriteGeojson<W, T> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _s: std::marker::PhantomData,
        }
    }
}

impl<W, T> crate::pipe::Pipe for WriteGeojson<W, T>
where
    W: Write,
    T: Into<Feature>,
{
    type Input = Vec<T>;

    type Output = ();

    type Error = Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {

        let features = input.into_iter().map(Into::<Feature>::into).collect::<Vec<_>>();

        info!("Writing {} features to geojson", features.len());
        Ok(Some(geojson::ser::to_feature_collection_writer(&mut self.writer, &features)?))
    }
}


pub struct WriteTesselation;

impl crate::pipe::Pipe for WriteTesselation {
    type Input = RefinedTesselation;

    type Output = ();

    type Error = crate::Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        let mut s = flexbuffers::FlexbufferSerializer::new();
        
        input.serialize(&mut s)?;

        let mut f = std::fs::File::create("navmesh.flat")?;
        f.write_all(s.view())?;
        Ok(Some(()))
    }
}
