use axum::http::{header::{SET_COOKIE, COOKIE}, HeaderMap};

pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
const COOKIE_DURATION_DAYS: i64 = 7;

pub struct CookieManager;

impl CookieManager {
    pub fn create_refresh_token_cookie(refresh_token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let cookie = format!(
            "{}={}; HttpOnly; Path=/; Max-Age={}; SameSite=Strict", 
            REFRESH_TOKEN_COOKIE,
            refresh_token,
            COOKIE_DURATION_DAYS * 24 * 60 * 60
        );
        headers.insert(SET_COOKIE, cookie.parse().unwrap());
        headers
    }

    pub fn clear_refresh_token_cookie() -> HeaderMap {
        let mut headers = HeaderMap::new();
        let cookie = format!(
            "{}=; HttpOnly; Path=/; Max-Age=0; SameSite=Strict", 
            REFRESH_TOKEN_COOKIE
        );
        headers.insert(SET_COOKIE, cookie.parse().unwrap());
        headers
    }

    pub fn extract_refresh_token(headers: &HeaderMap) -> Option<String> {
        headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';')
                    .find(|cookie| cookie.trim().starts_with(REFRESH_TOKEN_COOKIE))
                    .and_then(|cookie| cookie.split('=').nth(1))
                    .map(|token| token.trim().to_string())
            })
    }
} 