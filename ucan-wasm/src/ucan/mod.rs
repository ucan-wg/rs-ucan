pub mod cid;
pub mod token;
pub mod verify;

pub type JsResult<T> = Result<T, js_sys::Error>;
