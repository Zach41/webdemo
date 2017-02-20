use url_ext::{self, Host};

use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Url {
    inner: url_ext::Url,
}

impl Url {
    pub fn parse(input: &str) -> Result<Url, String> {
        match url_ext::Url::parse(input) {
            Ok(url) => Url::from_generic_url(url),
            Err(e) => Err(format!("Parsing Error: {}", e)),
        }
    }

    pub fn from_generic_url(url: url_ext::Url) -> Result<Url, String> {
        if url.cannot_be_a_base() {
            Err(format!("Not a sepecial scheme: {}", url.scheme()))
        } else if url.port_or_known_default().is_none() {
            Err(format!("Invalid URL for scheme: {}", url.scheme()))
        } else {
            Ok(
                Url {
                    inner: url,
                }
            )
        }
    }

    pub fn scheme(&self) -> &str {
        self.inner.scheme()
    }

    pub fn host(&self) -> Host<&str> {
        self.inner.host().unwrap()
    }

    pub fn port(&self) -> u16 {
        self.inner.port_or_known_default().unwrap()
    }

    pub fn path(&self) -> Vec<&str> {
        self.inner.path_segments().unwrap().collect()
    }

    pub fn username(&self) -> Option<&str> {
        match self.inner.username() {
            "" => None,
            s => Some(s),
        }
    }

    pub fn password(&self) -> Option<&str> {
        match self.inner.password() {
            None => None,
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
        }
    }

    pub fn query(&self) -> Option<&str> {
        match self.inner.query() {
            None => None,
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
        }
    }

    pub fn fragment(&self) -> Option<&str> {
        match self.inner.fragment() {
            None => None,
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s),
        }
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }    
}

impl fmt::Debug for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Into<url_ext::Url> for Url {
    fn into(self) -> url_ext::Url {
        self.inner
    }
}

impl AsRef<url_ext::Url> for Url {
    fn as_ref(&self) -> &url_ext::Url {
        &self.inner
    }
}

impl AsMut<url_ext::Url> for Url {
    fn as_mut(&mut self) -> &mut url_ext::Url {
        &mut self.inner
    }
}

