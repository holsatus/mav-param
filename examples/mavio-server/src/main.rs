use std::{sync::Arc, time::Duration};

use mav_param::Value;
use mavio::{
    dialects::common::enums::{MavCmd, MavParamType, MavResult},
    dialects::common::messages as m,
    prelude::V2,
    protocol::{IntoPayload, Payload},
};

use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
    sync::{
        RwLock,
        mpsc::{Receiver, Sender},
    },
};

// A nested struct of parameters
#[derive(mav_param::Tree, Default)]
pub struct Parameters {
    mav: MavlinkParameters,
    rate: ControlParameters,
    angl: ControlParameters,
    rc: RcMapParameters,
}

// A nested struct of parameters
#[derive(mav_param::Tree, Default)]
struct MavlinkParameters {
    timeout_ms: u16,
    id: MavlinkId,
    flags: Flags,
}

// Another nested tree of parameters
// with some renamed entries.
#[derive(mav_param::Tree)]
struct MavlinkId {
    #[param(rename = "sys")]
    system_id: u8,
    #[param(rename = "com")]
    component_id: u8,
}

impl Default for MavlinkId {
    fn default() -> Self {
        Self {
            system_id: 1,
            component_id: 1,
        }
    }
}

// A "transparent" unit struct, makes
// working with the bitflags crate easy!
#[derive(mav_param::Node, Default)]
struct Flags(u8);

bitflags::bitflags! {
    impl Flags: u8 {
        const EMIT_HEARTBEAT = 1 << 0;
        // .. and so on
    }
}

#[derive(mav_param::Tree, Default)]
struct ControlParameters {
    #[param(rename = "rol")]
    roll: AxisParameters,
    #[param(rename = "pit")]
    pitch: AxisParameters,
    #[param(rename = "yaw")]
    yaw: AxisParameters,
    /// Slewrate limiter for reference signal
    ref_slew: f32,
    /// Low-pass filter for reference signal
    ref_lp: f32,
}

#[derive(mav_param::Tree, Default)]
struct AxisParameters {
    /// Proportional gain
    p: Gain,
    /// Integral gain
    i: Gain,
    /// Derivative gain
    d: Gain,
    /// Configuration flags
    cfg: AxisFlags,
    /// Time-constant of D-term LP filter
    dtau: f32,
    /// Prediction model time-constant
    pred: f32,
    /// Complementary filter time constant
    comp: f32,
}

#[derive(mav_param::Node, Default)]
struct Gain(f32);

#[derive(mav_param::Node, Default)]
struct AxisFlags(u8);

bitflags::bitflags! {
    impl AxisFlags: u8 {
        const D_TERM_LP = 1 << 0;
        const REF_SLEW = 1 << 1;
        const COMP_PRED = 1 << 2;
    }
}

#[derive(mav_param::Tree)]
pub struct RcMapParameters {
    #[param(rename = "ch")]
    channel: [Binding; 16],
}

#[repr(u8)]
#[derive(mav_param::Enum, Default, Clone)]
enum Binding {
    #[default]
    None = 0,
    Analog(AnalogBinding),
    Digital([DigitalBinding; 3]),
}

#[derive(mav_param::Tree, Default, Clone)]
struct AnalogBinding {
    flags: AnalogFlags,
    ax: AnalogAxis,
    min: u16,
    max: u16,
}

#[derive(mav_param::Node, Default, Clone, Copy)]
struct AnalogFlags(u8);

bitflags::bitflags! {
    impl AnalogFlags: u8 {
        const REVERSE = 1 << 0;
        const HALF_RANGE = 1 << 1;
    }
}

#[repr(u8)]
#[derive(mav_param::Enum, Default, Clone, Copy)]
enum AnalogAxis {
    #[default]
    None = 0,
    Pitch,
    Roll,
    Yaw,
    Throttle,
}

#[derive(mav_param::Tree, Default, Clone)]
struct DigitalBinding {
    #[param(rename = "evt")]
    event: DigitalEvent,
    #[param(rename = "val")]
    value: u16,
}

#[repr(u16)]
#[derive(mav_param::Enum, Default, Clone, Copy)]
enum DigitalEvent {
    #[default]
    None = 0,
    ArmVehicle,
    DisarmVehicle,
    DoAccCalibrate,
    DoGyrCalibrate,
    DoMagCalibrate,
    SetModeRate,
    SetModeAngle,
}

