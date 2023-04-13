use std::collections::HashMap;

use svg::node;

use crate::{mapreader::MapOperation, sampler::PathSampler, parse::{RGB, self}};


#[derive(Debug)]
pub enum AppendMode {
    Direct,
    AppendExterior,
    AppendInterior,
    BuildPoly,
}


pub struct IntExtGrouper<T, S>
where
    T: Iterator<Item = MapOperation<S>>,
    S: PathSampler,
{
    reader: T,
    state: Vec<AppendMode>,
    current_id: String,
    current_poly: Vec<S::Sample>,
    current_properties: std::collections::HashMap<String, SampleProperties>,
    current_groups: Vec<String>,
}

impl<T, S> IntExtGrouper<T, S>
where
    T: Iterator<Item = MapOperation<S>>,
    S: PathSampler,
{
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            state: vec![AppendMode::Direct],
            current_id: String::new(),
            current_poly: vec![],
            current_properties: HashMap::new(),
            current_groups: vec![],
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PolySample<S> {
    pub id: String,
    pub poly: Vec<S>,
    pub properties: HashMap<String, SampleProperties>,
    pub groups: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SampleProperties {
    groups: Vec<String>,
    fill: Option<RGB>,
}

impl<T, S> Iterator for IntExtGrouper<T, S>
where
    T: Iterator<Item = MapOperation<S>>,
    S: PathSampler,
{
    type Item = PolySample<S::Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.next()? {
                MapOperation::StartNewGroup(id) => {
                    self.current_groups.push(id.clone());
                    if id == "interior" {
                        self.state.push(AppendMode::AppendInterior);
                    } else if id == "exterior" {
                        self.state.push(AppendMode::AppendExterior)
                    } else {
                        self.current_id = id;
                        self.current_poly = vec![];
                    }
                }

                MapOperation::NewPath(samples, attrs) => {
                    let id = attrs.get("id").map(ToString::to_string).unwrap_or_default();

                    fn fill(fill: &node::Value) -> Option<RGB> {
                        let (_, rgb) =
                            nom::branch::alt((parse::parse_rgb, parse::parse_hex_rgb))(fill)
                                .ok()?;
                        Some(rgb)
                    }
                    let properties = SampleProperties {
                        groups: self.current_groups.clone(),
                        fill: attrs.get("fill").as_deref().and_then(fill),
                    };

                    match self.state.last() {
                        Some(AppendMode::Direct) => {
                            self.current_id = id.clone();
                            self.current_poly = vec![samples];
                            self.current_properties.insert(id, properties);

                            break Some(PolySample {
                                id: self.current_id.clone(),
                                poly: self.current_poly.drain(..).collect(),
                                properties: self
                                    .current_properties
                                    .drain()
                                    .collect::<HashMap<_, _>>(),
                                groups: self.current_groups.clone(),
                            });
                        }
                        Some(AppendMode::AppendInterior) => {
                            self.current_poly.push(samples);
                            self.current_properties
                                .insert("interior".to_string(), properties);
                        }
                        Some(AppendMode::AppendExterior) => {
                            self.current_poly.insert(0, samples);
                            self.current_properties
                                .insert("exterior".to_string(), properties);
                        }
                        _ => {}
                    }
                }
                MapOperation::EndNewGroup => {
                    let _ = self.current_groups.pop();

                    use AppendMode::AppendExterior as E;
                    use AppendMode::AppendInterior as I;
                    let last_two = [
                        self.state.len().saturating_sub(2),
                        self.state.len().saturating_sub(1),
                    ];
                    if let Ok([I, E] | [E, I]) = self.state.get_many_mut(last_two) {
                        self.state.pop();
                        self.state.pop();

                        break Some(PolySample {
                            id: self.current_id.clone(),
                            poly: self.current_poly.drain(..).collect(),
                            properties: self.current_properties.drain().collect::<HashMap<_, _>>(),
                            groups: self.current_groups.clone(),
                        });
                    }
                }
                MapOperation::NotSupported => {}
            }
        }
    }
}
