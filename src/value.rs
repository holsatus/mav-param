#[derive(Debug, Clone, Copy)]
pub enum Value {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
}

macro_rules! impl_into_value {
    ($($var:ident($type:ty)),+ $(,)?) => {
        $(
            impl From<$type> for Value {
                fn from(from: $type) -> Value {
                    Value::$var(from)
                }
            }
        )+
    };
}

impl_into_value! {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
}

///  Primitives which can be represented as a "float" in a MavLink parameter
pub trait Floaty {
    fn bytewise_from_float(val: f32) -> Value;
}

impl Floaty for u8 {
    fn bytewise_from_float(val: f32) -> Value {
        let [_, _, _, b0] = val.to_le_bytes();
        u8::from_le_bytes([b0]).into()
    }
}

impl Floaty for i8 {
    fn bytewise_from_float(val: f32) -> Value {
        let [_, _, _, b0] = val.to_le_bytes();
        i8::from_le_bytes([b0]).into()
    }
}

impl Floaty for u16 {
    fn bytewise_from_float(val: f32) -> Value {
        let [_, _, b1, b0] = val.to_le_bytes();
        u16::from_le_bytes([b1, b0]).into()
    }
}

impl Floaty for i16 {
    fn bytewise_from_float(val: f32) -> Value {
        let [_, _, b1, b0] = val.to_le_bytes();
        i16::from_le_bytes([b1, b0]).into()
    }
}

impl Floaty for u32 {
    fn bytewise_from_float(val: f32) -> Value {
        let bytes = val.to_le_bytes();
        u32::from_le_bytes(bytes).into()
    }
}

impl Floaty for i32 {
    fn bytewise_from_float(val: f32) -> Value {
        let bytes = val.to_le_bytes();
        i32::from_le_bytes(bytes).into()
    }
}

impl Floaty for f32 {
    fn bytewise_from_float(val: f32) -> Value {
        val.into()
    }
}

impl Value {
    pub fn bytewise_from_float<T: Floaty>(val: f32) -> Value {
        T::bytewise_from_float(val)
    }

    pub fn bytewise_into_float(&self) -> f32 {
        match self {
            Value::U8(v) => {
                let [b0] = v.to_le_bytes();
                f32::from_le_bytes([0, 0, 0, b0])
            }
            Value::I8(v) => {
                let [b0] = v.to_le_bytes();
                f32::from_le_bytes([0, 0, 0, b0])
            }
            Value::U16(v) => {
                let [b1, b0] = v.to_le_bytes();
                f32::from_le_bytes([0, 0, b1, b0])
            }
            Value::I16(v) => {
                let [b1, b0] = v.to_le_bytes();
                f32::from_le_bytes([0, 0, b1, b0])
            }
            Value::U32(v) => f32::from_le_bytes(v.to_le_bytes()),
            Value::I32(v) => f32::from_le_bytes(v.to_le_bytes()),
            Value::F32(v) => f32::from(*v),
        }
    }
}

/// Represents a mutable reference to some parameters value.
#[derive(Debug)]
pub enum ValueMut<'a> {
    U8(&'a mut u8),
    I8(&'a mut i8),
    U16(&'a mut u16),
    I16(&'a mut i16),
    U32(&'a mut u32),
    I32(&'a mut i32),
    F32(&'a mut f32),
}

impl ValueMut<'_> {
    /// Obtain an owned version of this value.
    pub fn owned(&self) -> Value {
        match &self {
            ValueMut::U8(x) => (**x).into(),
            ValueMut::I8(x) => (**x).into(),
            ValueMut::U16(x) => (**x).into(),
            ValueMut::I16(x) => (**x).into(),
            ValueMut::U32(x) => (**x).into(),
            ValueMut::I32(x) => (**x).into(),
            ValueMut::F32(x) => (**x).into(),
        }
    }

    /// Turn this value into a byte-wise float representation,
    /// for use in the MavLink parameter protocol.
    pub fn bytewise_into_float(&self) -> f32 {
        self.owned().bytewise_into_float()
    }
}
