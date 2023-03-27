use std::{collections::{LinkedList}};

use iced::{widget::canvas::{Frame, Path, Stroke}, Color, Vector, Size, Point};
use leaguesimulator::{GameState, wave::{WaveSpawner, Wave}, core::{Lane, GameTimer, Team}, unit::minion::MinionType};
use lyon::{algorithms::{measure::{PathMeasurements, SampleType}}, geom::{euclid::{UnknownUnit, Point2D}}};

use crate::minimap::{geometry::MinimapGeometry, model::team_color};

pub struct WaveSpawnerState {
    path: lyon::path::Path,
    measurements: PathMeasurements,
    reversed: bool,
    spawner: WaveSpawner,
    waves: LinkedList<WaveState>,
    prev_timer: GameTimer,
    sink: WaveSink,
}

impl WaveSpawnerState {
    pub fn new(path: lyon::path::Path, team: Team, lane: Lane) -> WaveSpawnerState {
        const DEFAULT_POS: f32 = 0.48;
        let reversed = team == Team::Red;

        let measurements = PathMeasurements::from_path(&path, 0.1);
        let mut sampler = measurements.create_sampler(&path, SampleType::Normalized);
        
        let position = sampler.sample(if reversed { 1.0 - DEFAULT_POS } else { DEFAULT_POS }).position();
        let sink = WaveSink { position, absorbed: false, waves: vec![] };
        dbg!(team, lane);
        Self {
            measurements,
            path,
            reversed,
            spawner: WaveSpawner::from_first_spawn(team, lane).peekable(),
            waves: LinkedList::new(),
            prev_timer: GameTimer::FIRST_SPAWN,
            sink,
        }
    }

    pub fn sink(&self) -> &WaveSink {
        &self.sink
    }

    pub fn move_sink(&mut self, point: Point) {
        self.sink.position = lyon::math::point(point.x, point.y);
        self.sink.project_on_path(&self.path);

        if let Some(wave) = self.waves.front_mut() {
            let ws = wave.with(self.reversed, &self.path, &self.prev_timer).current_pos();
            let point = lyon::math::point(ws.x, ws.y);
            
            let angle = (1.0 - 2.0 * f32::from(self.reversed)) * point.to_vector().angle_to(self.sink.position.to_vector()).get();
            

            if angle > 0.0 {
                wave.draw = false;
                if !self.sink.absorbed {
                    self.sink.absorbed = true;
                    self.sink.absorb(wave.clone().wave);
                }
            } else {
                wave.draw = true;
                if self.sink.absorbed {
                    self.sink.absorbed = false;
                    self.sink.waves.pop();
                }
            }
        }
    }

    pub fn update(&mut self, gamestate: &GameState) {
        if let Some(wave) = self.spawner.peek() && (wave.spawn <= gamestate.timer) {
            let wave = self.spawner.next().unwrap();
            self.waves.push_back(WaveState {wave, position: 0.0, draw: true });
   
        }

        if let Some(wave) = self.waves.front() {
            let ws = wave.with(self.reversed, &self.path, &gamestate.timer).current_pos();
            let point = lyon::math::point(ws.x, ws.y);
            
            let angle = (1.0 - 2.0 * f32::from(self.reversed)) * point.to_vector().angle_to(self.sink.position.to_vector()).get();

            if angle > 0.0 {
                let wave = self.waves.pop_front().unwrap().wave;
                if !self.sink.absorbed {
                    self.sink.absorb(wave);
                }
            }
        }
        
        for wave in self.waves.iter_mut() {
            wave.position += (gamestate.raw_timer() - *self.prev_timer).as_secs_f32() * wave.wave.movespeed(&gamestate.timer) as f32;

        }
        self.prev_timer = gamestate.timer;

        if let Some(first) = self.waves.front() && first.position > (self.measurements.length() - 100.0) {
            self.waves.pop_front();
        }

    }
}

impl MinimapGeometry for WaveSpawnerState {
    fn draw(&self, frame: &mut Frame, gs: &GameState, team: Team) {
        self.sink.draw(frame, gs, team);

        for wave in self.waves.iter().filter(|w| w.draw) {
            wave.with(self.reversed, &self.path, &self.prev_timer).draw(frame, gs, team);
        }
    }

    fn describe(&self, gs: &GameState, point: iced::Point) -> Option<crate::information::Card> {
        let sink = self.sink.describe(gs, point);
        let wave = self.waves.iter().map(|ws| ws.with(self.reversed, &self.path, &self.prev_timer).describe(gs, point)).reduce(Option::or).flatten();
        sink.or(wave)
    }
}


#[derive(Debug, Clone, Copy)]
pub struct WaveState {
    wave: Wave,
    position: f32,
    draw: bool,
}


impl WaveState {

}

pub struct WavePathState<'a> {
    path: &'a lyon::path::Path,
    wave: &'a WaveState,
    timer: &'a GameTimer,
    reversed: bool,
}

