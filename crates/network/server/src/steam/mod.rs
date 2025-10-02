#[cfg(feature = "steamworks")]
mod real;
#[cfg(not(feature = "steamworks"))]
mod stub;

#[cfg(feature = "steamworks")]
pub use real::*;
#[cfg(not(feature = "steamworks"))]
pub use stub::*;
