// Using serde_json as the JSON handler
#[cfg(not(feature = "simd"))]
pub use serde::*;
// Using simd_json as the JSON handler
#[cfg(feature = "simd")]
pub use simd::*;

// Implementations

#[cfg(not(feature = "simd"))]
mod serde {
    use bytes::Bytes;
    use serde::de::DeserializeOwned;
    pub use serde_json::{
        self, error::Error as JsonError, from_reader, from_slice, from_str, from_value, json, to_string,
        to_string_pretty, to_value, to_writer, value::RawValue, Deserializer as JsonDeserializer, Value,
        to_vec,
    };
    pub fn from_bytes<T>(b: Bytes) -> serde_json::Result<T>
    where
    T: DeserializeOwned,
    {
        from_slice(&b)
    }

    pub fn from_string<T>(s: String) -> serde_json::Result<T>
    where
    T: DeserializeOwned,
    {
        from_str(s.as_str())
    }
    
    pub fn from_vec<T>(v: Vec<u8>) -> serde_json::Result<T>
    where
    T: DeserializeOwned,
    {
        from_slice(&v)
    }
}

#[cfg(feature = "simd")]
mod simd {
    use bytes::Bytes;
    use serde::de::DeserializeOwned;
    pub use simd_json::{
        self,
        json,
        owned::Value,
        serde::{
            from_owned_value as from_value,
            from_reader,
            from_str,   //THIS requires a mutable string slice AND is unsafe
            from_slice, //THIS requires a mutable slice!
            to_owned_value as to_value,
            to_string,
            to_string_pretty,
            to_writer,
            to_vec,
        },
        tape::Value as RawValue, //THIS is gonna be the fun one!
        Deserializer as JsonDeserializer,
        Error as JsonError,
    };

    pub fn from_bytes<'a, T>(b: Bytes) -> simd_json::Result<T>
    where
    T: DeserializeOwned,
    {
        match b.try_into_mut() {
            Ok(mut b) => from_slice(&mut b),
            Err(b) => {
                let mut v = b.to_vec();
                from_slice(&mut v)
            }
        }
    }

    pub fn from_string<'a, T>(mut s: String) -> simd_json::Result<T>
    where
    T: DeserializeOwned,
    {
        unsafe{ from_str(s.as_mut_str()) }
    }

    pub fn from_vec<'a, T>(mut v: Vec<u8>) -> simd_json::Result<T>
    where
    T: DeserializeOwned,
    {
        from_slice(&mut v)
    }
}

