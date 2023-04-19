mod reader;
mod sampler;
pub mod parse;

use std::collections::HashMap;

pub use reader::{SvgOperation, SvgReader};
pub use sampler::{PathSampler,  PointSampler, IdSampler, LineStringSampler};
use svg::node::Value;

#[derive(Debug, thiserror::Error)]
pub enum SvgError {
    #[error("svg path not found, available attributes: `{attrs:#?}`")]
    PathNotFound { attrs: HashMap<String, Value> },
    #[error("svg group tag group is missing an id, available attributes: `{attrs:#?}`")]
    MissingGroupId { attrs: HashMap<String, Value> },
}