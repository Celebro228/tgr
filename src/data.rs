use std::{collections::HashMap, any::Any};

#[cfg(feature = "storage")]
use quad_storage::STORAGE;

static mut GLOBAL_DATA: Option<HashMap<&'static str, Box<dyn Any + Send + Sync>>> = None;
static mut GLOBAL_STAT: Option<HashMap<usize, f32>> = None;

pub fn set_data<T: 'static + Send + Sync>(key: &'static str, value: T) {
    unsafe {
        if GLOBAL_DATA.is_none() {
            GLOBAL_DATA = Some(HashMap::new());
        }
        GLOBAL_DATA.as_mut().unwrap().insert(key, Box::new(value));
    }
}
#[inline(always)]
pub fn get_data<T: 'static>(key: &'static str) -> Option<&'static T> {
    unsafe { GLOBAL_DATA.as_ref()?.get(key)?.downcast_ref::<T>() }
}

#[inline(always)]
pub fn set_stat(key: usize, value: f32) {
    unsafe {
        if GLOBAL_STAT.is_none() {
            GLOBAL_STAT = Some(HashMap::new());
        }
        GLOBAL_STAT.as_mut().unwrap().insert(key, value);
    }
}
#[inline(always)]
pub fn get_stat(key: usize) -> &'static f32 {
    unsafe { GLOBAL_STAT.as_ref().unwrap().get(&key).unwrap_or(&0.) }
}
#[inline(always)]
pub fn add_stat(key: usize, value: f32) {
    unsafe {
        GLOBAL_STAT
            .as_mut()
            .unwrap()
            .insert(key, get_stat(key) + value);
    }
}

#[cfg(feature = "storage")]
#[inline(always)]
pub fn save_data(key: &'static str, value: &'static str) {
    STORAGE.lock().unwrap().set(key, value);
}

#[cfg(feature = "storage")]
#[inline(always)]
pub fn load_data(key: &'static str) -> Option<String> {
    STORAGE.lock().unwrap().get(key)
}

#[cfg(feature = "storage")]
#[inline(always)]
pub fn save_stat(key: usize, value: f32) {
    STORAGE
        .lock()
        .unwrap()
        .set(&key.to_string(), &value.to_string());
}

#[cfg(feature = "storage")]
#[inline(always)]
pub fn load_stat(key: usize) -> f32 {
    STORAGE
        .lock()
        .unwrap()
        .get(&key.to_string())
        .unwrap_or(String::from("0"))
        .parse()
        .unwrap()
}

#[cfg(feature = "miniquad")]
pub fn load_file(path: &str) -> Result<Vec<u8>, Error> {
    #[cfg(target_os = "ios")]
    let _ = std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap());

    let contents = Arc::new(Mutex::new(None));
    let path = path.to_owned();

    {
        let contents: Arc<Mutex<Option<Result<Vec<u8>, String>>>> = contents.clone();

        miniquad::fs::load_file(&path, move |bytes| {
            *contents.lock().expect("Error load file") =
                Some(bytes.map_err(|_| "oh my god".to_string()));
        });
    }

    let contents = contents.lock().expect("Error file lock").take();

    if let Some(contents) = contents {
        contents
    } else {
        Err(Error)
    }
}

#[cfg(feature = "wgpu")]
pub use std::fs::read as load_file;