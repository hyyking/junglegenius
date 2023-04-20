use std::collections::HashSet;

use spade::{ConstrainedDelaunayTriangulation, Point2, RefinementParameters, Triangulation, handles::{FixedFaceHandle, InnerTag, VertexHandle}, CdtEdge};

use crate::pipe::Pipe;


pub struct Refine;

impl Pipe for Refine {
    type Input = ConstrainedDelaunayTriangulation<Point2<f64>>;

    type Output = RefinedTesselation;

    type Error = crate::Error;

    #[tracing::instrument("refinement", skip(self, cdt), fields(inner_faces=cdt.num_inner_faces()), err(Debug))]
    fn process(&mut self, mut cdt: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        let refinement = cdt.refine(
            RefinementParameters::new()
                .keep_constraint_edges()
                .exclude_outer_faces(&cdt)
                .with_min_required_area(64.0 * 48.0),
        );
        info!(complete = refinement.refinement_complete);
        trace!(excluded_faces = refinement.excluded_faces.len());

        if refinement.refinement_complete {
            Ok(Some(RefinedTesselation {
                excluded: refinement.excluded_faces,
                cdt,
            }))
        } else {
            Err(eyre::eyre!("refinement was not completed"))
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RefinedTesselation {
    pub(crate) excluded: HashSet<FixedFaceHandle<InnerTag>>,
    pub cdt: ConstrainedDelaunayTriangulation<Point2<f64>>,
}

impl RefinedTesselation {
    pub fn unconstrained_inner_vertices(
        &self,
    ) -> impl Iterator<Item = VertexHandle<'_, Point2<f64>, (), CdtEdge<()>, ()>> + '_ {
        self.cdt
            .inner_faces()
            .filter(|f| !self.excluded.contains(&f.fix()))
            .map(|f| f.vertices())
            .flatten()
            .filter(|v| {
                v.as_voronoi_face()
                    .adjacent_edges()
                    .find(|v| {
                        v.as_delaunay_edge().is_constraint_edge() || 
                        matches!(
                                v.as_undirected().vertices(),
                                [
                                    spade::handles::VoronoiVertex::Outer(_),
                                    spade::handles::VoronoiVertex::Outer(_)
                                ] | [
                                    spade::handles::VoronoiVertex::Inner(_),
                                    spade::handles::VoronoiVertex::Outer(_)
                                ] | [
                                    spade::handles::VoronoiVertex::Outer(_),
                                    spade::handles::VoronoiVertex::Inner(_)
                                ]
                            )
                    })
                    .is_none()
            })
    }
}