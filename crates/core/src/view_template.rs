pub struct ViewTemplateBase {
    pub title: String,
    pub description: String,
    pub language: String,
}

impl Default for ViewTemplateBase {
    fn default() -> Self {
        Self {
            title: "Ntex web.".to_string(),
            description: "A simple web application based on Rust-Ntex.".to_string(),
            language: "en-US".to_string(),
        }
    }
}

pub trait ViewTemplate {
    fn title(&self) -> &str;
    fn description(&self) -> &str;
    fn language(&self) -> &str;
    fn set_title(&mut self, title: String) -> &mut Self;
    fn set_description(&mut self, description: String) -> &mut Self;
    fn set_language(&mut self, language: String) -> &mut Self;
}
