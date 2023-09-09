pub struct Response {
    messages: Vec<String>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            messages: Vec::new(),
        }
    }

    pub fn new_message(message: String) -> Response {
        Response {
            messages: vec![message],
        }
    }

    pub fn extend(&mut self, other: Response) {
        self.messages.extend(other.messages);
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn message(&self) -> String {
        self.messages.join("\n")
    }
}
