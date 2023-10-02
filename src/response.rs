use crate::model::{BlockType, Expression, Index};

pub struct Response {
    pub control: Control,
    pub requires_empty: bool,
    messages: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Control {
    ExecFunc(Index),
    ExecBlock(BlockType, Expression),
    Branch(Index),
    Return,
    None,
}

impl Control {
    fn requires_empty(&self) -> bool {
        match self {
            Control::Return => false,
            Control::Branch(_) => false,
            _ => true,
        }
    }
}

impl Response {
    pub fn new() -> Response {
        Response {
            messages: Vec::new(),
            control: Control::None,
            requires_empty: true,
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
        self.requires_empty = other.requires_empty;
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn message(&self) -> String {
        self.messages.join("\n")
    }

    pub fn new_ctrl(ctrl: Control) -> Response {
        let requires_empty = ctrl.requires_empty();
        Response {
            messages: Vec::new(),
            control: ctrl,
            requires_empty,
        }
    }

    fn new_message(message: String) -> Response {
        Response {
            messages: vec![message],
            control: Control::None,
            requires_empty: true,
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
        assert_eq!(resp.requires_empty, true);
    }

    #[test]
    fn test_new_index() {
        let resp = Response::new_index("local", 0, None);
        assert_eq!(resp.message(), "local ;0;");
        assert_eq!(resp.requires_empty, true);
    }

    #[test]
    fn test_new_index_with_id() {
        let resp = Response::new_index("local", 0, Some("foo".to_string()));
        assert_eq!(resp.message(), "local ;0; foo");
    }

    #[test]
    fn test_extend() {
        let mut resp1 = Response::new_index("local", 0, None);
        let mut resp2 = Response::new_index("local", 1, None);
        resp2.requires_empty = false;
        resp2.control = Control::Return;
        resp1.extend(resp2);
        assert_eq!(resp1.message(), "local ;0;\nlocal ;1;");
        assert_eq!(resp1.control, Control::Return);
        assert_eq!(resp1.requires_empty, false)
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
    fn test_requires_empty() {
        let resp = Response::new_ctrl(Control::ExecFunc(Index::Id(String::from("test"))));
        assert!(resp.requires_empty);
    }

    #[test]
    fn test_not_requires_empty() {
        let resp = Response::new_ctrl(Control::Return);
        assert!(!resp.requires_empty);

        let resp = Response::new_ctrl(Control::Branch(Index::Num(0)));
        assert!(!resp.requires_empty);
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
