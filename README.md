# Small Mavlink parameter system

Mavlink has a [parameter protocol](https://mavlink.io/en/services/parameter.html), that allows for enumerating, querying and modifying parameters based on an up to 16-character key, which maps to some primitive type (signed/unsigned integers and float) value. This allows for ground stations to request and list all parameters of systems that support the protocol, as well as save the config of a system as a flat list of keys and values. Even though Rusts type system allows for much more advanced types, supporting such a simple system would be beneficial for Mavlink capable MAV systems.



---

This library supports turning something like this:

```rust
// A nested struct of parameters
struct Parameters {
    mavlink: Mavlink,
    // add more as needed
}

struct Mavlink {
    timeout_ms: u16,
    id: MavlinkId,
    flags: Flags
}

struct MavlinkId {
    system_id: u8,
    component_id: u8,
}

bitflags::bitflags! {
    struct Flags: u8 {
        const EMIT_HEARTBEAT = 1 << 0;
        // .. and so on
    }
}
```

Into a flat list of key-value pairs, with optional field renaming:

```
"mav.timeout_ms" = U16(5000)
"mav.id.sys" = U8(1)
"mav.id.com" = U8(1)
"mav.flags" = U8(1)
```

## Usage

Structs with named fields are marked with the `mav_param::Tree` derive macro. This ensures we have a way to enumerate all fields by their string identifier, and retrieve the "node" (value or another struct) for a given identifier. Unit structs can be made transparent with the `mav_param::Node` derive macro.

For example an implementation of the Mavlink server may have the parameter tree:

```rust
// A nested struct of parameters
#[derive(mav_param::Tree)]
struct MavlinkParams {
    timeout_ms: u16,
    id: MavlinkId,
    flags: Flags
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

// A "transparent" unit struct, makes
// working with the bitflags crate easy!
#[derive(mav_param::Node)]
struct Flags(u8);

bitflags::bitflags! {
    impl Flags: u8 {
        const EMIT_HEARTBEAT = 1 << 0;
        // .. and so on
    }
}
```
We can now iteratively traverse the tree, to list out all parameters, ensuring "mav" is the root name of this parameter tree. This code will yield the list of key-value pairs shown previously.

```rust
// Print out all parameter identifiers, adding the "mav" prefix
for param in mav_param::param_iter_named(&mav, "mav").filter_map(|entry| entry.ok()) {
    println!("{:?} = {:?}", param.ident.as_str(), param.value);
}
```

We never had to explicitly define the full paths, for example `"mav.id.sys"`, since that is determined from the location of the parameter within its parent structs. The string is only generated at runtime by traversing the tree. This can save us a lot of memory by not having to store a 16-byte string for every parameter.

Alternatively we can index into the struct using a string, to modify a parameter:

```rust
// Set the mav system id by string-lookup. The value is only written if the
// stored parameter has the same type as the provided `Value`.
match mav_param::set_value(&mut mav, ".id.sys", mav_param::Value::U8(100)) {
    Some(()) => println!("updated mav.id.sys"),
    None => println!("warn: No such parameter"),
}
```

We may notice that the syntax for the stringified key and the Rust code to access the same value is very similar, which is intentional. This even extends to tuples (up to length 10) and arrays (up to length 20, using the `arr.0` indexing syntax):

```rust
#[derive(mav_param::Tree, Default)]
struct Parameters {
    cfg: Config,
}

#[derive(mav_param::Tree, Default)]
struct Config {
    var: (u8, f32, i16),
    arr: [f32; 8],
}

let mut param = Parameters::default()

// Regular Rust syntax
param.cfg.var.0 = 255;

// Stringy syntax (only writes if the stored type is `f32`)
mav_param::set_value(&mut param, ".cfg.var.1", mav_param::Value::F32(2.718));

// Stringy syntax
mav_param::set_value(&mut param, ".cfg.arr.7", mav_param::Value::F32(3.1415));
```

And iterating all key-value pair would give:

```
"param.cfg.var.0" = U8(255)
"param.cfg.var.1" = F32(2.718)
"param.cfg.var.2" = I16(0)
```
# Working with a Mavlink library

For whatever reason, the value in the [PARAM_VALUE](https://mavlink.io/en/messages/common.html#PARAM_VALUE) field is encoded as a float. So when working with a Mavlink library, we need to do a byte-wise conversion of the primitive type into a float. For this we provide some helper functions to handle the byte-wise conversion itself, though some extra work is required to make it interact with the library.

## Mavlink - Reading parameter

Retrieving a parameter for Mavlink, through e.g. [PARAM_REQUEST_READ](PARAM_REQUEST_READ) involves taking the `param_id` from the incoming request, and using that (converted to a `&str`) to get the value. The value is converted into its bytewise representation, and the type of the parameter is included in the outgoing message.

```rust
use your_mavlink_library as mav;
use mav_param::{Ident, Value, get_value};

// We have received a parameter read request
let in_message = mav::ParamRequestRead::decode(payload)?;

// Convert into a valid identifier and fetch the parameter
let ident = Ident::try_from(&in_message.param_id)?;
let value = get_value(&params, ident.as_str())?;

// Do bytewise conversion into float value
let param_value = value.into_bytewise();

// Encode which type the value is
let param_type = match value {
    Value::U8(_) => mav::ParamType::Uint8,
    Value::I8(_) => mav::ParamType::Int8,
    Value::U16(_) => mav::ParamType::Uint16,
    Value::I16(_) => mav::ParamType::Int16,
    // .. and so on
}

// This is what is sent to the GCS
let out_message = mav::ParamValue {
    param_value,
    param_type,
    // .. other fields
};
```

## Mavlink - Setting parameter

A Mavlink request to set a parameter, e.g. [PARAM_SET](https://mavlink.io/en/messages/common.html#PARAM_SET), is very similar. Here we use the `from_bytewise` function to convert the float payload into the type announced by the GCS, then call `mav_param::set_value`. That function only writes the value if the stored parameter has the same type, so a type mismatch is reported by the `None` return.

```rust
use your_mavlink_library as mav;
use mav_param::{
    Ident, Value, set_value,
    value::from_bytewise
};

// We have received a parameter set request
let in_message = mav::ParamSet::decode(payload)?;

// Convert into a valid identifier and fetch the parameter
let ident = Ident::try_from(&in_message.param_id)?;

// Convert the float payload into the type announced by the GCS
use mav::ParamType as T;
let value = match in_message.param_type {
    T::Uint8  => Value::U8(from_bytewise(in_message.param_value)),
    T::Int8   => Value::I8(from_bytewise(in_message.param_value)),
    T::Uint16 => Value::U16(from_bytewise(in_message.param_value)),
    T::Int16  => Value::I16(from_bytewise(in_message.param_value)),
    // .. and so on
    _ => return Err("parameter type is not supported by mav_param"),
};

// Only written if the parameter exists and has the matching type
if set_value(&mut params, ident.as_str(), value).is_none() {
    return Err("attempted to set parameter using the wrong type");
}

// We should now respond with our new value to confirm..
```

# Implementation

This library relies on deriving the `mav_param::Tree` on structs, where each field/entry implements the `mav_param::Node` trait, which allows for converting the field into either a primitive type/value, or another `Tree`. Both `Tree`s and the supported primitives implement `Node`. This is what allows for using composition to combine structs, tuples, arrays and primitives into a data type that can be iterated to generate all stringy identifiers.
