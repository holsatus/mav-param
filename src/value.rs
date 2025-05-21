
///  Primitives which can be represented as a "float" in a MavLink parameter
pub trait Primitive {
    fn from_bytewise(val: f32) -> Self;
    fn into_bytewise(self) -> f32;
    fn into_value(self) -> Value;
}

/// Converts the float-encoded value into the correct primitive type.
/// 
/// It is up to the user to ensure the conversion is done correctly!
pub fn from_bytewise<F: Primitive>(val: f32) -> F {
    F::from_bytewise(val)
}

impl Primitive for u8 {
    fn from_bytewise(val: f32) -> u8 {
        let [_, _, _, b0] = val.to_le_bytes();
        u8::from_le_bytes([b0])
    }
    
    fn into_bytewise(self) -> f32 {
        let [b0] = self.to_le_bytes();
        f32::from_le_bytes([0, 0, 0, b0])
    }
    
    fn into_value(self) -> Value {
        Value::U8(self)
    }
}

impl Primitive for i8 {
    fn from_bytewise(val: f32) -> i8 {
        let [_, _, _, b0] = val.to_le_bytes();
        i8::from_le_bytes([b0])
    }
    
    fn into_bytewise(self) -> f32 {
        let [b0] = self.to_le_bytes();
        f32::from_le_bytes([0, 0, 0, b0])
    }

    fn into_value(self) -> Value {
        Value::I8(self)
    }
}

impl Primitive for u16 {
    fn from_bytewise(val: f32) -> u16 {
        let [_, _, b1, b0] = val.to_le_bytes();
        u16::from_le_bytes([b1, b0])
    }
    
    fn into_bytewise(self) -> f32 {
        let [b1, b0] = self.to_le_bytes();
        f32::from_le_bytes([0, 0, b1, b0])
    }

    fn into_value(self) -> Value {
        Value::U16(self)
    }
}

impl Primitive for i16 {
    fn from_bytewise(val: f32) -> i16 {
        let [_, _, b1, b0] = val.to_le_bytes();
        i16::from_le_bytes([b1, b0])
    }
    
    fn into_bytewise(self) -> f32 {
        let [b1, b0] = self.to_le_bytes();
        f32::from_le_bytes([0, 0, b1, b0])
    }

    fn into_value(self) -> Value {
        Value::I16(self)
    }
}

impl Primitive for u32 {
    fn from_bytewise(val: f32) -> u32 {
        u32::from_le_bytes(val.to_le_bytes())
    }
    
    fn into_bytewise(self) -> f32 {
        f32::from_le_bytes(self.to_le_bytes())
    }

    fn into_value(self) -> Value {
        Value::U32(self)
    }
}

impl Primitive for i32 {
    fn from_bytewise(val: f32) -> i32 {
        i32::from_le_bytes(val.to_le_bytes())
    }
    
    fn into_bytewise(self) -> f32 {
        f32::from_le_bytes(self.to_le_bytes())
    }

    fn into_value(self) -> Value {
        Value::I32(self)
    }
}

impl Primitive for f32 {
    fn from_bytewise(val: f32) -> f32 {
        val
    }
    
    fn into_bytewise(self) -> f32 {
        self
    }

    fn into_value(self) -> Value {
        Value::F32(self)
    }
}

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

impl Value {
    pub fn into_bytewise(&self) -> f32 {
        match self {
            Value::U8(v) => v.into_bytewise(),
            Value::I8(v) => v.into_bytewise(),
            Value::U16(v) => v.into_bytewise(),
            Value::I16(v) => v.into_bytewise(),
            Value::U32(v) => v.into_bytewise(),
            Value::I32(v) => v.into_bytewise(),
            Value::F32(v) => v.into_bytewise(),
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
            ValueMut::U8(x) => x.into_value(),
            ValueMut::I8(x) => x.into_value(),
            ValueMut::U16(x) => x.into_value(),
            ValueMut::I16(x) => x.into_value(),
            ValueMut::U32(x) => x.into_value(),
            ValueMut::I32(x) => x.into_value(),
            ValueMut::F32(x) => x.into_value(),
        }
    }

    pub fn into_bytewise(&self) -> f32 {
        self.owned().into_bytewise()
    }
}
