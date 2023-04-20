use iced::widget::canvas::Frame;
use libmap::maptri::refined::RefinedTesselation;
use std::fmt;

use super::EngineRenderer;

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct DebugFlags: u16 {
        const DRAW_NAV = 1 << 0;
        const DRAW_HULL = 1 << 1;
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

pub fn draw_debug(frame: &mut Frame, renderer: &EngineRenderer) {
    dbg!(renderer.debug_flags);
    
    if renderer.debug_flags.contains(DebugFlags::DRAW_NAV) {
        draw_navmesh(frame, &renderer.store.nav.triangulation)
    }

    if renderer.debug_flags.contains(DebugFlags::DRAW_HULL) {
        for line in renderer.hull.exterior().lines() {
            let start = line.start;
            let end = line.end;
    
            frame.stroke(
                &iced::widget::canvas::Path::line(
                    iced::Point::new(start.x as f32, start.y as f32),
                    iced::Point::new(end.x as f32, end.y as f32),
                ),
                iced::widget::canvas::Stroke::default()
                    .with_color(iced::Color::from_rgb(0.0, 0.0, 1.0))
                    .with_width(2.0),
            )
        }
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
