use std::{sync::Arc, time::Duration};

use indexmap::IndexMap;
use mav_param::{Ident, Value};
use mavio::{dialects::common::enums::MavParamType, prelude::V2};
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
    sync::RwLock,
};

// Where our local parameter file is stored, used for restoring
const BACKUP_PARAM_PATH: &'static str = "examples/mavio-client/backup.param";

// Where we will save the parameters we received from the vehicle
const VEHICLE_PARAM_PATH: &'static str = "examples/mavio-client/vehicle.param";

#[derive(Clone, Debug)]
pub struct Parameters {
    map: IndexMap<Ident, Value>,
}

#[derive(Clone, Debug)]
pub struct ArcParameters {
    inner: Arc<RwLock<IndexMap<(u8, u8), Parameters>>>,
}

impl ArcParameters {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(IndexMap::new())),
        }
    }

    pub async fn insert(&self, id: (u8, u8), ident: &str, value: Value) {
        let mut writer = self.inner.write().await;

        match writer.get_mut(&id) {
            Some(params) => {
                params.map.insert(Ident::from_str_truncated(ident), value);
            }
            None => {
                let mut params = Parameters {
                    map: IndexMap::new(),
                };

                params.map.insert(Ident::from_str_truncated(ident), value);
                writer.insert(id, params);
            }
        }
    }

    pub async fn get_param(&self, id: (u8, u8), ident: &str) -> Option<Value> {
        self.inner
            .read()
            .await
            .get(&id)?
            .map
            .get(&Ident::from_str_truncated(ident))
            .cloned()
    }

    pub async fn get_table(&self, id: (u8, u8)) -> Option<Parameters> {
        self.inner.read().await.get(&id).cloned()
    }
}

