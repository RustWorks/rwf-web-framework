use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Query {
    query: HashMap<String, String>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            query: HashMap::new(),
        }
    }

    pub fn get<T: FromStr>(&self, name: &str) -> Option<T> {
        match self.query.get(name) {
            Some(value) => match value.parse::<T>() {
                Ok(value) => Some(value),
                Err(_) => None,
            },

            None => None,
        }
    }
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut params = vec![];
        for (key, value) in &self.query {
            params.push(format!("{}={}", key, value));
        }

        write!(f, "?{}", params.join("&"))
    }
}

impl Deref for Query {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

impl DerefMut for Query {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query
    }
}
