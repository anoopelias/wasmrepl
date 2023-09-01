use crate::{elements::Elements, value::Value};
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

    pub fn grow(&mut self, value: Value) -> usize {
        self.elements.grow(value)
    }

    pub fn grow_by_id(&mut self, id: &str, value: Value) -> Result<()> {
        self.elements.grow_by_id(id, value)
    }

    pub fn set(&mut self, index: usize, value: Value) -> Result<()> {
        self.get(index)?.is_same(&value)?;
        self.elements.set(index, value)
    }

    pub fn set_by_id(&mut self, id: &str, value: Value) -> Result<()> {
        self.get_by_id(id)?.is_same(&value)?;
        self.elements.set_by_id(id, value)
    }

    pub fn get(&self, index: usize) -> Result<&Value, anyhow::Error> {
        self.elements.get(index)
    }

    pub fn get_by_id(&self, id: &str) -> Result<&Value, anyhow::Error> {
        self.elements.get_by_id(id)
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

    #[test]
    fn test_grow_get_set() {
        let mut locals = super::Locals::new();
        assert_eq!(locals.grow(0.into()), 0);
        locals.set(0, 1.into()).unwrap();

        assert_eq!(locals.get(0).unwrap().clone(), 1.into());
        assert!(locals.get(1).is_err());
    }

    #[test]
    fn test_set_wrong_type() {
        let mut locals = super::Locals::new();
        locals.grow(0.into());
        assert!(locals.set(0, 1i64.into()).is_err());
    }

    #[test]
    fn test_grow_get_set_by_id() {
        let mut locals = super::Locals::new();
        locals.grow_by_id("a", 0.into()).unwrap();
        locals.set_by_id("a", 1.into()).unwrap();

        assert_eq!(locals.get_by_id("a").unwrap().clone(), 1.into());
        assert!(locals.get_by_id("b").is_err());
    }

    #[test]
    fn test_set_wrong_type_by_id() {
        let mut locals = super::Locals::new();
        locals.grow_by_id("a", 0.into()).unwrap();
        assert!(locals.set_by_id("a", 1i64.into()).is_err());
    }

    #[test]
    fn test_commit_rollback() {
        let mut locals = super::Locals::new();
        locals.grow(0.into());
        locals.set(0, 1.into()).unwrap();
        locals.commit();
        locals.set(0, 2.into()).unwrap();
        locals.rollback();
        assert_eq!(locals.get(0).unwrap().clone(), 1.into());
    }
}
