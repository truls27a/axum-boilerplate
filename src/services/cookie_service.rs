use axum::http::{HeaderMap, HeaderValue};
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};

const ACCESS_TOKEN_COOKIE: &str = "access_token";
const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
const SECURE: bool = true; // Set to true for HTTPS
const HTTP_ONLY: bool = true;
const SAME_SITE: tower_cookies::SameSite = tower_cookies::SameSite::Strict;

pub struct CookieService;

impl CookieService {
    pub fn set_auth_cookies(access_token: &str, refresh_token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        
        // Set access token cookie
        let access_cookie = Self::create_cookie(
            ACCESS_TOKEN_COOKIE,
            access_token,
            Duration::minutes(15), // 15 minutes for access token
        );
        
        // Set refresh token cookie
        let refresh_cookie = Self::create_cookie(
            REFRESH_TOKEN_COOKIE,
            refresh_token,
            Duration::days(7), // 7 days for refresh token
        );

        headers.insert(
            "Set-Cookie",
            HeaderValue::from_str(&access_cookie.to_string()).unwrap(),
        );
        headers.insert(
            "Set-Cookie",
            HeaderValue::from_str(&refresh_cookie.to_string()).unwrap(),
        );

        headers
    }

    pub fn clear_auth_cookies() -> HeaderMap {
        let mut headers = HeaderMap::new();
        
        // Clear access token
        let access_cookie = Self::create_removal_cookie(ACCESS_TOKEN_COOKIE);
        
        // Clear refresh token
        let refresh_cookie = Self::create_removal_cookie(REFRESH_TOKEN_COOKIE);

        headers.insert(
            "Set-Cookie",
            HeaderValue::from_str(&access_cookie.to_string()).unwrap(),
        );
        headers.insert(
            "Set-Cookie",
            HeaderValue::from_str(&refresh_cookie.to_string()).unwrap(),
        );

        headers
    }

    pub fn extract_refresh_token(headers: &HeaderMap) -> Option<String> {
        headers
            .get_all("cookie")
            .iter()
            .find_map(|cookie_header| {
                let cookie_str = cookie_header.to_str().ok()?;
                Cookie::parse(cookie_str)
                    .ok()
                    .filter(|c| c.name() == REFRESH_TOKEN_COOKIE)
                    .map(|c| c.value().to_string())
            })
    }

    fn create_cookie(name: &str, value: &str, max_age: Duration) -> Cookie<'static> {
        let expires = OffsetDateTime::now_utc() + max_age;
        
        Cookie::build(name, value.to_string())
            .secure(SECURE)
            .http_only(HTTP_ONLY)
            .same_site(SAME_SITE)
            .path("/")
            .expires(expires)
            .build()
    }

    fn create_removal_cookie(name: &str) -> Cookie<'static> {
        Cookie::build(name, "")
            .secure(SECURE)
            .http_only(HTTP_ONLY)
            .same_site(SAME_SITE)
            .path("/")
            .expires(OffsetDateTime::now_utc() - Duration::days(1))
            .build()
    }
}
