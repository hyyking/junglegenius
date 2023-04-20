#![feature(
    get_many_mut,
    let_chains,
    iterator_try_collect,
    try_trait_v2,
    try_trait_v2_residual
)]

#[macro_use]
extern crate tracing;

pub mod intextgrouper;
pub mod maptri;
pub mod mesh_mapper;
pub mod pipe;
pub mod ser;
pub mod svg;
pub mod structures;


type Error = eyre::Report;