impl Default for RcMapParameters {
    fn default() -> RcMapParameters {
        const CONST_INIT: Binding = Binding::None;
        let mut channels = [CONST_INIT; 16];

        channels[0] = Binding::Analog(AnalogBinding {
            flags: AnalogFlags::empty(),
            ax: AnalogAxis::Pitch,
            min: 180,
            max: 1820,
        });

        channels[1] = Binding::Analog(AnalogBinding {
            flags: AnalogFlags::REVERSE,
            ax: AnalogAxis::Roll,
            min: 180,
            max: 1820,
        });

        channels[2] = Binding::Analog(AnalogBinding {
            flags: AnalogFlags::HALF_RANGE,
            ax: AnalogAxis::Throttle,
            min: 180,
            max: 1820,
        });

        channels[3] = Binding::Analog(AnalogBinding {
            flags: AnalogFlags::empty(),
            ax: AnalogAxis::Yaw,
            min: 180,
            max: 1820,
        });

        channels[4] = Binding::Digital([
            DigitalBinding {
                event: DigitalEvent::ArmVehicle,
                value: 250,
            },
            DigitalBinding {
                event: DigitalEvent::DisarmVehicle,
                value: 500,
            },
            DigitalBinding {
                event: DigitalEvent::None,
                value: 750,
            },
        ]);

        RcMapParameters { channel: channels }
    }
}

