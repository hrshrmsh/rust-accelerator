use auth_service::Application;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let app = Application::build("127.0.0.1:0")
            .await
            .expect("could not build application");
        let address = format!("http://{}", &app.address);

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();

        Self {
            address,
            http_client,
        }
    }

    #[inline]
    pub async fn get_root(&self) -> reqwest::Response {
        self.get("/").await
    }

    #[inline]
    pub async fn get_signup(&self) -> reqwest::Response {
        self.post("/signup").await
    }

    #[inline]
    pub async fn get_login(&self) -> reqwest::Response {
        self.post("/login").await
    }

    #[inline]
    pub async fn get_logout(&self) -> reqwest::Response {
        self.post("/logout").await
    }

    #[inline]
    pub async fn get_verify_2fa(&self) -> reqwest::Response {
        self.post("/verify-2fa").await
    }

    #[inline]
    pub async fn get_verify_token(&self) -> reqwest::Response {
        self.post("/verify-token").await
    }

    #[inline]
    async fn get(&self, addr: &str) -> reqwest::Response {
        self.http_client
            .get(&format!("{}{addr}", &self.address))
            .send()
            .await
            .expect("could not execute request")
    }

    #[inline]
    async fn post(&self, addr: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}{addr}", &self.address))
            .send()
            .await
            .expect("could not execute request")
    }
}
