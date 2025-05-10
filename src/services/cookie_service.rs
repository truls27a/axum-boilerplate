use axum::http::{HeaderMap, HeaderValue};
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};
use cookie::SameSite;

const ACCESS_TOKEN_COOKIE: &str = "access_token";
const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
const SECURE: bool = true; // Set to true for HTTPS
const HTTP_ONLY: bool = true;
const SAME_SITE: SameSite = SameSite::Strict;


#[derive(Clone)]
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
        
        let mut cookie = Cookie::new(name.to_owned(), value.to_owned());
        cookie.set_secure(SECURE);
        cookie.set_http_only(HTTP_ONLY);
        cookie.set_same_site(Some(SAME_SITE));
        cookie.set_path("/");
        cookie.set_expires(expires);
        cookie
    }

    fn create_removal_cookie(name: &str) -> Cookie<'static> {
        let mut cookie = Cookie::new(name.to_owned(), String::new());
        cookie.set_secure(SECURE);
        cookie.set_http_only(HTTP_ONLY);
        cookie.set_same_site(Some(SAME_SITE));
        cookie.set_path("/");
        cookie.set_expires(OffsetDateTime::now_utc() - Duration::days(1));
        cookie
    }
}
