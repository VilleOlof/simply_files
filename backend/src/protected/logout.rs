use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};

pub async fn logout(jar: CookieJar) -> CookieJar {
    let mut cookie = Cookie::new("token", "");
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::None);
    cookie.set_http_only(true);

    jar.remove(cookie)
}
