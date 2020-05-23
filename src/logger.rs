use web_sys::console;
use wasm_bindgen::JsValue;
use crate::logger::Level::{DEBUG, INFO, WARN, ERROR};

pub enum Level {
    DEBUG,
    INFO,
    WARN,
    ERROR
}

const LOG_LEVEL: Level = DEBUG;

pub fn log_debug(x: &str) {
    match LOG_LEVEL {
        DEBUG => console::log_1(&JsValue::from_str((String::from("[DEBUG] ") + x).as_str())),
        _ => {}
    }
}

pub fn log_info(x: &str) {
    match LOG_LEVEL {
        DEBUG | INFO => console::log_1(&JsValue::from_str((String::from("[INFO] ") + x).as_str())),
        _ => {}
    }
}

pub fn log_warn(x: &str) {
    match LOG_LEVEL {
        DEBUG | INFO | WARN => console::log_1(&JsValue::from_str((String::from("[WARN] ") + x).as_str())),
        _ => {}
    }
}
pub fn log_error(x: &str) {
    console::log_1(&JsValue::from_str((String::from("[ERROR] ") + x).as_str()));
}
