#[cfg(feature = "simd_json")]
mod deserializer_simd_json;
#[cfg(feature = "simd_json")]
#[allow(unused_imports)]
pub use deserializer_simd_json::*;

#[cfg(not(feature = "simd_json"))]
mod deserializer_serde_json;
#[cfg(not(feature = "simd_json"))]
#[allow(unused_imports)]
pub use deserializer_serde_json::*;