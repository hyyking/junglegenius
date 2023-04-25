use std::{
    collections::{HashSet, LinkedList},
    ops::Index,
    sync::{Arc, LazyLock},
};

use lyon::math::Point;

use crate::{
    core::{GameTimer, Lane, Team},
    ecs::store::EntityStore,
};

pub static LANE_PATHS: LazyLock<LanePaths> = LazyLock::new(|| LanePaths {
    paths: [
        Arc::new(crate::core::top_lane_path(crate::core::Team::Blue)),
        Arc::new(crate::core::mid_lane_path(crate::core::Team::Blue)),
        Arc::new(crate::core::bot_lane_path(crate::core::Team::Blue)),
        Arc::new(crate::core::top_lane_path(crate::core::Team::Red)),
        Arc::new(crate::core::mid_lane_path(crate::core::Team::Red)),
        Arc::new(crate::core::bot_lane_path(crate::core::Team::Red)),
    ],
});

pub struct LanePaths {
    paths: [Arc<lyon::path::Path>; 6],
}

impl Index<(Team, Lane)> for LanePaths {
    type Output = Arc<lyon::path::Path>;

    fn index(&self, index: (Team, Lane)) -> &Self::Output {
        match index {
            (Team::Blue, Lane::Top) => &self.paths[0],
            (Team::Blue, Lane::Mid) => &self.paths[1],
            (Team::Blue, Lane::Bot) => &self.paths[2],
            (Team::Red, Lane::Top) => &self.paths[3],
            (Team::Red, Lane::Mid) => &self.paths[4],
            (Team::Red, Lane::Bot) => &self.paths[5],

            (Team::Red, Lane::Nexus) | (Team::Blue, Lane::Nexus) => unreachable!(),
        }
    }
}

pub enum Objective {
    Unit(crate::ecs::UnitId),
    Position(Point),
}

impl Objective {
    pub fn to_position(&self, store: &EntityStore) -> Result<PointE, ComputeError> {
        match self {
            Objective::Unit(id) => {
                let point = store
                    .position
                    .get(store.get_raw_by_id(*id).unwrap().position)
                    .ok_or(ComputeError::TargetNotFound)?
                    .1
                    .point;
                Ok(PointE {
                    x: point.x,
                    y: point.y,
                })
            }
            Objective::Position(p) => Ok(PointE { x: p.y, y: p.y }),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct PointE {
    pub x: f32,
    pub y: f32,
}

impl std::hash::Hash for PointE {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_be_bytes().hash(state);
        self.y.to_be_bytes().hash(state);
    }
}
impl Eq for PointE {}

pub struct PathfindInstance<'a> {
    from: PointE,
    to: PointE,
    store: &'a EntityStore,
}

type TargetVertices<'a> = HashSet<
    spade::handles::FaceHandle<
        'a,
        spade::handles::InnerTag,
        spade::Point2<f64>,
        (),
        spade::CdtEdge<()>,
        (),
    >,
>;

impl<'a> PathfindInstance<'a> {
    fn get_neighbours(&self, vertex: &PointE) -> impl Iterator<Item = (PointE, i64)> + '_ {
        use spade::{Point2, Triangulation};

        let position = self
            .store
            .nav
            .triangulation
            .cdt
            .locate(Point2::new(vertex.x as f64, vertex.y as f64));

