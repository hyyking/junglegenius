use std::{
    collections::{HashSet, LinkedList},
    ops::Index,
    sync::{Arc, LazyLock},
};

use lyon::{
    math::Point,
};


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
            spade::PositionInTriangulation::OnVertex(v) => {
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
                                chunk.into_iter().max_by(|a, b| {
                                    ((a.x - centroid.x).exp2() + (a.y - centroid.y).exp2())
                                        .total_cmp(
                                            &((b.x - centroid.x).exp2()
                                                + (b.y - centroid.y).exp2()),
                                        )
                                })
                            })
                            .filter(|p| {
                                let xy = Point2::new(p.x as f64, p.y as f64);

                                !store.nav.triangulation.cdt.intersects_constraint(prev, xy)
                                    && !store.nav.triangulation.cdt.intersects_constraint(xy, next)
                                    && (((p.x as f64) - next.x).exp2()
                                        + ((p.y as f64) - next.y).exp2()
                                        + ((p.x as f64) - prev.x).exp2()
                                        + ((p.y as f64) - prev.y).exp2()
                                        < 1.5 * ((prev.x as f64) - next.x).exp2()
                                            + ((prev.y as f64) - next.y).exp2())
                            }),
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

pub fn compute_path(from: Point, target: &Objective, store: &EntityStore) -> Option<PathResult> {
    use spade::{Point2, Triangulation};

    let target_pos = match target {
        Objective::Unit(id) => {
            store
                .position
                .get(store.get_raw_by_id(*id).unwrap().position)
                .unwrap()
                .1
                .point
        }
        Objective::Position(p) => p.clone(),
    }
    .cast::<i64>();

    let instance = PathfindInstance {
        from: PointE {
            x: from.x as f32,
            y: from.y as f32,
        },
        to: PointE {
            x: target_pos.x as f32,
            y: target_pos.y as f32,
        },
        store,
    };

    let target_points = instance.get_target_points()?;

    let result = pathfinding::prelude::astar(
        &instance.from,
        |pos| instance.get_neighbours(pos),
        |v| {
            let h = |v: &PointE| -> i64 {
                ((v.x - instance.to.x) * (v.x - instance.to.x)
                    + (v.y - instance.to.y) * (v.y - instance.to.y)) as i64
            };
            /*
            let h = |v: &Point2D<i64, UnknownUnit>| -> i64 {
                (v.x - target_pos.x).abs() + (v.y - target_pos.y).abs()
            };
             */
            h(v)
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
    )?
    .0;

    return Some(PathResult { result });
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
