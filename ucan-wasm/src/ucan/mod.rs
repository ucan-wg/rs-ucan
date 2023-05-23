pub mod verify;

pub type JsResult<T> = Result<T, js_sys::Error>;