#[tokio::main]
pub async fn main() {
    let stream = loop {
        match TcpStream::connect("127.0.0.1:14550").await {
            Ok(stream) => break stream,
            _ => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    };

    let parameters = ArcParameters::new();

    let (reader, writer) = tokio::io::split(stream);
    let r = tokio::spawn(reader_half(reader, parameters.clone()));
    let w = tokio::spawn(writer_half(writer, parameters));

    _ = r.await.unwrap();
    _ = w.await.unwrap();
}

pub async fn reader_half(reader: ReadHalf<TcpStream>, parameters: ArcParameters) {
    let reader = mavio::io::TokioReader::new(reader);
    let mut receiver = mavio::io::AsyncReceiver::new::<V2>(reader);

    loop {
        let frame = receiver.recv().await.unwrap();

        use mavio::dialects::common::messages as m;
        match frame.message_id() {
            m::ParamValue::ID => {
                let message = match m::ParamValue::try_from(frame.payload()) {
                    Ok(message) => message,
                    Err(_error) => {
                        println!("client: [error] Unable to decode ParamRequestList payload");
                        continue;
                    }
                };

                let ident = mav_param::Ident::try_from(&message.param_id).unwrap();
                let Some(value) = value_from_bytewise(message.param_value, message.param_type) else {
                    eprintln!("client: [error] Type not supported mav_param value");
                    continue;
                };

                let mav_id = (frame.system_id(), frame.component_id());

                println!("client: [info] Saving: {:?} : {:?}", ident.as_str(), value);

                parameters.insert(mav_id, ident.as_str(), value).await;

                if message.param_count == message.param_index.saturating_add(1) {
                    if let Some(parameters) = parameters.get_table(mav_id).await {
                        save_parameters(VEHICLE_PARAM_PATH, parameters);
                    }
                }
            }
            id => {
                println!("client: [error] unsupported message ID {id}")
            }
        }
    }
}

pub async fn writer_half(writer: WriteHalf<TcpStream>, parameters: ArcParameters) {
    let writer = mavio::io::TokioWriter::new(writer);
    let mut sender = mavio::io::AsyncSender::new::<V2>(writer);

    let mut sequence = 0;

    use mavio::dialects::common::messages as m;
    loop {
        println!("client [info] Sending ParamRequestList message");
        let message = m::ParamRequestList::default();
        let frame = mavio::Frame::builder()
            .component_id(1)
            .system_id(100)
            .sequence(sequence)
            .version(V2)
            .message(&message)
            .unwrap()
            .build();

        if sender.send(&frame).await.is_ok() {
            sequence += 1;
        }

        tokio::time::sleep(Duration::from_secs(5)).await;

        let loaded = load_parameters_from_ini(BACKUP_PARAM_PATH);

        let mav_id = (frame.system_id(), frame.component_id());

        for (key, value) in loaded.iter() {
            // If the value on the vehicle matches, do not send
            match parameters.get_param(mav_id, key).await {
                Some(vehicle_value) if vehicle_value == *value => continue,
                _ => (),
            }

            let message = m::ParamSet {
                target_system: 1,
                target_component: 1,
                param_id: *mav_param::Ident::try_from(key.as_bytes()).unwrap().as_raw(),
                param_value: value.into_bytewise(),
                param_type: match value {
                    Value::U8(_) => MavParamType::Uint8,
                    Value::I8(_) => MavParamType::Int8,
                    Value::U16(_) => MavParamType::Uint16,
                    Value::I16(_) => MavParamType::Int16,
                    Value::U32(_) => MavParamType::Uint32,
                    Value::I32(_) => MavParamType::Int32,
                    Value::F32(_) => MavParamType::Real32,
                },
            };

            let frame = mavio::Frame::builder()
                .component_id(1)
                .system_id(100)
                .sequence(sequence)
                .version(V2)
                .message(&message)
                .unwrap()
                .build();

            println!("client [info] Sending ParamSet message");
            if sender.send(&frame).await.is_ok() {
                sequence += 1;
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// In your client code where you currently save the TOML file:
fn save_parameters(path: impl AsRef<std::path::Path>, parameter_map: Parameters) {
    let mut config = ini::Ini::new();

    // Group parameters by their first segment
    for (key, value) in parameter_map.map.iter() {
        let parts: Vec<&str> = key.as_str().splitn(2, '.').collect();

        let (section, property) = if parts.len() > 1 {
            (Some(parts[0]), parts[1])
        } else {
            (None, parts[0]) // Root section for items without dots
        };

        // Convert value to string
        let type_value_str = match value {
            mav_param::Value::U8(v) => format!("u8 = {v}"),
            mav_param::Value::I8(v) => format!("i8 = {v}"),
            mav_param::Value::U16(v) => format!("u16 = {v}"),
            mav_param::Value::I16(v) => format!("i16 = {v}"),
            mav_param::Value::U32(v) => format!("u32 = {v}"),
            mav_param::Value::I32(v) => format!("i32 = {v}"),
            mav_param::Value::F32(v) => format!("f32 = {v}"),
        };

        // Set the property in the appropriate section
        config.with_section(section).set(property, type_value_str);
    }

    // Write to file
    config
        .write_to_file_opt(
            path,
            ini::WriteOption {
                kv_separator: ": ",
                ..Default::default()
            },
        )
        .unwrap();
}

fn load_parameters_from_ini(
    path: impl AsRef<std::path::Path>,
) -> IndexMap<String, mav_param::Value> {
    let conf = ini::Ini::load_from_file(path).unwrap();
    let mut parameter_map = IndexMap::new();

    for (section, properties) in conf.iter() {
        for (key, ty_val) in properties.iter() {
            // Concatenate section and key if required
            let ident = if let Some(section_name) = section {
                format!("{}.{}", section_name, key)
            } else {
                key.to_string()
            };

            // Split type and value and trim white spaces
            let (ty, val) = ty_val.split_once('=').unwrap();
            let (ty, val) = (ty.trim(), val.trim());

            // Try to parse into the expected type
            let value = match ty {
                "u8" => mav_param::Value::U8(val.parse::<u8>().unwrap()),
                "i8" => mav_param::Value::I8(val.parse::<i8>().unwrap()),
                "u16" => mav_param::Value::U16(val.parse::<u16>().unwrap()),
                "i16" => mav_param::Value::I16(val.parse::<i16>().unwrap()),
                "u32" => mav_param::Value::U32(val.parse::<u32>().unwrap()),
                "i32" => mav_param::Value::I32(val.parse::<i32>().unwrap()),
                "f32" => mav_param::Value::F32(val.parse::<f32>().unwrap()),
                _ => panic!("Invalid type"), // Todo, return error
            };

            parameter_map.insert(ident, value);
        }
    }

    parameter_map
}

pub fn value_from_bytewise(param_value: f32, param_type: MavParamType) -> Option<Value> {
    use mav_param::{Value, value::from_bytewise};
    let value = match param_type {
        MavParamType::Uint8 => Value::U8(from_bytewise(param_value)),
        MavParamType::Int8 => Value::I8(from_bytewise(param_value)),
        MavParamType::Uint16 => Value::U16(from_bytewise(param_value)),
        MavParamType::Int16 => Value::I16(from_bytewise(param_value)),
        MavParamType::Uint32 => Value::U32(from_bytewise(param_value)),
        MavParamType::Int32 => Value::I32(from_bytewise(param_value)),
        MavParamType::Real32 => Value::F32(from_bytewise(param_value)),
        _ => return None,
    };

    Some(value)
}

pub fn value_into_bytewise(value: Value) -> (f32, MavParamType) {
    use mav_param::{Value, value::into_bytewise};
    match value {
        Value::U8(v) => (into_bytewise(v), MavParamType::Uint8),
        Value::I8(v) => (into_bytewise(v), MavParamType::Int8),
        Value::U16(v) => (into_bytewise(v), MavParamType::Uint16),
        Value::I16(v) => (into_bytewise(v), MavParamType::Int16),
        Value::U32(v) => (into_bytewise(v), MavParamType::Uint32),
        Value::I32(v) => (into_bytewise(v), MavParamType::Int32),
        Value::F32(v) => (into_bytewise(v), MavParamType::Real32),
    }
}
