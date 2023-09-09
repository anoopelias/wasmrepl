pub struct Response {
    messages: Vec<String>,
}

impl Response {
    pub fn new(message: String) -> Response {
        Response {
            messages: vec![message],
        }
    }

    pub fn message(&self) -> String {
        self.messages.join("\n")
    }
}
