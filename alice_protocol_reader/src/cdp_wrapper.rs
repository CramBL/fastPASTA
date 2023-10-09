//! Contains the wrappers for the loaded CDP batches.
//!
//! The CDP batches are loaded from the CDP files and wrapped in a struct that
//! provides a common interface for accessing the data.
//!
//! If you know the size of the batches at compile time, use the `CdpArray`.
//! If you don't or you need more flexibility at runtime, use the `CdpVec`.
pub mod cdp_array;
pub mod cdp_vec;