impl WavePathState<'_> {
    fn from<'a>(wave: &'a Wave, reversed: bool, path: &'a lyon::path::Path, timer: &'a GameTimer) -> WavePathState<'a> {
        WavePathState { reversed, path, wave, timer}
    }

    fn current_pos(&self) -> iced::Point {
        self.point_from_position(self.wave.position)
    }

    fn minion_paths(&self, offset: f32) -> Vec<(iced::Point, iced::widget::canvas::Path)> {
        let mut current =  self.wave.position;
        let mut minions = vec![];
        for minion in self.wave.wave.minions_types() {
            if current < 0.0 {
                break;
            }
            let point = self.point_from_position(current);
            let radius = match minion {
                MinionType::Melee => 48.0,
                MinionType::Ranged => 48.0,
                MinionType::Siege => 65.0,
                MinionType::SuperMinion => 65.0,
            };

            minions.push((point, Path::circle(point, radius)));
            
            current -= radius + offset;
        }
        minions
    }

    fn point_from_position(&self, position: f32) -> iced::Point { 
        let mut point = iced::Point::new(0.0, 0.0);
        let mut pattern = lyon::algorithms::walk::RegularPattern {
            callback: &mut |event: lyon::algorithms::walk::WalkerEvent| {
                point = iced::Point::new(event.position.x, event.position.y);
                false
            },
            interval: self.wave.wave.movespeed(self.timer) as f32,
        };
        if self.reversed {
            lyon::algorithms::walk::walk_along_path(self.path.reversed(), position, 0.1, &mut pattern);            
        } else {
            lyon::algorithms::walk::walk_along_path(self.path, position, 0.1, &mut pattern);
        }
        point
    }
}

impl MinimapGeometry for WavePathState<'_> {
    fn draw(&self, frame: &mut Frame, _gs: &GameState, team: Team) {
        for (_, path) in self.minion_paths(80.0) {
            frame.fill(
                &path,
                team_color(team),
            );
        }
       
    }

    fn describe(&self, _: &GameState, point: iced::Point) -> Option<crate::information::Card> {
        for (minion, _) in self.minion_paths(80.0) {
            let bb = iced::Rectangle::new(minion - Vector::new(60.0, 60.0), Size::new(120.0, 120.0));
            if bb.contains(point) {
                return Some(crate::information::Card::Wave { wave: self.wave.wave });
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct WaveSink {
    pub position: Point2D<f32, UnknownUnit>,
    absorbed: bool,
    waves: Vec<Wave>,
}


impl WaveSink {
    pub fn selection_area(&self) -> iced::Rectangle {
        let pad = 20.0;
        let size = 100.0 + pad;
        iced::Rectangle {
            x: self.position.x - (size / 2.0),
            y: self.position.y - (size / 2.0),
            width: size,
            height: size,
        }
    }

    pub fn project_on_path(&mut self, path: &lyon::path::Path) {
        let sink_pos = self.position;

        let (mut distance, mut point) = (f32::MAX, lyon::math::point(0.0, 0.0));
        for event in path.iter() {
            match event {
                lyon::path::Event::Line { from, to } => {
                    let segment = lyon::geom::LineSegment{from, to};
                    
                    let tpoint = segment.closest_point(sink_pos);
                    let tdistance = tpoint.distance_to(sink_pos);
                    if tdistance < distance {
                        point = tpoint;
                        distance = tdistance
                    }
                },
                lyon::path::Event::Quadratic { from, ctrl, to } => {
                    let segment = lyon::geom::QuadraticBezierSegment {from, ctrl, to};

                    let t = segment.closest_point(sink_pos);
                    let tpoint = lyon::math::point(segment.x(t), segment.y(t));
                    let tdistance = tpoint.distance_to(sink_pos);
                    if tdistance < distance {
                        point = tpoint;
                        distance = tdistance
                    }
                },
                _ => {}
            }
        }
        self.position = point;
    }

    pub fn as_card(&self) -> crate::information::Card {
        crate::information::Card::Text { text: format!("{:#?}", self) }
    }

    pub fn absorb(&mut self, wave: Wave) {
        self.waves.push(wave);
    }
}

impl MinimapGeometry for WaveSink {
    fn draw(&self, frame: &mut Frame, _gs: &GameState, team: Team) {

        frame.fill(
            &Path::circle(iced::Point::new(self.position.x, self.position.y), 100.0),
            team_color(team),
        );
        frame.stroke(
            &Path::circle(iced::Point::new(self.position.x, self.position.y), 100.0),
            Stroke::default().with_color(Color::BLACK).with_width(2.0),
        );  
    }

    fn describe(&self, _: &GameState, point: Point) -> Option<crate::information::Card> {
        self.selection_area().contains(point).then_some(self.as_card())
    }
}