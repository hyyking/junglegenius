use std::collections::HashMap;

use lyon_path::Winding;

use crate::{
    intextgrouper::PolySample,
    mesh_mapper::Mesh,
    pipe::Producer,
    svg::{LineStringSampler, PathSampler},
};

#[derive(Debug, serde::Deserialize)]
struct StructureDeser {
    guid: u64,
    x: f32,
    y: f32,
    radius: f32,
}

pub struct StructureProducer {
    items: std::vec::IntoIter<StructureDeser>,
    sampler: LineStringSampler,
}

impl StructureProducer {
    pub fn from_file(path: impl AsRef<std::path::Path>, sample_rate: f32) -> Self {
        let file = std::fs::File::open(path).unwrap();

        Self {
            items: serde_json::from_reader::<_, Vec<StructureDeser>>(file)
                .unwrap()
                .into_iter(),
            sampler: LineStringSampler { rate: sample_rate },
        }
    }
}

impl Producer for StructureProducer {
    type Item = Result<crate::mesh_mapper::Mesh, crate::Error>;

    fn produce(&mut self) -> Option<Self::Item> {
        let turret = self.items.next()?;

        let mut builder = lyon_path::Path::builder();
        builder.add_circle(
            lyon_geom::Point::new(turret.x, turret.y),
            turret.radius,
            Winding::Negative,
        );

        Some(Ok(Mesh::Wall(PolySample {
            id: turret.guid.to_string(),
            poly: geo::Polygon::new(self.sampler.sample(builder.build()), vec![]),
            properties: HashMap::new(),
            groups: vec!["walls".to_string()],
        })))
    }
}
