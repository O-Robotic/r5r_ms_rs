use {
    crate::{
        database::website_auth,
        endpoints::panel::{get_login_js, get_ms_post_js},
    },
    actix_session::Session,
    actix_web::{
        error, Error, get, post,
        web::{self, Redirect},
    },
    maud::{html, Markup, PreEscaped, DOCTYPE},
    serde::Deserialize,
};

#[derive(Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

#[get("/login")]
pub async fn login_page() -> actix_web::Result<Markup> {
    Ok(html! {
        (DOCTYPE)
        meta name="viewport" content="width=device-width, initial-scale=1.0";
        html lang = "en" {
            (PreEscaped(
                "<style>

                        body {
                            text-align: center;
                        }

                        input {
                            margin: 0.25rem;
                        }

                        form {
                            display: inline-block;
                        }

                    </style>"
            ))

            (PreEscaped(get_ms_post_js()))
            (PreEscaped(get_login_js()))

            title {"Login"}
            body {
                h1 {"Login"}
                form action = "#" {
                    label for = "username" {"Username:"}
                    br;
                    input type = "text" id = "username";
                    br;
                    label for = "password" {"Password:"}
                    br;
                    input type = "password" id = "password";
                }
                br;
                button onclick = "post_login()" {"Login"}
                p id = "login_message";

            }
        }
    })
}

#[post("/panel/auth")]
pub async fn panel_auth(form: web::Json<LoginInfo>, session: Session) -> Result<Redirect, Error> {
    let authed: bool = website_auth(&form.0.username, form.0.password).await;
    
    if authed {
        session.renew();
        session.insert("user", form.0.username)?;
        return Ok(Redirect::to("/panel/"))
    }

    Err(error::ErrorUnauthorized(""))
}

#[get("/logout")]
pub async fn panel_logout(session: Session) -> Redirect {
    session.purge();
    Redirect::to("/")
}
