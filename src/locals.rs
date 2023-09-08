use crate::{elements::Elements, model::Index, value::Value};
use anyhow::Result;

pub struct Locals {
    elements: Elements<Value>,
}

impl Locals {
    pub fn new() -> Locals {
        Locals {
            elements: Elements::new(),
        }
    }

    pub fn grow(&mut self, id: Option<String>, value: Value) -> Result<usize> {
        self.elements.grow(id, value)
    }

    pub fn set(&mut self, index: &Index, value: Value) -> Result<()> {
        self.elements.get(index)?.is_same(&value)?;
        self.elements.set(index, value)
    }

    pub fn get(&self, index: &Index) -> Result<&Value> {
        self.elements.get(index)
    }

    pub fn commit(&mut self) {
        self.elements.commit();
    }

    pub fn rollback(&mut self) {
        self.elements.rollback();
    }
}

#[cfg(test)]
mod tests {

    use crate::model::Index;

    fn test_index(id: &str) -> Index {
        Index::Id(String::from(id))
    }

    #[test]
    fn test_grow_get_set() {
        let mut locals = super::Locals::new();
        locals.grow(None, 0.into()).unwrap();
        locals.set(&Index::Num(0), 1.into()).unwrap();

        assert_eq!(locals.get(&Index::Num(0)).unwrap().clone(), 1.into());
        assert!(locals.get(&Index::Num(1)).is_err());
    }

    #[test]
    fn test_set_wrong_type() {
        let mut locals = super::Locals::new();
        locals.grow(None, 0.into()).unwrap();
        assert!(locals.set(&Index::Num(0), 1i64.into()).is_err());
    }

    #[test]
    fn test_grow_get_set_by_id() {
        let mut locals = super::Locals::new();
        locals.grow(Some(String::from("a")), 0.into()).unwrap();
        locals.set(&test_index("a"), 1.into()).unwrap();

        assert_eq!(locals.get(&test_index("a")).unwrap().clone(), 1.into());
        assert!(locals.get(&test_index("b")).is_err());
    }

    #[test]
    fn test_grow_get_set_by_index() {
        let mut locals = super::Locals::new();
        locals.grow(Some(String::from("a")), 0.into()).unwrap();
        locals.set(&Index::Id(String::from("a")), 1.into()).unwrap();

        assert_eq!(locals.get(&test_index("a")).unwrap().clone(), 1.into());
        assert!(locals.get(&Index::Num(1)).is_err());
    }

    #[test]
    fn test_set_wrong_type_by_id() {
        let mut locals = super::Locals::new();
        locals.grow(Some(String::from("a")), 0.into()).unwrap();
        assert!(locals.set(&test_index("a"), 1i64.into()).is_err());
    }

    #[test]
    fn test_commit_rollback() {
        let mut locals = super::Locals::new();
        locals.grow(None, 0.into()).unwrap();
        locals.set(&Index::Num(0), 1.into()).unwrap();
        locals.commit();
        locals.set(&Index::Num(0), 2.into()).unwrap();
        locals.rollback();
        assert_eq!(locals.get(&Index::Num(0)).unwrap().clone(), 1.into());
    }
}
