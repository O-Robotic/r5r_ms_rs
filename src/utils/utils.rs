use serde::Serialize;

#[derive(Default, Serialize)]
pub struct MSErrorJson {
    success: bool,
    error: String,
}

pub fn ms_error_format(str: impl Into<String>) -> String {
    return serde_json::to_string(&MSErrorJson{
        success: false, 
        error: str.into()}
    ).unwrap();
}
