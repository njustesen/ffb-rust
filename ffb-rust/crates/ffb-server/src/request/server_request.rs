/// 1:1 translation of com.fumbbl.ffb.server.request.ServerRequest.
pub trait ServerRequest {
    fn process(&self) -> Result<(), String>;
    fn get_request_url(&self) -> &str;
    fn set_request_url(&mut self, url: String);
}
