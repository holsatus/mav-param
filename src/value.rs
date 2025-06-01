/// Primitives which can be represented as a float in a Mavlink parameter
///
/// This trait enables conversion between MAVLink parameter values (which are transmitted
/// as IEEE 754 floats) and their actual primitive types through bytewise reinterpretation
/// rather than numeric conversion.
pub trait Floaty: Leaf {
    fn from_bytewise(val: f32) -> Self;
    fn into_bytewise(self) -> f32;
}

/// Allows for getting and setting the inner [`Value`] of a [`Leaf`] node.
pub trait Leaf {
    fn get(&self) -> Value;
    fn set(&mut self, val: Value) -> bool;
}

/// Converts the float-encoded value into the correct primitive type.
#[cfg(target_endian = "little")]
pub fn from_bytewise<F: Floaty>(val: f32) -> F {
    F::from_bytewise(val)
}

/// Converts the primite value into the float-encoded equivalent.
#[cfg(target_endian = "little")]
pub fn into_bytewise<F: Floaty>(val: F) -> f32 {
    F::into_bytewise(val)
}

macro_rules! impl_primitive {
    (
        $( $variant:ident($type:ident) ),+ $(,)?
    ) => {

        /// Represents the value of a parameter.
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum Value {
            $( $variant($type), )+
        }

        /// Represents a mutable reference to some parameters value.
        #[derive(Debug, PartialEq)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum ValueMut<'a> {
            $( $variant(&'a mut $type), )+
        }

        // An unsafe type that allows for byte-wise conversion between the inner types.
        //
        // This matches the Mavlink C API, and is why we do the same here. It is however
        // only safe to use on little-endian systems. PRs for big-endian support are welcome.
        #[repr(C)]
        #[cfg(target_endian = "little")]
        union Bytewise {
            $( $type: $type, )+
        }

        $( impl Floaty for $type {
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
        } )+

        $( impl Leaf for $type {
            fn get(&self) -> Value {
                Value::$variant(*self)
            }

            fn set(&mut self, val: Value) -> bool {
                if let Value::$variant(val) = val {
                    *self = val;
                    true
                } else {
                    false
                }
            }
        } )+

        $( impl super::Node<'_> for $type {
            fn node_ref(&self) -> super::NodeRef<'_> {
                super::NodeRef::Leaf(self)
            }

            fn node_mut(&mut self) -> super::NodeMut<'_> {
                super::NodeMut::Leaf(self)
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
