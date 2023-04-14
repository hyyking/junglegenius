use std::io::Write;

use geojson::Feature;

use crate::Error;

#[derive(Debug)]
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
        geojson::ser::to_feature_collection_writer(&mut self.writer, &features)
        .map_err(|_| Error::GeoJsonWrite).map(Some)
    }
}