#[tokio::main]
pub async fn main() {
    let params = Arc::new(RwLock::new(Parameters::default()));

    println!("server: [info] Listening on TCP 127.0.0.1:14550");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:14550")
        .await
        .unwrap();
    let (stream, _) = listener.accept().await.unwrap();
    let (reader, writer) = tokio::io::split(stream);

    let (tx, rx) = tokio::sync::mpsc::channel(4);

    println!("server: [info] Starting server (Vehicle/MAV)");
    tokio::spawn(reader_half(reader, params.clone(), tx.clone()));
    tokio::spawn(writer_half(writer, params, rx));

    loop {
        let message = m::Heartbeat::default();
        let encoded = message.encode(mavio::prelude::MavLinkVersion::V2).unwrap();
        tx.send((encoded, m::Heartbeat::ID, m::Heartbeat::crc_extra()))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

pub async fn reader_half(
    reader: ReadHalf<TcpStream>,
    params: Arc<RwLock<Parameters>>,
    tx: Sender<(Payload, u32, u8)>,
) {
    let reader = mavio::io::TokioReader::new(reader);
    let mut receiver = mavio::io::AsyncReceiver::new::<V2>(reader);

    while let Ok(frame) = receiver.recv().await {
        match frame.message_id() {
            m::ParamRequestList::ID => {
                if m::ParamRequestList::try_from(frame.payload()).is_err() {
                    println!("server: [error] Unable to decode ParamRequestList payload");
                    continue;
                } else {
                    println!("server: [info] Received ParamRequestList, sending all parameters");
                }

                let params = params.read().await;
                let param_count = mav_param::param_iter(&*params).count();

                for (param_index, param) in mav_param::param_iter(&*params).enumerate() {
                    let param = param.unwrap();
                    println!("server: [info] Sending parameter: {param:?}");

                    let (param_value, param_type) = value_into_bytewise(param.value);

                    let message = m::ParamValue {
                        param_id: *param.ident.as_raw(),
                        param_value,
                        param_type,
                        param_count: param_count as u16,
                        param_index: param_index as u16,
                    };

                    _ = tx
                        .send((
                            message.encode(mavio::prelude::MavLinkVersion::V2).unwrap(),
                            m::ParamValue::ID,
                            m::ParamValue::crc_extra(),
                        ))
                        .await;
                }
                println!("server: [info] Done sending {} parameters", param_count);
            }
            m::ParamSet::ID => {
                let param_set = match m::ParamSet::try_from(frame.payload()) {
                    Ok(param_set) => param_set,
                    Err(_) => {
                        println!("server: [error] Unable to decode ParamSet payload");
                        continue;
                    }
                };

                // Set the value
                let ident = mav_param::Ident::try_from(&param_set.param_id).unwrap();

                let mut param_writer = params.write().await;

                let Some(value) = value_from_bytewise(param_set.param_value, param_set.param_type)
                else {
                    eprintln!("server: [error] ");
                    continue;
                };

                if mav_param::set_value(&mut *param_writer, ident.as_str(), value).is_none() {
                    println!("server: [error] No parameter with ident {}", ident.as_str());
                    continue;
                };

                println!(
                    "server: [info] Received ParamSet message for {} = {:?}",
                    ident.as_str(),
                    value
                );

                // Respond
                let message = m::ParamValue {
                    param_id: *ident.as_raw(),
                    param_value: param_set.param_value,
                    param_type: param_set.param_type,

                    // Note, since we have not climbed the parameter
                    // tree, we do not know exactly which index this is,
                    // so we use the MIN and  MAX value to signal that
                    // these values are not valid.
                    param_count: u16::MIN,
                    param_index: u16::MAX,
                };

                _ = tx
                    .send((
                        message.encode(mavio::prelude::MavLinkVersion::V2).unwrap(),
                        m::ParamValue::ID,
                        m::ParamValue::crc_extra(),
                    ))
                    .await;
            }
            m::CommandLong::ID => {
                let command = match m::CommandLong::try_from(frame.payload()) {
                    Ok(command) => {
                        println!("server: [info] Received command: {:?}", command);
                        command
                    }
                    Err(_) => {
                        println!("server: [error] Unable to decode ParamSet payload");
                        continue;
                    }
                };

                let command_accept = async |command| {
                    let message = m::CommandAck {
                        command,
                        result: MavResult::Accepted,
                        progress: u8::MAX,
                        result_param2: 0,
                        target_system: 0,
                        target_component: 1,
                    };

                    _ = tx
                        .send((
                            message.encode(mavio::prelude::MavLinkVersion::V2).unwrap(),
                            m::CommandAck::ID,
                            m::CommandAck::crc_extra(),
                        ))
                        .await;
                };

                let command_unsupported = async |command| {
                    let message = m::CommandAck {
                        command,
                        result: MavResult::Unsupported,
                        progress: u8::MAX,
                        result_param2: 0,
                        target_system: 0,
                        target_component: 1,
                    };

                    _ = tx
                        .send((
                            message.encode(mavio::prelude::MavLinkVersion::V2).unwrap(),
                            m::CommandAck::ID,
                            m::CommandAck::crc_extra(),
                        ))
                        .await;
                };

                match command.command {
                    MavCmd::RequestMessage => match command.param1 as u32 {
                        m::AutopilotVersion::ID => {
                            let message = m::AutopilotVersion::default();

                            _ = tx
                                .send((
                                    message.encode(mavio::prelude::MavLinkVersion::V2).unwrap(),
                                    m::AutopilotVersion::ID,
                                    m::AutopilotVersion::crc_extra(),
                                ))
                                .await;

                            command_accept(command.command).await;

                            continue;
                        }
                        m::ProtocolVersion::ID => {
                            let mut message = m::ProtocolVersion::default();
                            message.version = 200;
                            message.min_version = 200;
                            message.max_version = 200;

                            _ = tx
                                .send((
                                    message.encode(mavio::prelude::MavLinkVersion::V2).unwrap(),
                                    m::ProtocolVersion::ID,
                                    m::ProtocolVersion::crc_extra(),
                                ))
                                .await;

                            command_accept(command.command).await;

                            continue;
                        }
                        _ => {
                            println!("server: [error] Unsupported mavlink message");
                        }
                    },
                    _ => {
                        println!("server: [error] Unsupported mavlink command");
                    }
                }

                command_unsupported(command.command).await;
            }

            id @ 1.. => {
                println!("server: [error] unsupported message ID {id}")
            }

            _ => {}
        }
    }
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

pub async fn writer_half(
    writer: WriteHalf<TcpStream>,
    params: Arc<RwLock<Parameters>>,
    mut rx: Receiver<(Payload, u32, u8)>,
) {
    let writer = mavio::io::TokioWriter::new(writer);
    let mut sender = mavio::io::AsyncSender::new::<V2>(writer);

    let mut sequence = 0;

    loop {
        let (payload, id, crc) = rx.recv().await.unwrap();

        let params = params.read().await;

        let frame = mavio::Frame::builder()
            .component_id(params.mav.id.component_id)
            .system_id(params.mav.id.system_id)
            .sequence(sequence)
            .version(V2)
            .message_id(id)
            .payload(payload.bytes())
            .crc_extra(crc)
            .build();

        if sender.send(&frame).await.is_ok() {
            sequence = sequence.wrapping_add(1);
        }
    }
}
