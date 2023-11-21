pub fn auth_get(url: url::Url, access_token: &str) -> reqwest::blocking::RequestBuilder {
    let client = reqwest::blocking::Client::new();
    client.get(url).header("accessToken", access_token)
}
