use {
    actix_web::get,
    maud::{html, Markup, DOCTYPE},
};

#[get("/")]
pub async fn panel_main_menu() -> actix_web::Result<Markup> {
    Ok(html! {
        (DOCTYPE)
        meta name="viewport" content="width=device-width, initial-scale=1.0";
        html lang = "en" {
            h1 {"Management"}

            a href = "/panel/moderation/player" {"Player Moderation"}
            br;

            a href = "/panel/list" {"Server List"}
            br;

            //a href = "/panel/config" {"Configuration"}
            //br;
            
            a href = "/panel/logout" {"Logout"}
            br;
        }
    })
}