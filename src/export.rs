//! C bindings for the json crate

use std::{ffi::{c_char, CStr}, mem, ptr, slice};

type RustJson = crate::Json;

/// Json struct for C
#[repr(C)]
pub enum Json {
    Array {
        elems: *mut Json,
        len: usize,
    },
    Object {
        elems: *mut Pair,
        len: usize,
    },
    String(JsonString),
    Number(f64),
    True, False, Null,
    Error
}

#[repr(C)]
pub struct JsonString {
    buf: *mut u8,
    len: usize,
}

impl JsonString {
    fn new(mut o: String) -> Self {
        let len = o.len();
        o.push('\0'); /* Make sure the string is NULL terminated  */
        let mut data = o.into_boxed_str();
        let buf = data.as_mut_ptr();
        mem::forget(data);
        Self { buf, len }
    }
}

impl Drop for JsonString {
    fn drop(&mut self) {
        let elems = ptr::slice_from_raw_parts_mut(self.buf, self.len);
        let b = unsafe { Box::from_raw(elems) };
        mem::drop( b );
    }
}

#[repr(C)]
pub struct Pair {
    key: JsonString,
    val: *mut Json
}

impl Json {
    fn from_json(json: RustJson) -> Self {
        match json {
            RustJson::Array(arr) => {
                let elems = arr.into_vec().into_iter().map(Json::from_json).collect::<Vec<_>>();
                let len = elems.len();
                let elems = vec_2_ptr(elems);
                Json::Array{ elems, len }
            },
            RustJson::Object(obj) => {
                let elems = obj.into_iter().map(|(k,v)| {
                    let string = JsonString::new(k.into_string());
                    let v = Json::from_json(v);
                    let v = Box::new(v);
                    Pair { key : string, val: Box::into_raw(v) }
                }).collect::<Vec<_>>();
                let len = elems.len();
                let elems = vec_2_ptr(elems);
                Json::Object{ elems, len }
            },
            RustJson::String(s) => {
                Json::String(JsonString::new(s.into_string()))
            },
            RustJson::Number(n) => Json::Number(n),
            RustJson::True => Json::True,
            RustJson::False => Json::False,
            RustJson::Null => Json::Null,
        }
    }
}

/// Deserializes the given string into a Json struct.
/// If any error is encountered while parsing, the
/// type of the Json struct is `Json::Error`.
///
/// The caller of this function must free the returned
/// struct by calling [`json_free`] afterwards.
///
/// # Safety
/// The pointer must be a valid NULL terminated C string
#[no_mangle]
pub unsafe extern "C"
fn json_deserialize(ptr: *const c_char) -> Json {
    let cstr = unsafe { CStr::from_ptr(ptr) };
    if let Ok(s) = cstr.to_str() {
        if let Ok(json) = crate::Json::deserialize(s) {
            return Json::from_json(json);
        }
    }
    Json::Error
}

/// Deserializes the given string into a Json struct.
/// Using the given `JsonConfig` struct
///
/// If any error is encountered while parsing, the
/// type of the Json struct is `Json::Error`.
///
/// The caller of this function must free the returned
/// struct by calling [`json_free`] afterwards.
///
/// # Safety
/// The pointer must be a valid NULL terminated C string
#[no_mangle]
pub unsafe extern "C"
fn json_deserialize_with_config(ptr: *const c_char, conf: crate::JsonConfig) -> Json {
    let cstr = unsafe { CStr::from_ptr(ptr) };
    if let Ok(s) = cstr.to_str() {
        if let Ok(json) = crate::Json::deserialize_with_config(s, conf) {
            return Json::from_json(json);
        }
    }
    Json::Error
}

fn ptr_2_vec<T>(ptr: *mut T, len: usize) -> Vec<T> {
    let elems = unsafe {
        let elems = slice::from_raw_parts_mut(ptr, len);
        Box::from_raw(elems)
    };
    elems.into_vec()
}

fn vec_2_ptr<T>(vec: Vec<T>) -> *mut T {
    let mut data = vec.into_boxed_slice();
    let elems = data.as_mut_ptr();
    mem::forget(data);
    elems
}

/// Frees the given Json structure.
#[no_mangle]
pub extern "C"
fn json_free(json: Json) {
    match json{
        Json::Array{ elems, len } => {
            let elems = ptr_2_vec(elems, len);
            for e in elems {
                json_free(e);
            }
        },
        Json::Object{ elems, len } => {
            let elems = ptr_2_vec(elems, len);
            for Pair { key, val } in elems {
                mem::drop( key );
                let val = unsafe { Box::from_raw(val) };
                json_free(*val);
            }
        },
        Json::String(s) => {
            mem::drop(s);
        },
        Json::Number(_) |
        Json::True | Json::False |
        Json::Null | Json::Error => {},
    }
}
