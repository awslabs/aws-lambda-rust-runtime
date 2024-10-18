// Using serde_json as the JSON handler
#[cfg(feature = "serde")]
pub use serde::*;
// Using simd_json as the JSON handler
#[cfg(feature = "simd")]
pub use simd::*;

// Implementations

#[cfg(feature = "serde")]
mod serde {
	pub use serde_json::{
		from_reader, 
		from_slice, 
		from_str,
		from_value, 
		json, 
		to_string_pretty,
		to_string, 
		to_value, 
		to_writer, 
		Deserializer as JsonDeserializer,
		Value, 
		error::Error as JsonError, 
		value::RawValue as RawValue,
	};
}

#[cfg(feature = "simd")]
mod simd {
	pub use simd_json::{
		serde::{
			from_reader, 
			from_slice,                  //THIS is mutable!
			from_str,                    //THIS is mutable and is unsafe!
			from_owned_value as from_value, 
			json, 
			to_string_pretty,
			to_string, 
			to_owned_value as to_value, 
			to_writer,
		}, 
		Deserializer as JsonDeserializer,
		owned::Value, 
		Error as JsonError, 
		tape::Value as RawValue, //THIS is gonna be the fun one!
	};
}
