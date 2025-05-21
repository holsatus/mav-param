///  Primitives which can be represented as a "float" in a MavLink parameter
pub trait Primitive {
    fn from_bytewise(val: f32) -> Self;
    fn into_bytewise(self) -> f32;
    fn into_value(self) -> Value;
}

/// Converts the float-encoded value into the correct primitive type.
#[cfg(target_endian = "little")]
pub fn from_bytewise<F: Primitive>(val: f32) -> F {
    F::from_bytewise(val)
}

/// Converts the primite value into the float-encoded equivalent.
#[cfg(target_endian = "little")]
pub fn into_bytewise<F: Primitive>(val: F) -> f32 {
    F::into_bytewise(val)
}

macro_rules! impl_primitive {
    (
        $( $variant:ident($type:ident) ),+ $(,)?
    ) => {

        #[derive(Debug, Clone, Copy)]
        /// Represents the value of a parameter.
        pub enum Value {
            $( $variant($type), )+
        }

        /// Represents a mutable reference to some parameters value.
        #[derive(Debug)]
        pub enum ValueMut<'a> {
            $( $variant(&'a mut $type), )+
        }

        #[repr(C)]
        #[cfg(target_endian = "little")]
        union Bytewise {
            $( $type: $type, )+
        }

        $( impl Primitive for $type {
            #[inline(always)]
            #[cfg(target_endian = "little")]
            fn from_bytewise(val: f32) -> $type {
                unsafe { Bytewise { f32: val }.$type }
            }

            #[inline(always)]
            #[cfg(target_endian = "little")]
            fn into_bytewise(self) -> f32 {
                unsafe { Bytewise { $type: self }.f32 }
            }

            fn into_value(self) -> Value {
                Value::$variant(self)
            }
        } )+


        impl Value {
            #[cfg(target_endian = "little")]
            pub fn into_bytewise(&self) -> f32 {
                match self {
                    $( Value::$variant(v) => v.into_bytewise(), )+
                }
            }
        }

        impl ValueMut<'_> {
            /// Obtain an owned version of this value.
            pub fn owned(&self) -> Value {
                match &self {
                    $( ValueMut::$variant(x) => x.into_value(), )+
                }
            }

            #[cfg(target_endian = "little")]
            pub fn into_bytewise(&self) -> f32 {
                self.owned().into_bytewise()
            }
        }
    };
}

impl_primitive! {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
}
