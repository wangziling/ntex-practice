pub struct ViewTemplateBase {
    pub title: String,
    pub description: String,
    pub language: String,
}

impl Default for ViewTemplateBase {
    fn default() -> Self {
        Self {
            title: "Axum web.".to_string(),
            description: "A simple web application based on Rust-Axum.".to_string(),
            language: "en-US".to_string(),
        }
    }
}
