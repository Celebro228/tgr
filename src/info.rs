#[cfg(target_os = "windows")]
pub static DEVICE: u8 = 0;
#[cfg(target_os = "linux")]
pub static DEVICE: u8 = 0;
#[cfg(target_os = "macos")]
pub static DEVICE: u8 = 0;

#[cfg(target_os = "android")]
pub static DEVICE: u8 = 1;
#[cfg(target_os = "ios")]
pub static DEVICE: u8 = 1;

#[cfg(target_arch = "wasm32")]
pub static DEVICE: u8 = 2;
