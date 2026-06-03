#![allow(unused)]

use std::collections::HashMap;
use std::time::Duration;

pub struct Url {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: String,
    client: reqwest::blocking::Client,
    async_client: reqwest::Client,
}

impl Url {
    pub fn new(
        url: &str,
        method: &str,
        headers: HashMap<String, String>,
        body: &str,
    ) -> Self {
        Self::new_with_timeout(url, method, headers, body, Duration::from_secs(60))
    }

    pub fn new_with_timeout(
        url: &str,
        method: &str,
        headers: HashMap<String, String>,
        body: &str,
        timeout: Duration,
    ) -> Self {
        Url {
            url: url.to_string(),
            method: method.to_string(),
            headers,
            body: body.to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap(),
            async_client: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap(),
        }
    }

    // Set request url
    pub fn setUrl(&mut self, url: &str) {
        self.url = url.to_string();
    }

    // Set request method: GET POST PUT DELETE
    pub fn setMethod(&mut self, method: &str) {
        self.method = method.to_string();
    }

    // Set header key:value
    pub fn setHeaders(&mut self, headers: HashMap<String, String>) {
        self.headers = headers;
    }

    // Set request body
    pub fn setBody(&mut self, body: &str) {
        self.body = body.to_string();
    }

    // Get request url
    pub fn getUrl(&self) -> &str {
        &self.url
    }

    // Async send request and return (status code, headers, body)
    pub async fn send(&self) -> (usize, HashMap<String, String>, String) {
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &self.headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_bytes(value.as_bytes()).unwrap(),
            );
        }
        let req = match self.method.as_str() {
            "GET" => self.async_client.get(&self.url),
            "POST" => self.async_client.post(&self.url),
            "PUT" => self.async_client.put(&self.url),
            "DELETE" => self.async_client.delete(&self.url),
            _ => panic!("unsupported method, only support GET POST PUT DELETE"),
        };
        let req = req.headers(headers);
        let req = if self.body.len() > 0 {
            req.body(self.body.clone())
        } else {
            req
        };
        let resp = req.send().await;
        match resp {
            Ok(resp) => {
                let status = resp.status().as_u16() as usize;
                let mut resp_headers = HashMap::new();
                for (key, value) in resp.headers() {
                    resp_headers.insert(key.to_string(), value.to_str().unwrap().to_string());
                }
                let resp_body = resp.text().await.unwrap();
                (status, resp_headers, resp_body)
            }
            Err(e) => {
                if e.is_timeout() {
                    (408, HashMap::new(), "Request timeout".to_string())
                } else {
                    (500, HashMap::new(), "Internal server error".to_string())
                }
            }
        }
    }

    // Sync send request and return (status code, headers, body)
    pub fn sendSync(&self) -> (usize, HashMap<String, String>, String) {
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &self.headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_bytes(value.as_bytes()).unwrap(),
            );
        }
        let req = match self.method.as_str() {
            "GET" => self.client.get(&self.url),
            "POST" => self.client.post(&self.url),
            "PUT" => self.client.put(&self.url),
            "DELETE" => self.client.delete(&self.url),
            _ => panic!("unsupported method, only support GET POST PUT DELETE"),
        };
        let req = req.headers(headers);
        let req = if self.body.len() > 0 {
            req.body(self.body.clone())
        } else {
            req
        };
        let resp = req.send();
        match resp {
            Ok(resp) => {
                let status = resp.status().as_u16() as usize;
                let mut resp_headers = HashMap::new();
                for (key, value) in resp.headers() {
                    resp_headers.insert(key.to_string(), value.to_str().unwrap().to_string());
                }
                let resp_body = resp.text().unwrap();
                (status, resp_headers, resp_body)
            }
            Err(e) => {
                if e.is_timeout() {
                    (408, HashMap::new(), "Request timeout".to_string())
                } else {
                    (500, HashMap::new(), "Internal server error".to_string())
                }
            }
        }
    }
}
