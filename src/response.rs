use crate::model::Index;

pub struct Response {
    pub contd: Control,
    pub is_return: bool,
    messages: Vec<String>,
}

pub enum Control {
    ExecFunc(Index),
    None,
}

impl Response {
    pub fn new() -> Response {
        Response {
            messages: Vec::new(),
            is_return: false,
            contd: Control::None,
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
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn message(&self) -> String {
        self.messages.join("\n")
    }

    pub fn new_return() -> Response {
        Response {
            messages: Vec::new(),
            is_return: true,
            contd: Control::None,
        }
    }

    pub fn new_exec_func(index: Index) -> Response {
        Response {
            messages: vec![],
            is_return: false,
            contd: Control::ExecFunc(index),
        }
    }

    fn new_message(message: String) -> Response {
        Response {
            messages: vec![message],
            is_return: false,
            contd: Control::None,
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
        assert!(!resp.is_return);
    }

    #[test]
    fn test_new_index() {
        let resp = Response::new_index("local", 0, None);
        assert_eq!(resp.message(), "local ;0;");
        assert!(!resp.is_return);
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
        let resp = Response::new_return();
        assert_eq!(resp.message(), "");
        assert!(resp.is_return);
    }

    #[test]
    fn test_new_exec_func() {
        let resp = Response::new_exec_func(Index::Id(String::from("fn")));
        assert_eq!(resp.message(), "");
        assert!(!resp.is_return);
        match resp.contd {
            Control::ExecFunc(Index::Id(str)) => assert_eq!(str, "fn"),
            _ => panic!("expected CallFunc"),
        }
    }
}