        match position {
            spade::PositionInTriangulation::OnFace(face) => {
                let face = self.store.nav.triangulation.cdt.face(face);

                let vertices = face.adjacent_edges().into_iter().filter_map(|edge| {
                    self.store
                        .nav
                        .triangulation
                        .keep_vertex(&edge.from())
                        .then_some(edge.from())
                });

                Box::new(
                    vertices
                        .clone()
                        .flat_map(|v| v.as_voronoi_face().adjacent_edges())
                        .flat_map(|edge| edge.as_undirected().vertices())
                        .filter_map(|v| v.position())
                        .chain(vertices.clone().map(|v| v.position())),
                ) as Box<dyn Iterator<Item = Point2<f64>>>
            }
            spade::PositionInTriangulation::OnVertex(v)
                if self
                    .store
                    .nav
                    .triangulation
                    .keep_vertex(&self.store.nav.triangulation.cdt.vertex(v)) =>
            {
                let v = self.store.nav.triangulation.cdt.vertex(v);
                Box::new(
                    v.as_voronoi_face()
                        .adjacent_edges()
                        .flat_map(|edge| edge.as_undirected().vertices())
                        .filter_map(|v| v.position()),
                )
            }
            spade::PositionInTriangulation::OnEdge(e) => {
                let e = self.store.nav.triangulation.cdt.directed_edge(e);
                Box::new(
                    e.vertices()
                        .into_iter()
                        .filter(|v| self.store.nav.triangulation.keep_vertex(&v))
                        .flat_map(|v| v.as_voronoi_face().adjacent_edges())
                        .flat_map(|edge| edge.as_undirected().vertices())
                        .filter_map(|v| v.position()),
                )
            }
            _ => Box::new(std::iter::empty()),
        }
        .map(|p| {
            (
                PointE {
                    x: p.x as f32,
                    y: p.y as f32,
                },
                1,
            )
        })
    }
    fn get_target_points(&self) -> Option<TargetVertices> {
        use spade::Triangulation;

        let res = self
            .store
            .nav
            .triangulation
            .cdt
            .locate(spade::Point2::new(self.to.x as f64, self.to.y as f64));

        let target_face = match res {
            spade::PositionInTriangulation::OnFace(face) => {
                self.store.nav.triangulation.cdt.face(face)
            }
            _ => return None,
        };

        Some(
            target_face
                .vertices()
                .into_iter()
                .flat_map(|v| v.as_voronoi_face().adjacent_edges())
                .flat_map(|e| e.as_undirected().vertices())
                .flat_map(|v| v.out_edges().unwrap().into_iter())
                .flat_map(|edge| edge.as_undirected().vertices())
                .map(|v| v.as_delaunay_face().unwrap())
                .collect::<HashSet<_>>(),
        )
    }
}

pub struct PathResult {
    pub result: Vec<PointE>,
}

#[derive(Debug, Clone, Copy)]
pub enum ComputeError {
    TargetNotFound,
    NoTargetPoints,
    NoPathFound,
}

impl PathResult {
    pub fn smooth_path(&self, store: &EntityStore) -> impl Iterator<Item = PointE> {
        use spade::Point2;

        let mut result = self.result.iter();

        let Some(init) = result.next().cloned() else {return Box::new(std::iter::empty()) as Box<dyn Iterator<Item = PointE>> };

        let (result, last, _) = result.fold(
            (vec![init], init, vec![]),
            |(mut ps, init, mut ignored), p| {
                let prev = ps
                    .last()
                    .map(|p| Point2::new(p.x as f64, p.y as f64))
                    .unwrap();
                let next = Point2::new(p.x as f64, p.y as f64);

                if store
                    .nav
                    .triangulation
                    .cdt
                    .intersects_constraint(prev, next)
                {
                    ps.push(init);
                } else {
                    ignored.push(p.clone());
                }

                (ps, p.clone(), ignored)
            },
        );

        Box::new(result.into_iter().chain(Some(last)))
    }

