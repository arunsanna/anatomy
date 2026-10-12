use crate::{metal::Metal, model::Model};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{MTLCommandBuffer, MTLDevice, MTLTexture};
use std::ffi::{c_char, c_void, CStr, CString};
struct Renderer {
    metal: Metal,
    model: Option<Model>,
    error: String,
}
fn string(value: String) -> *mut c_char {
    CString::new(value.replace('\0', "�")).unwrap().into_raw()
}
unsafe fn renderer<'a>(ptr: *mut c_void) -> Option<&'a mut Renderer> {
    ptr.cast::<Renderer>().as_mut()
}
fn finish(r: &mut Renderer, result: Result<(), String>) -> i32 {
    match result {
        Ok(()) => {
            r.error.clear();
            1
        }
        Err(e) => {
            r.error = e;
            0
        }
    }
}
/// Caller supplies a valid borrowed Metal device; wrapper serializes access.
#[no_mangle]
pub unsafe extern "C" fn anatomy_renderer_create(device: *mut c_void) -> *mut c_void {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    let device = Retained::retain(device.cast::<ProtocolObject<dyn MTLDevice>>()).unwrap();
    match Metal::new(device) {
        Ok(metal) => Box::into_raw(Box::new(Renderer {
            metal,
            model: None,
            error: String::new(),
        }))
        .cast(),
        Err(_) => std::ptr::null_mut(),
    }
}
#[no_mangle]
pub unsafe extern "C" fn anatomy_renderer_load_scene(
    ptr: *mut c_void,
    json: *const u8,
    length: usize,
) -> i32 {
    let Some(r) = renderer(ptr) else {
        return 0;
    };
    if json.is_null() || length == 0 {
        return finish(r, Err("Scene JSON is empty".into()));
    }
    match Model::load(std::slice::from_raw_parts(json, length)) {
        Ok(model) => {
            r.model = Some(model);
            finish(r, Ok(()))
        }
        Err(e) => finish(r, Err(e)),
    }
}
#[no_mangle]
pub unsafe extern "C" fn anatomy_renderer_set_control(
    ptr: *mut c_void,
    id: *const c_char,
    value: f64,
) -> i32 {
    let Some(r) = renderer(ptr) else {
        return 0;
    };
    let result = (|| {
        if id.is_null() {
            return Err("Control id is null".into());
        }
        let id = CStr::from_ptr(id)
            .to_str()
            .map_err(|_| "Control id must be UTF-8")?;
        r.model
            .as_mut()
            .ok_or("No scene loaded")?
            .set_control(id, value)
    })();
    finish(r, result)
}
#[no_mangle]
pub unsafe extern "C" fn anatomy_renderer_encode(
    ptr: *mut c_void,
    command: *mut c_void,
    texture: *mut c_void,
    time: f64,
    yaw: f64,
    pitch: f64,
    distance: f64,
) -> i32 {
    let Some(r) = renderer(ptr) else {
        return 0;
    };
    let result = (|| {
        if command.is_null() || texture.is_null() {
            return Err("Command buffer and texture must be nonnull".into());
        }
        let model = r.model.as_ref().ok_or("No scene loaded")?;
        r.metal.encode(
            model,
            &*command.cast::<ProtocolObject<dyn MTLCommandBuffer>>(),
            &*texture.cast::<ProtocolObject<dyn MTLTexture>>(),
            time,
            yaw,
            pitch,
            distance,
        )
    })();
    finish(r, result)
}
#[no_mangle]
pub unsafe extern "C" fn anatomy_renderer_state_json(ptr: *mut c_void, time: f64) -> *mut c_char {
    let Some(r) = renderer(ptr) else {
        return std::ptr::null_mut();
    };
    match r
        .model
        .as_ref()
        .ok_or("No scene loaded".to_owned())
        .and_then(|m| m.state(time))
        .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
    {
        Ok(value) => {
            r.error.clear();
            string(value)
        }
        Err(e) => {
            r.error = e;
            std::ptr::null_mut()
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn anatomy_renderer_last_error(ptr: *mut c_void) -> *mut c_char {
    renderer(ptr)
        .map(|r| string(r.error.clone()))
        .unwrap_or_else(|| string("Renderer is null".into()))
}
#[no_mangle]
pub unsafe extern "C" fn anatomy_string_free(text: *mut c_char) {
    if !text.is_null() {
        drop(CString::from_raw(text));
    }
}
#[no_mangle]
pub unsafe extern "C" fn anatomy_renderer_destroy(ptr: *mut c_void) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr.cast::<Renderer>()));
    }
}
