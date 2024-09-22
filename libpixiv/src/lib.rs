extern crate md5;
extern crate reqwest;
extern crate serde_json;

use chrono::{Local};
use serde_json::{Value};
use std::sync::{Arc, Mutex};
use reqwest::header::{HeaderName, USER_AGENT, CONTENT_TYPE};


struct PixivAppClient {
    /// bearer token
    access_token: Arc<Mutex<String>>,
    refresh_token: Arc<Mutex<String>>,
    http_client: reqwest::Client,
    host: &str,
}

impl PixivAppClient {
    fn new(token: &str) -> Self {
        let client = Self {
            access_token: Arc::new(Mutex::new(String::new())),
            refresh_token: Arc::new(Mutex::new(String::from(token))),
            http_client: reqwest::Client::new(),
            host: "https://app-api.pixiv.net/"
        };
        client
    }

    fn md5(input: &str) -> String {
        let result = md5::compute(input);
        format!("{:02x}", result)
    }

    pub async fn refresh_token(&mut self) {
        let time = Local::now().format("%y-%m-%dT%H:%m:%s+00:00");
        let time_str = format!("{}", time);
        let cloned_refresh_token = Arc::clone(&self.refresh_token);
        let cloned_refresh_token_str = &cloned_refresh_token.lock().unwrap();

        let client_id = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
        let client_secret = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";
        let hash_input = format!("{}{}\n", &time_str, "28c1fdd170a5204386cb1313c7077b34f83e4aaf4aa829ce78c231e05b0bae2c");
        let hash = PixivAppClient::md5(hash_input.as_str());

        let req = self.http_client
            .post("https://oauth.secure.pixiv.net/auth/token")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, "PixivAndroidApp/5.0.115 (Android 6.0; PixivBot)")
            .header(HeaderName::from_lowercase(b"x-client-time").unwrap(), &time_str)
            .header(HeaderName::from_lowercase(b"x-client-hash").unwrap(), hash)
            .body(format!("grant_type=refresh_token&client_id={}&refresh_token={}&client_secret={}&get_secure_url=1", client_id, cloned_refresh_token_str, client_secret))
            .build()
            .expect("failed to build login request");

        if let Some(body) = req.body() {
            if let Some(bytes) = body.as_bytes() {
                println!("Body: {}", String::from_utf8_lossy(bytes));
            } else {
                println!("Body: Non-text data or stream");
            }
        }

        let r = match self.http_client
            .execute(req)
            .await {
            Ok(r) => r.text().await.unwrap(),
            Err(_e) => return
        };
        let d: Value = serde_json::from_str(&r).unwrap();

        assert!(!d["response"]["access_token"].is_null());
        assert!(!d["response"]["refresh_token"].is_null());

        self.access_token = Arc::new(Mutex::new(String::from(d["response"]["access_token"].as_str().unwrap())));
        self.refresh_token = Arc::new(Mutex::new(String::from(d["response"]["refresh_token"].as_str().unwrap())));
    }
}


#[cfg(test)]
mod client_tests {
    use super::*;
    use std::{assert, panic, env};

    #[tokio::test]
    async fn login() {
        let token = env::var("PIXIV_REFRESH_TOKEN");
        let mut client = PixivAppClient::new(&token.expect("expecting PIXIV_REFRESH_TOKEN variable for testing!"));
        client.refresh_token().await;
        let cloned_access_token = Arc::clone(&client.access_token);
        match cloned_access_token.lock() {
            Ok(t) => assert!(!t.is_empty()),
            Err(_) => panic!("No token received!"),
        };
    }


}