pub struct Response {
    pub message: String,
}

impl Response {
    pub fn new(message: String) -> Response {
        Response { message }
    }
}
