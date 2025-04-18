use std::collections::HashMap;
use std::any::Any;

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
    unsafe {
        GLOBAL_DATA.as_ref()?.get(key)?.downcast_ref::<T>()
    }
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
    unsafe {
        GLOBAL_STAT.as_ref().unwrap().get(&key).unwrap_or(&0.)
    }
}
#[inline(always)]
pub fn add_stat(key: usize, value: f32) {
    unsafe {
        GLOBAL_STAT.as_mut().unwrap().insert(key, get_stat(key) + value);
    }
}