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
	use serde_json::{from_str as serde_json_from_str, from_slice as serde_json_from_slice};
	pub use serde_json::{
		self,
		from_reader, 
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
	pub fn from_str<'a, T>(s: &'a mut str) -> serde_json::Result<T>
	where
    	T: Deserialize<'a> {
			serde_json_from_str(s)
	}
	pub fn from_slice<'a, T>(s: &'a mut [u8]) -> serde_json::Result<T>
	where
    	T: Deserialize<'a> {
			serde_json_from_slice(s)
	}
}

#[cfg(feature = "simd")]
mod simd {
	use serde::Deserialize;
	use simd_json::serde::from_str as unsafe_from_str; //THIS is mutable and is unsafe!
	pub use simd_json::{
		self,
		serde::{
			from_reader, 
			from_slice,                  //THIS is mutable!                    
			from_owned_value as from_value, 
			to_string_pretty,
			to_string, 
			to_owned_value as to_value, 
			to_writer,
		}, 
		Deserializer as JsonDeserializer,
		json,
		owned::Value, 
		Error as JsonError, 
		tape::Value as RawValue, //THIS is gonna be the fun one!
	};

	pub fn from_str<'a, T>(s: &'a mut str) -> simd_json::Result<T>
		where
    		T: Deserialize<'a> {
				unsafe { unsafe_from_str(s) }
	} 
}
