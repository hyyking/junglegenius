use engine::ecs::entity::TempSerEntity;
use serde::{Serializer, ser::SerializeTuple};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_, store) = engine::MinimapEngine::init();
    
    let mut f = std::fs::File::create("structures.json")?;
    let mut ser = serde_json::ser::Serializer::new(&mut f);

    let mut seq = ser.serialize_seq(None)?;

    for turret in store.turrets().map(TempSerEntity::new) {
        seq.serialize_element(&turret)?;
    }

    for inhib in store.inhibitors().map(TempSerEntity::new) {
        seq.serialize_element(&inhib)?;
    }

    for nexus in store.nexuses().map(TempSerEntity::new) {
        seq.serialize_element(&nexus)?;
    }

    seq.end()?;


    Ok(())
}
