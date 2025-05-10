use axum::http::{HeaderMap, HeaderValue};
use time::{Duration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};
use cookie::SameSite;
use tracing::debug;

pub const ACCESS_TOKEN_COOKIE: &str = "access_token";
pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
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

        // Each Set-Cookie header should be in its own header field
        headers.append(
            "Set-Cookie",
            HeaderValue::from_str(&access_cookie.to_string()).unwrap(),
        );
        headers.append(
            "Set-Cookie",
            HeaderValue::from_str(&refresh_cookie.to_string()).unwrap(),
        );

        headers
    }

    pub fn clear_auth_cookies() -> HeaderMap {
        debug!("Clearing auth cookies");
        let mut headers = HeaderMap::new();
        
        // Clear access token
        let access_cookie = Self::create_removal_cookie(ACCESS_TOKEN_COOKIE);
        debug!("Access cookie removal header: {}", access_cookie.to_string());
        
        // Clear refresh token
        let refresh_cookie = Self::create_removal_cookie(REFRESH_TOKEN_COOKIE);
        debug!("Refresh cookie removal header: {}", refresh_cookie.to_string());

        // Each Set-Cookie header should be in its own header field
        headers.append(
            "Set-Cookie",
            HeaderValue::from_str(&access_cookie.to_string()).unwrap(),
        );
        headers.append(
            "Set-Cookie",
            HeaderValue::from_str(&refresh_cookie.to_string()).unwrap(),
        );

        debug!("Final headers for cookie removal: {:?}", headers);
        headers
    }

    pub fn extract_token(headers: &HeaderMap, token_name: &str) -> Option<String> {
        debug!("Attempting to extract token: {}", token_name);
        debug!("All headers: {:?}", headers);
        
        headers
            .get_all("cookie")
            .iter()
            .find_map(|cookie_header| {
                let cookie_str = match cookie_header.to_str() {
                    Ok(s) => {
                        debug!("Processing cookie header: {}", s);
                        s
                    },
                    Err(e) => {
                        debug!("Failed to convert cookie header to string: {}", e);
                        return None;
                    }
                };

                // Split cookie string by semicolon and parse each cookie individually
                for single_cookie_str in cookie_str.split(';') {
                    let trimmed_cookie_str = single_cookie_str.trim();
                    debug!("Processing individual cookie: {}", trimmed_cookie_str);
                    
                    if let Ok(cookie) = Cookie::parse(trimmed_cookie_str) {
                        debug!("Successfully parsed cookie: {}={}", cookie.name(), cookie.value());
                        if cookie.name() == token_name {
                            return Some(cookie.value().to_string());
                        }
                    }
                }
                None
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
        debug!("Creating removal cookie for: {}", name);
        let mut cookie = Cookie::new(name.to_owned(), String::new());
        cookie.set_path("/");
        cookie.make_removal();
        debug!("Removal cookie created: {}", cookie.to_string());
        cookie
    }
}