    pub fn smooth_path_2(&self, store: &EntityStore) -> impl Iterator<Item = PointE> {
        use spade::Point2;

        let mut result = self.result.iter();

        let Some(init) = result.next().cloned() else {return Box::new(std::iter::empty()) as Box<dyn Iterator<Item = PointE>> };

        let (result, last, _) = result.fold(
            (vec![init], init, vec![]),
            |(mut ps, init, mut ignored), p| {
                let prev = ps
                    .last()
                    .map(|p| Point2::new(p.x as f64, p.y as f64))
                    .unwrap();
                let next = Point2::new(p.x as f64, p.y as f64);

                if store
                    .nav
                    .triangulation
                    .cdt
                    .intersects_constraint(prev, next)
                {
                    let a = PointE {
                        x: (prev.x - next.x) as f32,
                        y: (prev.y - next.y) as f32,
                    };

                    ps.extend(
                        ignored
                            .drain(..)
                            .array_chunks()
                            .filter_map(|chunk: [PointE; 5]| {
                                let centroid = PointE {
                                    x: chunk.iter().map(|a| a.x).sum::<f32>()
                                        / (chunk.len() as f32),
                                    y: chunk.iter().map(|a| a.y).sum::<f32>()
                                        / (chunk.len() as f32),
                                };
                                Some(centroid)
                            })
                            .filter(|p: &PointE| {
                                let line_to = Point2::new(p.x as f64, p.y as f64);
                                !store
                                    .nav
                                    .triangulation
                                    .cdt
                                    .intersects_constraint(prev, line_to)
                                    && !store
                                        .nav
                                        .triangulation
                                        .cdt
                                        .intersects_constraint(next, line_to)
                            })
                            .min_by(|poss: &PointE, poss2| {
                                let b = PointE {
                                    x: (prev.x as f32 - poss.x),
                                    y: (prev.y as f32 - poss.y),
                                };
                                let c = PointE {
                                    x: (prev.x as f32 - poss2.x),
                                    y: (prev.y as f32 - poss2.y),
                                };
                                (a.x * b.x + a.y * b.y).total_cmp(&(a.x * c.x + a.y * c.y))
                            })
                    );
                    ps.push(init);
                } else {
                    ignored.push(p.clone());
                }

                (ps, p.clone(), ignored)
            },
        );

        Box::new(result.into_iter().chain(Some(last)))
    }
}

pub fn compute_path(
    from: Point,
    target: &Objective,
    store: &EntityStore,
) -> Result<PathResult, ComputeError> {
    use spade::{Point2, Triangulation};

    let instance = PathfindInstance {
        from: PointE {
            x: from.x as f32,
            y: from.y as f32,
        },
        to: target.to_position(store)?,
        store,
    };

    let target_points = instance
        .get_target_points()
        .ok_or(ComputeError::NoTargetPoints)?;

    let result = pathfinding::prelude::astar(
        &instance.from,
        |pos| instance.get_neighbours(pos),
        |v| {
            let h = |v: &PointE| -> i64 {
                ((v.x - instance.to.x) * (v.x - instance.to.x)
                    + (v.y - instance.to.y) * (v.y - instance.to.y))
                    .floor() as i64
            };
            h(v) / 4
        },
        |v| match store
            .nav
            .triangulation
            .cdt
            .locate(Point2::new(v.x as f64, v.y as f64))
        {
            spade::PositionInTriangulation::OnFace(face) => {
                target_points.contains(&store.nav.triangulation.cdt.face(face))
            }
            _ => false,
        },
    )
    .ok_or(ComputeError::NoPathFound)?
    .0;

    return Ok(PathResult { result });
}

pub struct PathfindingComponent {
    pub(crate) path: Pathfinding,
    pub(crate) position: f32,
    pub(crate) speed: f32,
    pub(crate) objectives: LinkedList<Objective>,
}

#[derive(Debug)]
pub enum PathfindError {
    EndReached(lyon::math::Point),
}

impl PathfindingComponent {
    pub fn add_objective(&mut self, objective: Objective) {
        self.objectives.push_front(objective)
    }

    pub fn is_static(&self) -> bool {
        matches!(self.path, Pathfinding::Static)
    }

    pub fn no_path() -> Self {
        Self {
            path: Pathfinding::Static,
            position: 0.0,
            speed: 0.0,
            objectives: LinkedList::new(),
        }
    }

    pub fn persistent(path: Arc<lyon::path::Path>, speed: f32) -> Self {
        Self {
            path: Pathfinding::Persistent(path),
            position: 0.0,
            speed,
            objectives: LinkedList::new(),
        }
    }

    pub fn offset_position(mut self, offset: f32) -> Self {
        self.position += offset;
        self
    }
}

pub enum Pathfinding {
    Static,
    Persistent(Arc<lyon::path::Path>),
    Dynamic {
        path: lyon::path::Path,
        start: GameTimer,
        end: GameTimer,
    },
}
