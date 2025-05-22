# Small Mavlink parameter system

Mavlink has a [parameter protocol](https://mavlink.io/en/services/parameter.html), that allows for enumerating, querying and modifying parameters based on an up to 16-character key, which maps to some primitive type (signed/unsigned integers and float) value. This allows for ground stations to request and list all parameters of systems that support the protocol, as well as save the config of a system as a flat list of keys and values. Even though Rusts type system allows for much more advanced types, supporting such a simple system would be beneficial for Mavlink capable MAV systems.

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
    #[tree(rename = "sys")]
    system_id: u8,
    #[tree(rename = "com")]
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
// Mutable get the mav system id by string-lookup
match mav_param::get_value_mut(&mut mav, ".id.sys") {
    Some(mav_param::ValueMut::U8(sys_id)) => *sys_id = 100
    _ => println!("warn: No such parameter"),
}
```

We may notice that the syntax for the stringified key and the Rust code to access the same value is very similar, which is intentional. This even extends to tuples and arrays up to length 10 (though arrays also use the `arr.0` indexing syntax):

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

// Stringy syntax
match mav_param::get_leaf_mut(&mut param, ".cfg.var.1") {
    Some(mav_param::ValueMut::F32(var_0)) => *var_0 = 2.718,
    _ => println!("warn: No such parameter"),
}

// Stringy syntax
match mav_param::get_leaf_mut(&mut param, ".cfg.arr.7") {
    Some(mav_param::ValueMut::F32(arr_7)) => *arr_7 = 3.1415,
    _ => println!("warn: No such parameter"),
}
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

A Mavlink request to set a parameter, e.g. [PARAM_SET](https://mavlink.io/en/messages/common.html#PARAM_SET), is very similar. Here we instead mutably look up the parameter with `mav_param::get_value_mut` and use the `from_bytewise` function to convert from the float into the desired type. Here it is important to do a manual type-check, to ensure the new value's type matches the original.

```rust
use your_mavlink_library as mav;
use mav_param::{
    Ident, ValueMut, get_value_mut,
    value::from_bytewise
};

// We have received a parameter set request
let in_message = mav::ParamSet::decode(payload)?;

// Convert into a valid identifier and fetch the parameter
let ident = Ident::try_from(&in_message.param_id)?;
let value_mut = get_value_mut(&mut params, ident.as_str())?;

// Get the float-value we need to convert
let param_value = in_message.param_value;

// Modify the value only if the parameter types match
use mav::ParamType as T;
match (value_mut, in_message.param_type) {
    (ValueMut::U8(v),  T::Uint8)  => *v = from_bytewise::<u8>(param_value),
    (ValueMut::I8(v),  T::Int8)   => *v = from_bytewise::<i8>(param_value),
    (ValueMut::U16(v), T::Uint16) => *v = from_bytewise::<u16>(param_value),
    (ValueMut::I16(v), T::Int16)  => *v = from_bytewise::<i16>(param_value),
    // .. and so on, with an error in case of mismatching types
    _ => return Err(Error::ParamTypeMismatch),
};

// We should now respond with our new value to confirm..
```

# Implementation

This library relies on a deriving the `mav_param::Tree` on strucs, where each field/entry implements the `mav_param::Node` trait, which allows for converting the field into either a primitive type/value, or another `Tree`. Anything that that is a `Tree` or supported primitives automatically `Node`. This is what allows for using composition to combine structs, tuples, arrays and primitives into a data type that can be iterated to generate all stringy identifiers.

## Limitations

The main limitation is that the types we can represent are fairly basic, due to how the non-extended parameter protocol works. So all paths in the tree must end up at one of the following primitive types. Technically the protocol also supports f64, u64 and i64, but since the payload can only be 32 bits, it makes more sense to do without.

```rust
pub enum Value {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
}
```

Thus, working with tagged unions/enums with values, including `Option<T>` is not easily doable. One workaround for optional values is to have one parameter act as a flag (for example using `bitflags`) to designate whether some other parameter should be considered a `Some` variant, or to be ignored. Consider the following example.

```rust
// Normal Rust struct with optionals
struct OptionalStruct {
    value1: Option<i32>,
    value2: Option<(f32, f32)>,
}

// .. may be represented as follows:
#[derive(mav_param::Tree)]
struct OptionalParams {
    opt: Optional,
    value1: i32,
    value2: (f32, f32),
}

#[derive(mav_param::Node)]
struct Optional(u8);

bitflags::bitflags! {
    impl Optional: u8 {
        // Each optional get a unique bit
        const VALUE_1 = 1 << 0;
        const VALUE_2 = 1 << 1;
    }
}

// .. with implementations to convert between the
// Rust-idiomatic representation, to the Mavlink
// compatible parameter representetion:

impl From<OptionalParams> for OptionalStruct {
    fn from(params: OptionalParams) -> Self {
        OptionalStruct {
            value1: params.opt.contains(Optional::VALUE_1).then(||params.value1),
            value2: params.opt.contains(Optional::VALUE_2).then(||params.value2),
        }
    }
}

impl From<OptionalStruct> for OptionalParams {
    fn from(params: OptionalStruct) -> Self {
        let mut opt = Optional::empty();
        opt.set(Optional::VALUE_1, params.value1.is_some());
        opt.set(Optional::VALUE_2, params.value2.is_some());
        OptionalParams {
            opt,
            value1: params.value1.unwrap_or_default(),
            value2: params.value2.unwrap_or_default(),
        }
    }
}
```

This is by no means ideal, but creates a workable conversion between the idiomatic struct, and something that can be sent as Mavlink parameters. If anyone has suggestions for how this can be simplified, either within the derive macro, or by using a declarative helper macro, please open an issue or PR!
