use geojson::Feature;

use crate::{intextgrouper::PolySample, pipe::Pipe};

#[derive(Clone)]
pub enum Mesh {
    Wall(PolySample),
    Nav(PolySample),
    Unspecified(PolySample),
}

impl From<Mesh> for Feature {
    fn from(value: Mesh) -> Self {
        match value {
            Mesh::Wall(poly) => Feature::from(poly),
            Mesh::Nav(poly) => Feature::from(poly),
            Mesh::Unspecified(poly) => Feature::from(poly),
        }
    }
}

pub struct MeshMapper {}

impl Pipe for MeshMapper {
    type Input = PolySample;

    type Output = Mesh;

    type Error = crate::Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        let PolySample {
            id,
            poly,
            properties,
            groups,
        } = input;

        let mesh = match groups.get(0).map(String::as_str) {
            Some("walls") => {
                debug!("{id}: {groups:?}");
                Mesh::Wall(PolySample {
                    id,
                    poly,
                    properties,
                    groups,
                })
            }
            Some("nav") => Mesh::Nav(PolySample {
                id,
                poly,
                properties,
                groups,
            }),
            _ => Mesh::Unspecified(PolySample {
                id,
                poly,
                properties,
                groups,
            }),
        };

        Ok(Some(mesh))
    }
}
