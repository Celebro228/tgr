use std::collections::HashMap;
use std::any::Any;

static mut GLOBAL_DATA: Option<HashMap<&'static str, Box<dyn Any + Send + Sync>>> = None;

pub fn set_data<T: 'static + Send + Sync>(key: &'static str, value: T) {
    unsafe {
        if GLOBAL_DATA.is_none() {
            GLOBAL_DATA = Some(HashMap::new());
        }
        GLOBAL_DATA.as_mut().unwrap().insert(key, Box::new(value));
    }
}

pub fn get_data<T: 'static>(key: &'static str) -> Option<&'static T> {
    unsafe {
        GLOBAL_DATA.as_ref()?.get(key)?.downcast_ref::<T>()
    }
}