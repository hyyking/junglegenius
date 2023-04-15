use std::collections::HashMap;

use geo::{Polygon, LineString};
use geojson::Feature;

use crate::{pipe::Pipe, svg::parse::RGB, svg::SvgOperation};

#[derive(Debug)]
pub enum AppendMode {
    Direct,
    AppendExterior,
    AppendInterior,
}
#[derive(Debug)]
pub struct IntExtGrouper<S> {
    state: Vec<AppendMode>,
    current_id: String,
    current_poly: Vec<S>,
    current_properties: std::collections::HashMap<String, SampleProperties>,
    current_groups: Vec<String>,
}

impl<S> IntExtGrouper<S> {
    pub fn new() -> Self {
        Self {
            state: vec![AppendMode::Direct],
            current_id: String::new(),
            current_poly: vec![],
            current_properties: HashMap::new(),
            current_groups: vec![],
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PolySample {
    pub id: String,
    pub poly: Polygon,
    pub properties: HashMap<String, SampleProperties>,
    pub groups: Vec<String>,
}

impl From<PolySample> for Feature {
    fn from(sample: PolySample) -> Self {
        let properties = serde_json::to_value(sample.properties)
            .ok()
            .as_ref()
            .and_then(geojson::JsonValue::as_object)
            .cloned();
        Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::from(&sample.poly).into()),
            id: Some(geojson::feature::Id::String(sample.id)),
            properties,
            foreign_members: None,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SampleProperties {
    pub groups: Vec<String>,
    pub fill: Option<RGB>,
}

impl<S> Pipe for IntExtGrouper<S> where Vec<S>: CollectPoly {
    type Input = SvgOperation<S>;
    type Output = PolySample;

    type Error = crate::Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        match input {
            SvgOperation::StartNewGroup(id) => {
                trace!("new group: {id}");
                self.current_groups.push(id.clone());
                if id.as_str() == "interior" {
                    self.state.push(AppendMode::AppendInterior);
                } else if id.as_str() == "exterior" {
                    self.state.push(AppendMode::AppendExterior)
                } else {
                    self.current_id = id;
                    self.current_poly = vec![];
                }
                Ok(None)
            }

            SvgOperation::NewPath(samples, attrs) => {
                let id = attrs.get("id").map(ToString::to_string).unwrap_or_default();

                fn fill(fill: &svg::node::Value) -> Option<RGB> {
                    let (_, rgb) =
                        nom::branch::alt((crate::svg::parse::parse_rgb, crate::svg::parse::parse_hex_rgb))(fill).ok()?;
                    Some(rgb)
                }
                let properties = SampleProperties {
                    groups: self.current_groups.clone(),
                    fill: attrs.get("fill").as_deref().and_then(fill),
                };

                match self.state.last().ok_or(crate::Error::ParseRGB)? {
                    AppendMode::Direct => {
                        trace!("new path: {id}");
                        self.current_id = id.clone();
                        self.current_poly = vec![samples];
                        self.current_properties.insert(id, properties);

                        return Ok(Some(PolySample {
                            id: self.current_id.clone(),
                            poly: self.current_poly.split_off(0).collect_poly(),
                            properties: self.current_properties.drain().collect::<HashMap<_, _>>(),
                            groups: self.current_groups.clone(),
                        }));
                    }
                    AppendMode::AppendInterior => {
                        trace!("collecting interior of: {}", self.current_id);
                        self.current_poly.push(samples);
                        self.current_properties
                            .insert("interior".to_string(), properties);
                    }
                    AppendMode::AppendExterior => {
                        trace!("collecting exterior of: {}", self.current_id);
                        self.current_poly.insert(0, samples);
                        self.current_properties
                            .insert("exterior".to_string(), properties);
                    }
                }
                Ok(None)
            }
            SvgOperation::EndNewGroup => {
                use AppendMode::AppendExterior as E;
                use AppendMode::AppendInterior as I;

                let _ = self.current_groups.pop();

                let last_two = [
                    self.state.len().saturating_sub(2),
                    self.state.len().saturating_sub(1),
                ];
                if let Ok([I, E] | [E, I]) = self.state.get_many_mut(last_two) {
                    self.state.pop();
                    self.state.pop();

                    trace!("new path: {}", self.current_id);
                    return Ok(Some(PolySample {
                        id: self.current_id.clone(),
                        poly: self.current_poly.split_off(0).collect_poly(),
                        properties: self.current_properties.drain().collect::<HashMap<_, _>>(),
                        groups: self.current_groups.clone(),
                    }));
                }
                Ok(None)
            }
            SvgOperation::NotSupported => Ok(None),
        }
    }
}


pub trait CollectPoly {
    fn collect_poly(self) -> Polygon;
}

impl CollectPoly for Vec<LineString> {
    fn collect_poly(mut self) -> Polygon {
        let interior = self.split_off(1);
        Polygon::new(self.pop().unwrap(), interior)
    }
}