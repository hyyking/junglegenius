use libmap::{svg::{LineStringSampler, SvgReader}, pipe::{Pipe, Producer, TryCollector, CloneSplit, ConsumeLeft}, intextgrouper::IntExtGrouper, mesh_mapper::MeshMapper, ser, maptri::{cvt::CenterTesselation, refined::Refine, MapTri}};

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    
    tracing_subscriber::fmt()
        .with_env_filter("libmap=trace,geo=warn")
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .init();

    let mut buff = String::with_capacity(4096);

    let mut pipes = ::svg::open("map.svg", &mut buff)
        .unwrap()
        .feed(
            SvgReader::default()
                .pipe(LineStringSampler { rate: 32.0 })
                .pipe(IntExtGrouper::new())
                .pipe(MeshMapper {}),
        )
        .producer()
        .feed(
            TryCollector::new()
                .pipe(CloneSplit::new())
                .pipe(ConsumeLeft::new(ser::WriteGeojson::new(
                    std::fs::File::create("map.json").unwrap(),
                )))
                .pipe(MapTri::new())
                .pipe(Refine)
                .pipe(CenterTesselation { threshold: 16.0 })
                .pipe(ser::WriteTesselation),
        );

    Ok(std::iter::from_fn(|| pipes.produce()).for_each(drop))
}
