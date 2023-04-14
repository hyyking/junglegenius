use lyon_algorithms::walk::{RegularPattern, WalkerEvent};
use lyon_path::Path;

use crate::{svg::SvgOperation, Error};


pub trait PathSampler {
    type Sample;

    fn sample(&self, path: Path) -> Self::Sample;
}

impl<S> crate::pipe::Pipe for S
where
    S: PathSampler,
{
    type Input = SvgOperation<lyon_path::Path>;
    type Output = SvgOperation<S::Sample>;
    type Error = Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        Ok(Some(match input {
            SvgOperation::NewPath(path, attrs) => SvgOperation::NewPath(self.sample(path), attrs),
            SvgOperation::StartNewGroup(g) => SvgOperation::StartNewGroup(g),
            SvgOperation::EndNewGroup => SvgOperation::EndNewGroup,
            SvgOperation::NotSupported => SvgOperation::NotSupported,
        }))
    }
}


pub struct IdSampler;
impl PathSampler for IdSampler {
    type Sample = Path;

    fn sample(&self, path: Path) -> Self::Sample {
        path
    }
}

pub struct PointSampler {
    pub rate: f32,
}

impl PathSampler for PointSampler {
    type Sample = Vec<Vec<f64>>;

    fn sample(&self, path: Path) -> Self::Sample {
        let mut samples = vec![];

        let mut pattern = RegularPattern {
            callback: &mut |event: WalkerEvent| {
                samples.push(vec![event.position.x as f64, event.position.y as f64]);
                true
            },
            interval: self.rate,
        };
        lyon_algorithms::walk::walk_along_path(&path, 0.0, 0.1, &mut pattern);
        if samples.len() > 1 {
            samples.push(samples[0].clone());
        }
        samples
    }
}