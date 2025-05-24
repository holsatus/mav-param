///  Primitives which can be represented as a "float" in a Mavlink parameter
///
/// This trait enables conversion between MAVLink parameter values (which are transmitted
/// as IEEE 754 floats) and their actual primitive types through bytewise reinterpretation
/// rather than numeric conversion.
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

        #[derive(Debug, Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        // An unsafe type that allows for byte-wise conversion between the inner types.
        //
        // This matches the Mavlink C API, and is why we do the same here. It is however
        // only safe to use on little-endian systems. PRs for big-endian support are welcome.
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
            /// Convert this [`Value`] into a [`ValueMut`] to allow per-type mutation.
            pub fn as_mut(&mut self) -> ValueMut<'_> {
                match self {
                    $( Value::$variant(v) => ValueMut::$variant(v), )+
                }
            }

            /// Get the Mavlink-compatible bytewise representation of this [`Value`]
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

            /// Get the Mavlink-compatible bytewise representation of this [`ValueMut`]
            #[cfg(target_endian = "little")]
            pub fn into_bytewise(&self) -> f32 {
                self.owned().into_bytewise()
            }

            /// Attempt to assign another [`Value`] to this [`ValueMut`] without changing its type.
            ///
            /// If the types matched this functions returns `true`, otherwise `false`.
            pub fn try_assign(&mut self, other: Value) -> bool {
                match (self, other) {
                    $( (ValueMut::$variant(vm), Value::$variant(v)) => **vm = v, )+
                    _ => return false, // Type mismatch
                }
                true
            }
        }
    };
}

// These primitive types are supported by MavLink.
//
// Technically, i64, u64 and f64 are also "supported", though they
// need to fit in a 32-bit float when sent, so supporting them does
// not make much sense. These can be transmitted losslessly while
// being supported. Sadly, bools are not supported.

impl_primitive! {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
}
