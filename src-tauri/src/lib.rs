pub mod core;
#[cfg(feature = "desktop")]
pub mod desktop;
pub mod rpc;

#[cfg(feature = "desktop")]
mod tray;
