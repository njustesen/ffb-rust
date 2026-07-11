/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblGameState.
pub struct FumbblGameState {
    pub url: String,
    pub result: String,
    pub reason: String,
    pub description: String,
    pub game_id: String,
}

impl FumbblGameState {
    pub const XML_TAG: &'static str = "gamestate";

    pub fn new(url: String) -> Self {
        Self {
            url,
            result: String::new(),
            reason: String::new(),
            description: String::new(),
            game_id: String::new(),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.result.eq_ignore_ascii_case("ok")
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_result(&self) -> &str {
        &self.result
    }

    pub fn get_reason(&self) -> &str {
        &self.reason
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_game_id(&self) -> &str {
        &self.game_id
    }

    pub fn to_xml(&self) -> String {
        let mut xml = format!("<{}>", Self::XML_TAG);
        xml.push_str(&format!("<url>{}</url>", self.url));
        xml.push_str(&format!("<result>{}</result>", self.result));
        xml.push_str(&format!("<reason>{}</reason>", self.reason));
        xml.push_str(&format!("<description>{}</description>", self.description));
        xml.push_str(&format!("<gameid>{}</gameid>", self.game_id));
        xml.push_str(&format!("</{}>", Self::XML_TAG));
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let g = FumbblGameState::new("http://example.com".to_string());
        assert!(!g.is_ok());
    }

    #[test]
    fn is_ok() {
        let mut g = FumbblGameState::new(String::new());
        g.result = "ok".to_string();
        assert!(g.is_ok());
        g.result = "OK".to_string();
        assert!(g.is_ok());
    }

    #[test]
    fn to_xml_contains_all_fields() {
        let mut g = FumbblGameState::new("http://x".to_string());
        g.result = "ok".to_string();
        g.game_id = "42".to_string();
        let xml = g.to_xml();
        assert!(xml.starts_with("<gamestate>"));
        assert!(xml.ends_with("</gamestate>"));
        assert!(xml.contains("<url>http://x</url>"));
        assert!(xml.contains("<gameid>42</gameid>"));
    }
}
