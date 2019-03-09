use base64;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde_derive::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum BlueprintObject {
  #[serde(rename = "blueprint")]
  Blueprint(Blueprint),

  #[serde(rename = "blueprint_book")]
  BlueprintBook(BlueprintBook),
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BlueprintBook {
  pub item: String,
  pub label: String,
  pub blueprints: Vec<BlueprintBookEntry>,
  pub active_index: i32,
  pub version: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlueprintBookEntry {
  pub index: i32,
  pub blueprint: Blueprint,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Blueprint {
  pub item: String,
  pub label: String,
  pub entities: Vec<Entity>,

  #[serde(default)]
  pub tiles: Vec<Tile>,

  #[serde(default)]
  pub icons: Vec<Icon>,

  pub version: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Entity {
  pub entity_number: i32,
  pub name: String,
  pub position: Position,
  pub direction: Option<u8>,

  #[serde(default)]
  pub connections: HashMap<u8, Connection>,

  #[serde(default)]
  pub items: HashMap<String, u32>,

  pub recipe: Option<String>,
  pub bar: Option<u8>,
  pub infinity_settings: Option<InfinitySettings>,

  #[serde(rename = "type")]
  pub underground_type: Option<UndergroundBeltOrLoaderType>,

  pub input_priority: Option<SplitterDirection>,
  pub output_priority: Option<SplitterDirection>,
  pub filter: Option<String>,

  #[serde(default)]
  pub filters: Vec<ItemFilter>,

  pub override_stack_size: Option<u8>,
  pub drop_position: Option<Position>,
  pub pickup_position: Option<Position>,
  pub request_filters: Option<Vec<LogisticFilter>>,
  pub request_from_buffers: Option<bool>,
  pub parameters: Option<SpeakerParameter>,
  pub alert_parameters: Option<SpeakerAlertParameter>,
  pub auto_launch: Option<bool>,
  pub color: Option<Color>,
  pub station: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Tile {
  pub name: String,
  pub position: Position,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Icon {
  pub index: i32,
  pub signal: SignalID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SignalID {
  pub name: String,

  #[serde(rename = "type")]
  pub signal_type: SignalType,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SignalType {
  #[serde(rename = "item")]
  Item,

  #[serde(rename = "fluid")]
  Fluid,

  #[serde(rename = "virtual")]
  Virtual,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Position {
  pub x: f64,
  pub y: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Connection {
  pub first_point: ConnectionPoint,
  pub second_point: ConnectionPoint,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConnectionPoint {
  pub red: Vec<ConnectionData>,
  pub green: Vec<ConnectionData>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConnectionData {
  pub entity_id: u8,
  pub circuit_id: u8,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InfinitySettings {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UndergroundBeltOrLoaderType {
  #[serde(rename = "input")]
  Input,

  #[serde(rename = "output")]
  Output,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ItemFilter {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LogisticFilter {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SpeakerParameter {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SpeakerAlertParameter {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Color {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SplitterDirection {
  #[serde(rename = "left")]
  Left,

  #[serde(rename = "right")]
  Right,
}

#[derive(Debug)]
pub struct EncodedBlueprint(pub String);

impl EncodedBlueprint {
  pub fn get_base64(&self) -> &str {
    &self.0[1..]
  }

  pub fn get_version_byte(&self) -> u8 {
    self.0.as_bytes()[0]
  }

  pub fn as_string(&self) -> &String {
    &self.0
  }

  pub fn decode(&self) -> Result<BlueprintObject> {
    let bytes: Vec<u8> = base64::decode(self.get_base64()).unwrap();
    serde_json::from_reader(ZlibDecoder::new(&bytes[..]))
  }
}

impl BlueprintObject {
  pub fn encode(&self) -> Result<EncodedBlueprint> {
    let mut bytes = Vec::new();
    serde_json::to_writer(ZlibEncoder::new(&mut bytes, Compression::best()), self)?;
    let mut result = "0".to_string();
    base64::encode_config_buf(&bytes, base64::STANDARD, &mut result);
    Ok(EncodedBlueprint(result))
  }

  pub fn visit_blueprints(&mut self, mut visitor: impl FnMut(&mut Blueprint)) {
    match self {
      BlueprintObject::Blueprint(blueprint) => (visitor)(blueprint),
      BlueprintObject::BlueprintBook(book) => {
        for BlueprintBookEntry { blueprint, .. } in &mut book.blueprints {
          (visitor)(blueprint)
        }
      }
    }
  }
}

impl Blueprint {
  pub fn renumber_entities(&mut self) {
    for (index, entity) in self.entities.iter_mut().enumerate() {
      entity.entity_number = index as i32 + 1;
    }
  }

  pub fn simple(name: String, entities: Vec<Entity>) -> Blueprint {
    let mut result = Blueprint {
      item: "blueprint".to_string(),
      label: name,
      entities: entities,
      version: 68722819072,
      ..Default::default()
    };
    result.renumber_entities();
    result
  }
}

impl BlueprintBook {
  pub fn simple(name: String, blueprints: Vec<Blueprint>) -> BlueprintBook {
    BlueprintBook {
      item: "blueprint-book".to_string(),
      label: name,
      blueprints: blueprints
        .into_iter()
        .enumerate()
        .map(|(index, blueprint)| BlueprintBookEntry {
          index: index as i32 + 1,
          blueprint,
        })
        .collect(),
      active_index: 0,
      version: 68722819072,
      ..Default::default()
    }
  }
}
