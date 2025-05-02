#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub static DEVICE: u8 = 0;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub static DEVICE: u8 = 1;

#[cfg(target_arch = "wasm32")]
pub static DEVICE: u8 = 2;
