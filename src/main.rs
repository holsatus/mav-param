use std::str;

use param_rs::{Parameter, Value};

struct OptionalStruct {
    regular: f32,
    value1: Option<i32>,
    value2: Option<(f32, f32)>,
}

#[derive(param_rs::Tree)]
struct OptionalParams {
    opt: Optional,
    regular: f32,
    value1: i32,
    value2: (f32, f32),
}

#[derive(param_rs::Node)]
struct Optional(u8);

bitflags::bitflags! {
    impl Optional: u8 {
        const VALUE_1 = 1 << 0;
        const VALUE_2 = 1 << 1;
    }
}
impl From<OptionalParams> for OptionalStruct {
    fn from(data: OptionalParams) -> Self {
        OptionalStruct {
            regular: data.regular,
            value1: data.opt.contains(Optional::VALUE_1).then(||data.value1),
            value2: data.opt.contains(Optional::VALUE_2).then(||data.value2),
        }
    }
}

impl From<OptionalStruct> for OptionalParams {
    fn from(data: OptionalStruct) -> Self {
        let mut opt = Optional::empty();
        opt.set(Optional::VALUE_1, data.value1.is_some());
        opt.set(Optional::VALUE_2, data.value2.is_some());
        OptionalParams {
            opt,
            regular: data.regular,
            value1: data.value1.unwrap_or_default(),
            value2: data.value2.unwrap_or_default(),
        }
    }
}

#[derive(param_rs::Tree)]
struct RateParameters {
    #[tree(rename = "rol")]
    roll: AxisParameters,
    #[tree(rename = "pit")]
    pitch: AxisParameters,
    #[tree(rename = "yaw")]
    yaw: AxisParameters,
    /// Slewrate limiter for reference signal
    ref_slew: f32,
    /// Low-pass filter for reference signal
    ref_lp: f32,
}

impl Default for RateParameters {
    fn default() -> Self {
        RateParameters {
            roll: AxisParameters::default(),
            pitch: AxisParameters::default(),
            yaw: AxisParameters::default(),
            ref_slew: 300.0,
            ref_lp: 0.004,
        }
    }
}

#[derive(param_rs::Tree)]
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

#[derive(param_rs::Node)]
struct Gain(f32);

impl Default for AxisParameters {
    fn default() -> Self {
        AxisParameters {
            p: Gain(20.),
            i: Gain(1.),
            d: Gain(1.),
            cfg: AxisFlags::default(),
            dtau: 0.001,
            pred: 0.04,
            comp: 0.01,
        }
    }
}

#[derive(param_rs::Node)]
struct AxisFlags(u8);

bitflags::bitflags! {
    impl AxisFlags: u8 {
        const D_TERM_LP = 1 << 0;
        const REF_SLEW = 1 << 1;
        const COMP_PRED = 1 << 2;
    }
}

impl Default for AxisFlags {
    fn default() -> Self {
        use AxisFlags as A;
        A::D_TERM_LP | A::REF_SLEW | A::COMP_PRED
    }
}

fn main() {
    let params = RateParameters::default();

    for result in param_rs::param_iter_named(&params, "rate") {
        match result {
            Ok(Parameter { ident, value }) => {
                println!("{:?} => {:?}", ident.as_str(), value,);
            }
            Err(error) => {
                println!("Iteration error: {:?}", error);
            }
        }
    }

    match param_rs::get_value(&params, ".pit.p") {
        Some(Value::F32(sys_id)) => println!("Parameter: {sys_id}"),
        _ => println!("warn: No such parameter"),
    }

    let mut string = [b'\0'; 16];

    string[..5].copy_from_slice(b"hello");

    let ident = str::from_utf8(&string).unwrap();

    println!("Ident: {}", ident);

    let val = param_rs::value::from_bytewise::<u8>(12345.2);
}
