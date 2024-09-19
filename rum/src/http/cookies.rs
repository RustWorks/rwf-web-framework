use std::collections::HashMap;
use time::{Duration, OffsetDateTime};

use super::url::urldecode;
use super::Error;
use crate::config::get_config;
use crate::crypto::{decrypt, encrypt};

#[derive(Debug, Clone, Default)]
pub struct Cookies {
    cookies: HashMap<String, Cookie>,
}

impl Cookies {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    pub fn parse(value: &str) -> Cookies {
        let parts = value.split(";");
        let mut cookies = HashMap::new();

        for part in parts {
            if let Some(cookie) = Cookie::parse(part.trim()) {
                cookies.insert(cookie.name.to_string(), cookie);
            }
        }

        Cookies { cookies }
    }

    pub fn add_private(&mut self, cookie: impl ToCookie) -> Result<(), Error> {
        let mut cookie = cookie.to_cookie();
        cookie.value = encrypt(cookie.value.as_bytes())?;
        self.cookies.insert(cookie.name.clone(), cookie);

        Ok(())
    }

    pub fn get_private(mut self, name: &str) -> Result<Option<Cookie>, Error> {
        if let Some(cookie) = self.cookies.get(name) {
            let mut cookie = cookie.clone();
            cookie.value = String::from_utf8(decrypt(&cookie.value)?)?;
            Ok(Some(cookie))
        } else {
            Ok(None)
        }
    }

    pub fn add(&mut self, cookie: impl ToCookie) {
        let cookie = cookie.to_cookie();
        self.cookies.insert(cookie.name.clone(), cookie);
    }

    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.cookies.get(name)
    }
}

impl std::fmt::Display for Cookies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut result = Vec::new();
        for (_name, cookie) in &self.cookies {
            result.push(cookie.to_string());
        }
        write!(f, "{}", result.join("; "))
    }
}

pub trait ToCookie {
    fn to_cookie(self) -> Cookie;
}

impl ToCookie for (&str, &str) {
    fn to_cookie(self) -> Cookie {
        let mut builder = CookieBuilder::new();
        builder.name(self.0).value(self.1).build()
    }
}

impl ToCookie for (String, String) {
    fn to_cookie(self) -> Cookie {
        let mut builder = CookieBuilder::new();
        builder.name(self.0).value(self.1).build()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cookie {
    name: String,
    value: String,
    expiration: Option<OffsetDateTime>,
    max_age: Option<Duration>,
    path: Option<String>,
    domain: Option<String>,
    http_only: bool,
    secure: bool,
}

impl Cookie {
    pub fn parse(value: &str) -> Option<Self> {
        let mut parts = value.split(";");
        let mut builder = CookieBuilder::new();
        let mut cookie = if let Some(cookie) = parts.next() {
            match Self::key_value(cookie) {
                (Some(key), Some(value)) => builder = builder.name(&key).value(urldecode(&value)),
                (Some(key), None) => builder = builder.name(&key),
                _ => return None,
            }
        } else {
            return None;
        };

        for part in parts {
            match Self::key_value(part) {
                (Some(key), value) => match key.as_str().trim() {
                    "Domain" => {
                        if let Some(value) = value {
                            builder = builder.domain(value);
                        }
                    }
                    "HttpOnly" => {
                        builder = builder.http_only();
                    }
                    "Secure" => {
                        builder = builder.secure();
                    }
                    "Max-Age" => {
                        if let Some(value) = value {
                            match value.parse::<i64>() {
                                Ok(value) => {
                                    builder = builder.max_age(Duration::seconds(value));
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                    _ => continue,
                },

                _ => continue,
            };
        }

        Some(builder.build())
    }

    fn key_value(s: &str) -> (Option<String>, Option<String>) {
        let mut parts = s.split("=");
        if let Some(key) = parts.next() {
            if let Some(value) = parts.next() {
                (Some(key.to_owned()), Some(value.to_owned()))
            } else {
                (Some(key.to_owned()), None)
            }
        } else {
            (None, None)
        }
    }

    pub fn new(name: impl ToString) -> Self {
        Cookie {
            name: name.to_string(),
            max_age: Some(get_config().cookie_max_age),
            ..Default::default()
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn secure(&self) -> bool {
        self.secure
    }

    pub fn http_only(&self) -> bool {
        self.http_only
    }

    pub fn max_age(&self) -> Option<Duration> {
        self.max_age
    }
}

impl std::fmt::Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;
        if let Some(ref max_age) = self.max_age {
            write!(f, "; Max-Age: {}", max_age.whole_seconds())?;
        }

        if self.secure {
            write!(f, "; Secure")?;
        }

        if self.http_only {
            write!(f, "; HttpOnly")?;
        }

        if let Some(ref path) = self.path {
            write!(f, "; Path={}", path)?;
        }

        if let Some(ref domain) = self.domain {
            write!(f, "; Domain={}", domain)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct CookieBuilder {
    cookie: Cookie,
}

impl CookieBuilder {
    pub fn new() -> Self {
        Self {
            cookie: Cookie::default(),
        }
    }

    pub fn name(mut self, name: impl ToString) -> Self {
        self.cookie.name = name.to_string();
        self
    }

    pub fn value(mut self, value: impl ToString) -> Self {
        self.cookie.value = value.to_string();
        self
    }

    pub fn expiration(mut self, expiration: OffsetDateTime) -> Self {
        self.cookie.expiration = Some(expiration);
        self
    }

    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.cookie.max_age = Some(max_age);
        self
    }

    pub fn path(mut self, path: impl ToString) -> Self {
        self.cookie.path = Some(path.to_string());
        self
    }

    pub fn domain(mut self, domain: impl ToString) -> Self {
        self.cookie.domain = Some(domain.to_string());
        self
    }

    pub fn http_only(mut self) -> Self {
        self.cookie.http_only = true;
        self
    }

    pub fn secure(mut self) -> Self {
        self.cookie.secure = true;
        self
    }

    pub fn build(self) -> Cookie {
        self.cookie
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parsing_cookies() {
        let value = "name=some_value; Max-Age=55; Secure";
        let cookie = Cookie::parse(value).expect("cookie parse");
        assert_eq!(cookie.name(), "name");
        assert_eq!(cookie.value(), "some_value");
        assert!(cookie.secure());
        assert_eq!(cookie.max_age(), Some(Duration::seconds(55)));

        let value = "random=hello_world";
        let cookie = Cookie::parse(value).expect("cookie parse");
        assert_eq!(cookie.name(), "random");
        assert_eq!(cookie.value(), "hello_world");
    }

    #[test]
    fn test_creating_cookies() {
        let mut cookies = Cookies::new();
        cookies.add(("hello", "world"));
        cookies
            .add_private(("session", "super_secret_key"))
            .expect("private");
        let s = cookies.to_string();

        let cookies = Cookies::parse(&s);
        assert!(cookies.get("hello").is_some());
        assert_eq!(cookies.get("hello").unwrap().value(), "world");
        assert_eq!(
            cookies
                .get_private("session")
                .expect("decrypt")
                .expect("session cookie")
                .value(),
            "super_secret_key"
        );
    }
}