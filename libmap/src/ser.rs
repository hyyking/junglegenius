use geojson::Feature;
use std::io::Write;

use crate::Error;

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
        let features = input
            .into_iter()
            .map(Into::<Feature>::into)
            .collect::<Vec<_>>();

        info!("Writing {} features to geojson", features.len());
        Ok(Some(geojson::ser::to_feature_collection_writer(
            &mut self.writer,
            &features,
        )?))
    }
}

#[derive(Debug)]
pub struct WriteFlexbuffer<W, T> {
    writer: W,
    _s: std::marker::PhantomData<T>,
}

impl<W, T> WriteFlexbuffer<W, T> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _s: std::marker::PhantomData,
        }
    }
}

impl<W, T> crate::pipe::Pipe for WriteFlexbuffer<W, T>
where
    W: Write,
    T: serde::Serialize,
{
    type Input = T;

    type Output = ();

    type Error = crate::Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        let mut s = flexbuffers::FlexbufferSerializer::new();
        input.serialize(&mut s)?;
        Ok(Some(self.writer.write_all(s.view())?))
    }
}
