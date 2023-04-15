use geo::GeoFloat;
use num_traits::{Float, FromPrimitive};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use spade::{
    handles::VoronoiFace, ConstrainedDelaunayTriangulation, InsertionError, Point2, Triangulation,
};

pub trait CentralVoronoiTesselation {
    fn cvt_lloyds_algorithm(self, max_distance2: f64) -> Result<Self, ()>
    where
        Self: Sized;
}

impl CentralVoronoiTesselation for ConstrainedDelaunayTriangulation<Point2<f64>> {
    fn cvt_lloyds_algorithm(mut self, max_distance2: f64) -> Result<Self, ()> {
        loop {
            #[derive(Clone)]
            struct NoneCenteredCellsFold {
                vertex: Vec<Point2<f64>>,
                centroid: Vec<Point2<f64>>,
                max_move: f64,
            }

            let vertices = self.vertices().collect::<Vec<_>>();
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
                )?;

            if non_centered.vertex.is_empty() {
                break Ok(self);
            }

            println!(
                "lloyd's algo iteration | max centroid distance: {:4.4}, vertices: {:<4}",
                non_centered.max_move,
                non_centered.centroid.len()
            );

            let NoneCenteredCellsFold {
                vertex, centroid, ..
            } = non_centered;

            vertex
                .into_iter()
                .try_for_each(|v| {
                    self.remove(self.locate_vertex(v)?.fix());
                    Some(())
                })
                .ok_or(())?;

            centroid
                .into_iter()
                .try_for_each(|c| {
                    self.insert(c)?;
                    Ok::<(), InsertionError>(())
                })
                .map_err(|_| ())?;
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
        spade::SpadeNum + num_traits::float::Float + PartialOrd + FromPrimitive,
{
    let mut exterior = geo::LineString(Vec::with_capacity(face.adjacent_edges().count()));

    for edge in face.adjacent_edges() {
        let vertex = edge.to().position().or(edge.from().position());

        if let Some(pos) = vertex {
            let coord = geo::coord! {
            x: <T::Scalar>::max(<T::Scalar>::min(pos.x, T::Scalar::from(14980.0)), T::Scalar::from(0.0)), y: pos.y.min(T::Scalar::from(14980.0)).max(T::Scalar::from(0.0))};

            exterior.0.push(coord);
        } else {
            return Err(());
        }
    }

    geo::RemoveRepeatedPoints::remove_repeated_points_mut(&mut exterior);
    exterior.close();
    Ok(geo::Polygon::new(exterior, vec![]))
}
