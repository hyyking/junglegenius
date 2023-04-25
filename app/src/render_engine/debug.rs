use engine::ecs::{entity::EntityRef, generic::pathfinding::PointE, store::EntityStore};
use iced::widget::canvas::{Cache, Frame, Geometry};
use libmap::maptri::refined::RefinedTesselation;
use spade::{Point2, Triangulation};
use std::{borrow::Borrow, fmt};

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct DebugFlags: u16 {
        const DRAW_NAV = 1 << 0;
        const DRAW_HULL = 1 << 1;
        const CLOSEST_NAV_TO_CURSOR = 1 << 2;
        const MINION_NAV = 1 << 3;
        const PATHFIND_CURSOR = 1 << 4;
    }
}

impl fmt::Debug for DebugFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for DebugFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

pub fn draw_debug(
    cache: &Cache,
    flags: &DebugFlags,
    store: &EntityStore,
    bounds: iced::Rectangle,
    scale: f32,
    cursor: Option<iced::Point>,
) -> Vec<Geometry> {
    let mut frames = vec![];

    frames.push(cache.draw(bounds.size(), |frame| {
        frame.scale(scale);

        if flags.contains(DebugFlags::DRAW_NAV) {
            draw_navmesh(frame, &store.nav.triangulation)
        }

        if flags.contains(DebugFlags::DRAW_HULL) {
            let path = iced::widget::canvas::Path::new(|b| {
                for point in Triangulation::convex_hull(&store.nav.triangulation.cdt) {
                    let from = point.from().position();
                    let to = point.to().position();
                    b.move_to(iced::Point::new(from.x as f32, from.y as f32));
                    b.line_to(iced::Point::new(to.x as f32, to.y as f32));
                }
            });
            frame.stroke(
                &path,
                iced::widget::canvas::Stroke::default()
                    .with_color(iced::Color::from_rgb(0.0, 0.0, 1.0))
                    .with_width(2.0),
            )
        }

        if flags.contains(DebugFlags::MINION_NAV) {
            for path in store.minions().filter_map(|minion| {
                minion.path_to_latest_objective().ok().flatten()
            }) {
                frame.stroke(
                    &unsafe { std::mem::transmute::<_, iced::widget::canvas::Path>(path) },
                    iced::widget::canvas::Stroke::default()
                        .with_color(iced::Color::from_rgb(0.0, 1.0, 0.0))
                        .with_width(1.0),
                )
            }
        }
    }));

    if flags.contains(DebugFlags::CLOSEST_NAV_TO_CURSOR) {
        if let Some(cursor) = cursor.map(|c| spade::Point2::new(c.x as f64, c.y as f64)) {
            let mut frame = Frame::new(bounds.size());
            frame.scale(scale);

            if let Some(ref path) = ajd_face_path(&store.nav.triangulation, cursor) {
                frame.stroke(
                    path,
                    iced::widget::canvas::Stroke::default()
                        .with_color(iced::Color::from_rgb(0.0, 1.0, 0.0))
                        .with_width(1.0),
                )
            }

            frames.push(frame.into_geometry())
        }
    }

    if flags.contains(DebugFlags::PATHFIND_CURSOR) {
        let mut frame = Frame::new(bounds.size());
        frame.scale(scale);
        if let Some(cursor) = cursor {
            use engine::ecs::generic::pathfinding::Objective;
            let Ok(result) = engine::ecs::generic::pathfinding::compute_path(
                lyon::math::Point::new(cursor.x, cursor.y),
                &Objective::Unit(engine::ecs::entity::EntityBuilder::guid(
                    &engine::ecs::structures::turret::TurretIndex::RED_MID_OUTER,
                )),
                store,
            ) else { return frames };

            fn build_path(result: Vec<impl Borrow<PointE>>) -> iced::widget::canvas::Path {
                let path = result
                    .array_windows::<2>()
                    .fold(
                        lyon::path::Path::svg_builder(),
                        |mut builder, [from, to]| {
                            let from = from.borrow();
                            let to = to.borrow();
                            builder.move_to(lyon::math::Point::new(from.x, from.y));
                            builder.line_to(lyon::math::Point::new(to.x, to.y));
                            builder
                        },
                    )
                    .build();
                unsafe { std::mem::transmute::<_, iced::widget::canvas::Path>(path) }
            }
            
            frame.stroke(
                &build_path(result.smooth_path_2(store).collect()),
                iced::widget::canvas::Stroke::default()
                    .with_color(iced::Color::from_rgb(0.0, 0.0, 1.0))
                    .with_width(2.0),
            );
             
            frame.stroke(
                &build_path(result.smooth_path(store).collect()),
                iced::widget::canvas::Stroke::default()
                    .with_color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                    .with_width(1.0),
            );
            
            frame.stroke(
                &build_path(result.result.iter().collect()),
                iced::widget::canvas::Stroke::default()
                    .with_color(iced::Color::from_rgb(0.0, 1.0, 0.0))
                    .with_width(1.0),
            );
            
            frames.push(frame.into_geometry())
            
        }
    }

    frames
}

fn ajd_face_path(
    tesselation: &RefinedTesselation,
    at: Point2<f64>,
) -> Option<iced::widget::canvas::Path> {
    match tesselation.cdt.locate(at) {
        spade::PositionInTriangulation::OnFace(handle) => {
            let face = tesselation.cdt.face(handle);

            let path = iced::widget::canvas::Path::new(|builder| {
                for edge in face
                    .vertices()
                    .into_iter()
                    .map(|v| v.as_voronoi_face())
                    .map(|face| face.adjacent_edges())
                    .flatten()
                {
                    let adj_edges = edge
                        .as_undirected()
                        .vertices()
                        .into_iter()
                        .map(|v| v.out_edges())
                        .flatten()
                        .flatten()
                        .map(|edge| edge.as_undirected().vertices());

                    for [a, b] in adj_edges {
                        if let (Some(a), Some(b)) = (a.position(), b.position()) {
                            builder.move_to(iced::Point::new(a.x as f32, a.y as f32));
                            builder.line_to(iced::Point::new(b.x as f32, b.y as f32));
                        }
                    }
                }
            });

            Some(path)
        }
        _ => None,
    }
}

fn draw_navmesh(frame: &mut Frame, tesselation: &RefinedTesselation) {
    for vertex in tesselation.unconstrained_inner_vertices() {
        let pos = vertex.data();

        frame.fill(
            &iced::widget::canvas::Path::circle(iced::Point::new(pos.x as f32, pos.y as f32), 16.0),
            iced::Color::from_rgb(1.0, 1.0, 0.5),
        );

        vertex.as_voronoi_face().adjacent_edges().for_each(|edge| {
            let [a, b] = edge.as_undirected().vertices();
            let (a, b) = (a.position().unwrap(), b.position().unwrap());

            frame.stroke(
                &iced::widget::canvas::Path::line(
                    iced::Point::new(a.x as f32, a.y as f32),
                    iced::Point::new(b.x as f32, b.y as f32),
                ),
                iced::widget::canvas::Stroke::default()
                    .with_color(iced::Color::from_rgb(0.0, 1.0, 0.0))
                    .with_width(1.0),
            );
        });
    }
}
