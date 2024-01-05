use {
    actix_web::{
        post, Error, HttpResponse,
    },
    crate::database::{EULAData, get_latest_eula},
    shared::responses::ms_generic_response,
};

#[post("/eula")]
pub async fn get_eula() -> Result<HttpResponse, Error> {
    let eula = match get_latest_eula(String::from("english")).await {
        Some(eula) => eula,
        None => { EULAData {version: 0, lang: String::new(), contents: String::new()}   }
    };
    
    Ok(HttpResponse::Ok().body(ms_generic_response(String::from("data"), serde_json::to_string(&eula).unwrap())))
}