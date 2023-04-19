use std::collections::HashSet;

use geo::{GeoFloat, RemoveRepeatedPoints};
use num_traits::{Float, FromPrimitive};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use spade::{
    handles::VoronoiFace, ConstrainedDelaunayTriangulation, InsertionError, Point2, Triangulation,
};

use crate::{pipe::Pipe, Error};

use super::RefinedTesselation;

pub struct CenterTesselation {
    pub threshold: f64,
}

impl Pipe for CenterTesselation {
    type Input = RefinedTesselation;

    type Output = RefinedTesselation;

    type Error = Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        input
            .cvt_lloyds_algorithm(self.threshold)
            .map(Some)
    }
}

impl CentralVoronoiTesselation for RefinedTesselation {
    #[tracing::instrument("central_voronoi", skip(self), fields(len = self.cdt.num_vertices()))]
    fn cvt_lloyds_algorithm(self, max_distance2: f64) -> Result<Self, Error>
    where
        Self: Sized,
    {
        //self.unconstrained_inner_vertices();
        Ok(self)
    }
}

pub trait CentralVoronoiTesselation {
    fn cvt_lloyds_algorithm(self, max_distance2: f64) -> Result<Self, Error>
    where
        Self: Sized;
}

impl CentralVoronoiTesselation for ConstrainedDelaunayTriangulation<Point2<f64>> {
    #[tracing::instrument(skip(self), fields(len = self.num_vertices()))]
    fn cvt_lloyds_algorithm(mut self, max_distance2: f64) -> Result<Self, Error> {
        let mut i = 0;
        loop {
            if i > 450 {
                break Ok(self);
            }
            #[derive(Clone)]
            struct NoneCenteredCellsFold {
                vertex: Vec<Point2<f64>>,
                centroid: Vec<Point2<f64>>,
                max_move: f64,
            }

            let vertices = self
                .inner_faces()
                .into_iter()
                .map(|v| v.vertices())
                .flatten()
                .collect::<HashSet<_>>();

            let non_centered = vertices
                .into_par_iter()
                .map(|v| -> Result<(Point2<f64>, Point2<f64>, f64), ()> {
                    let centroid = get_centroid(v.as_voronoi_face());
                    let vertex = v.data().clone();
                    let distance = vertex.distance_2(centroid);
                    Ok((vertex, centroid, distance))
                })
                .filter(|data| match data {
                    Ok((_, _, distance)) => distance > &max_distance2,
                    _ => false,
                })
                .try_fold_with(
                    NoneCenteredCellsFold {
                        vertex: vec![],
                        centroid: vec![],
                        max_move: 0.0,
                    },
                    |mut ncf, data| {
                        data.map(|(v, c, d)| {
                            ncf.vertex.push(v);
                            ncf.centroid.push(c);
                            ncf.max_move = ncf.max_move.max(d);
                            ncf
                        })
                    },
                )
                .try_reduce(
                    || NoneCenteredCellsFold {
                        vertex: vec![],
                        centroid: vec![],
                        max_move: 0.0,
                    },
                    |mut a, b| {
                        a.vertex.extend_from_slice(&b.vertex);
                        a.centroid.extend_from_slice(&b.centroid);
                        a.max_move = a.max_move.max(b.max_move);
                        Ok(a)
                    },
                ).map_err(|_| eyre::eyre!("test"))?;

            if non_centered.vertex.is_empty() {
                break Ok(self);
            }

            trace!(
                i = i,
                max_move = non_centered.max_move,
                non_centered = non_centered.centroid.len(),
            );

            let NoneCenteredCellsFold {
                vertex, centroid, ..
            } = non_centered;

            vertex
                .into_iter()
                .try_for_each(|v| self.locate_and_remove(v).map(drop))
                .ok_or(()).map_err(|_| eyre::eyre!("test"))?;

            centroid
                .into_iter()
                .try_for_each(|c| {
                    self.insert(c)?;
                    Ok::<(), InsertionError>(())
                })
                .map_err(|_| ()).map_err(|_| eyre::eyre!("test"))?;

            i += 1;
        }
    }
}

pub fn get_centroid<DE, UE, F, T>(face: VoronoiFace<'_, T, DE, UE, F>) -> Point2<T::Scalar>
where
    T: spade::HasPosition,
    <T as spade::HasPosition>::Scalar:
        spade::SpadeNum + num_traits::float::Float + PartialOrd + FromPrimitive + GeoFloat,
{
    poly_from_voronoi_face(face)
        .and_then(|p| geo::Centroid::centroid(&p).ok_or(()))
        .map(|c| Point2::new(c.x(), c.y()))
        .unwrap()
}

pub fn poly_from_voronoi_face<DE, UE, F, T>(
    face: VoronoiFace<'_, T, DE, UE, F>,
) -> Result<geo::Polygon<<T as spade::HasPosition>::Scalar>, ()>
where
    T: spade::HasPosition,
    <T as spade::HasPosition>::Scalar:
        spade::SpadeNum + num_traits::float::Float + PartialOrd + FromPrimitive + geo::GeoFloat,
{
    let mut exterior = geo::LineString(Vec::with_capacity(face.adjacent_edges().count()));
    for edge in face.adjacent_edges() {
        let a = edge.as_undirected().vertices();

        if let Some(pos) = a[0].position() {
            let coord = geo::coord! {
                x: <T::Scalar>::max(<T::Scalar>::min(pos.x, T::Scalar::from(14980.0)), T::Scalar::from(0.0)),
                y: pos.y.min(T::Scalar::from(14980.0)).max(T::Scalar::from(0.0))
            };
            exterior.0.push(coord)
        }
    }
    exterior.remove_repeated_points_mut();
    exterior.close();
    Ok(geo::Polygon::new(exterior, vec![]))
}
