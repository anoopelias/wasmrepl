use crate::model::Index;

pub struct Response {
    pub control: Control,
    messages: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Control {
    ExecFunc(Index),
    Return,
    None,
    If(bool),
}

impl Response {
    pub fn new() -> Response {
        Response {
            messages: Vec::new(),
            control: Control::None,
        }
    }

    pub fn new_index(ty: &str, i: usize, id: Option<String>) -> Response {
        match id {
            Some(id) => Response::new_message(format!("{} ;{}; {}", ty, i, id)),
            None => Response::new_message(format!("{} ;{};", ty, i)),
        }
    }

    pub fn extend(&mut self, other: Response) {
        self.messages.extend(other.messages);
        self.control = other.control;
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn message(&self) -> String {
        self.messages.join("\n")
    }

    pub fn new_ctrl(ctrl: Control) -> Response {
        Response {
            messages: Vec::new(),
            control: ctrl,
        }
    }

    fn new_message(message: String) -> Response {
        Response {
            messages: vec![message],
            control: Control::None,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        model::Index,
        response::{Control, Response},
    };

    #[test]
    fn test_new() {
        let resp = Response::new();
        assert_eq!(resp.message(), "");
        assert_eq!(resp.control, Control::None);
    }

    #[test]
    fn test_new_index() {
        let resp = Response::new_index("local", 0, None);
        assert_eq!(resp.message(), "local ;0;");
    }

    #[test]
    fn test_new_index_with_id() {
        let resp = Response::new_index("local", 0, Some("foo".to_string()));
        assert_eq!(resp.message(), "local ;0; foo");
    }

    #[test]
    fn test_extend() {
        let mut resp1 = Response::new_index("local", 0, None);
        let resp2 = Response::new_index("local", 1, None);
        resp1.extend(resp2);
        assert_eq!(resp1.message(), "local ;0;\nlocal ;1;");
    }

    #[test]
    fn test_add_messages() {
        let mut resp = Response::new();
        resp.add_message("foo".to_string());
        resp.add_message("bar".to_string());
        assert_eq!(resp.message(), "foo\nbar");
    }

    #[test]
    fn test_new_return() {
        let resp = Response::new_ctrl(Control::Return);
        assert_eq!(resp.message(), "");
        assert_eq!(resp.control, Control::Return);
    }

    #[test]
    fn test_new_exec_func() {
        let resp = Response::new_ctrl(Control::ExecFunc(Index::Id(String::from("fn"))));
        assert_eq!(resp.message(), "");
        match resp.control {
            Control::ExecFunc(Index::Id(str)) => assert_eq!(str, "fn"),
            _ => panic!("expected ExecFunc"),
        }
    }
}
