// Using serde_json as the JSON handler
#[cfg(not(feature = "simd"))]
pub use serde::*;
// Using simd_json as the JSON handler
#[cfg(feature = "simd")]
pub use simd::*;

// Implementations

#[cfg(not(feature = "simd"))]
mod serde {
    use serde::Deserialize;
    pub use serde_json::{
        self, error::Error as JsonError, from_reader, from_slice, from_str, from_value, json, to_string,
        to_string_pretty, to_value, to_writer, value::RawValue, Deserializer as JsonDeserializer, Value,
    };
    pub fn from_str_mut<'a, T>(s: &'a mut str) -> serde_json::Result<T>
    where
        T: Deserialize<'a>,
    {
        from_str(s)
    }
    pub fn from_slice_mut<'a, T>(s: &'a mut [u8]) -> serde_json::Result<T>
    where
        T: Deserialize<'a>,
    {
        from_slice(s)
    }
}

#[cfg(feature = "simd")]
mod simd {
    use serde::Deserialize;
    use simd_json::serde::from_str as unsafe_from_str; //THIS is mutable and is unsafe!
    pub use simd_json::{
        self,
        json,
        owned::Value,
        serde::{
            from_owned_value as from_value,
            from_reader,
            from_slice as from_slice_mut, //THIS is mutable!
            to_owned_value as to_value,
            to_string,
            to_string_pretty,
            to_writer,
        },
        tape::Value as RawValue, //THIS is gonna be the fun one!
        Deserializer as JsonDeserializer,
        Error as JsonError,
    };

    /// BEWARE this ISN'T safe - but is marked so for compatibility at the
    /// moment.
    pub fn from_str_mut<'a, T>(s: &'a mut str) -> simd_json::Result<T>
    where
        T: Deserialize<'a>,
    {
        unsafe { unsafe_from_str(s) }
    }
}
