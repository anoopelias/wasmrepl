use crate::{dict::Dict, list::List, value::Value};
use anyhow::Result;

pub struct Locals {
    values: List,
    ids: Dict<usize>,
}

impl Locals {
    pub fn new() -> Locals {
        Locals {
            values: List::new(),
            ids: Dict::new(),
        }
    }

    pub fn grow(&mut self, value: Value) -> usize {
        self.values.grow(value)
    }

    pub fn grow_by_id(&mut self, id: &str, value: Value) -> Result<()> {
        // TODO: Check if id already exists
        let index = self.values.grow(value);
        self.ids.set(id, index);
        Ok(())
    }

    pub fn set(&mut self, index: usize, value: Value) -> Result<()> {
        self.get(index)?.is_same(&value)?;
        self.values.set(index, value)
    }

    pub fn set_by_id(&mut self, id: &str, value: Value) -> Result<()> {
        let index = self.ids.get(id)?;
        self.set(index, value)
    }

    pub fn get(&self, index: usize) -> Result<&Value> {
        self.values.get(index)
    }

    pub fn get_by_id(&self, id: &str) -> Result<&Value> {
        let index = self.ids.get(id)?;
        self.get(index)
    }

    pub fn commit(&mut self) {
        self.values.commit();
        self.ids.commit();
    }

    pub fn rollback(&mut self) {
        self.values.rollback();
        self.ids.rollback();
    }
}

#[cfg(test)]
mod tests {
    use crate::{locals::Locals, test_utils::test_val_i32, value::Value};

    fn locals_get_by_id(locals: &Locals, id: &str) -> Value {
        locals.get_by_id(id).unwrap().clone()
    }

    fn locals_get(locals: &Locals, index: usize) -> Value {
        locals.get(index).unwrap().clone()
    }

    #[test]
    fn test_locals_set_get() {
        let mut locals = Locals::new();
        locals.grow(test_val_i32(0));
        locals.grow(test_val_i32(0));
        locals.set(0, 1.into()).unwrap();
        locals.set(1, 2.into()).unwrap();

        assert_eq!(locals_get(&locals, 0), 1.into());
        assert_eq!(locals_get(&locals, 1), 2.into());
    }

    #[test]
    fn test_locals_get() {
        let mut locals = Locals::new();
        locals.grow(test_val_i32(0));
        assert_eq!(locals_get(&locals, 0), 0.into());
    }

    #[test]
    fn test_locals_set_get_by_id() {
        let mut locals = Locals::new();
        locals.grow_by_id("a", test_val_i32(0)).unwrap();
        locals.grow_by_id("b", test_val_i32(0)).unwrap();
        locals.set_by_id("a", 1.into()).unwrap();
        locals.set_by_id("b", 2.into()).unwrap();

        assert_eq!(locals_get_by_id(&locals, "a"), 1.into());
        assert_eq!(locals_get_by_id(&locals, "b"), 2.into());
    }

    #[test]
    fn test_locals_gid_set_get() {
        let mut locals = Locals::new();
        locals.grow_by_id("a", test_val_i32(0)).unwrap();
        locals.grow_by_id("b", test_val_i32(0)).unwrap();
        locals.set(0, 1.into()).unwrap();
        locals.set(1, 2.into()).unwrap();

        assert_eq!(locals_get(&locals, 0), 1.into());
        assert_eq!(locals_get(&locals, 1), 2.into());
    }

    #[test]
    fn test_locals_get_error() {
        let mut locals = Locals::new();
        locals.grow(test_val_i32(0));
        locals.set(0, 1.into()).unwrap();

        assert!(locals.get(1).is_err());
    }

    #[test]
    fn test_locals_set_error() {
        let mut locals = Locals::new();
        locals.grow(test_val_i32(0));
        locals.set(0, 1.into()).unwrap();

        assert!(locals.set(1, 2.into()).is_err());
    }

    #[test]
    fn test_locals_set_by_id_error() {
        let mut locals = Locals::new();
        locals.grow_by_id("a", test_val_i32(0)).unwrap();
        locals.set_by_id("a", 1.into()).unwrap();

        assert!(locals.set_by_id("b", 2.into()).is_err());
    }

    #[test]
    fn test_locals_get_by_id_error() {
        let mut locals = Locals::new();
        locals.grow_by_id("a", test_val_i32(0)).unwrap();
        locals.set_by_id("a", 1.into()).unwrap();

        assert!(locals.get_by_id("b").is_err());
    }

    #[test]
    fn test_locals_commit() {
        let mut locals = Locals::new();
        locals.grow(test_val_i32(0));
        locals.set(0, 1.into()).unwrap();
        locals.commit();

        locals.grow(test_val_i32(0));
        locals.set(0, 2.into()).unwrap();
        locals.set(1, 4.into()).unwrap();
        locals.commit();

        assert_eq!(locals_get(&locals, 0), 2.into());
        assert_eq!(locals_get(&locals, 1), 4.into());
        assert!(locals.get(2).is_err());
    }

    #[test]
    fn test_locals_commit_rollback() {
        let mut locals = Locals::new();
        locals.grow(test_val_i32(0));
        locals.grow(test_val_i32(0));
        locals.set(0, 1.into()).unwrap();
        locals.set(1, 2.into()).unwrap();
        locals.commit();

        locals.grow(test_val_i32(0));
        locals.set(0, 3.into()).unwrap();
        locals.set(2, 4.into()).unwrap();
        locals.rollback();

        assert_eq!(locals_get(&locals, 0), 1.into());
        assert_eq!(locals_get(&locals, 1), 2.into());
        assert!(locals.get(2).is_err());
    }

    #[test]
    fn test_locals_commit_rollback_id() {
        let mut locals = Locals::new();
        locals.grow_by_id("a", test_val_i32(0)).unwrap();
        locals.grow_by_id("b", test_val_i32(0)).unwrap();
        locals.set_by_id("a", 1.into()).unwrap();
        locals.set_by_id("b", 2.into()).unwrap();
        locals.commit();

        locals.grow_by_id("c", test_val_i32(0)).unwrap();
        locals.set_by_id("a", 3.into()).unwrap();
        locals.set_by_id("c", 4.into()).unwrap();
        locals.rollback();

        assert_eq!(locals_get_by_id(&locals, "a"), 1.into());
        assert_eq!(locals_get_by_id(&locals, "b"), 2.into());
        assert!(locals.get_by_id("c").is_err());
    }

    #[test]
    fn test_locals_rollback_recovery() {
        let mut locals = Locals::new();
        locals.grow(test_val_i32(0));
        locals.set(0, 1.into()).unwrap();
        locals.commit();

        locals.grow(test_val_i32(0));
        locals.set(1, 2.into()).unwrap();
        locals.rollback();

        locals.grow(test_val_i32(0));
        locals.set(0, 3.into()).unwrap();
        assert_eq!(locals_get(&locals, 0), 3.into());
        assert_eq!(locals_get(&locals, 1), 0.into());
        assert!(locals.set(2, 4.into()).is_err());
    }

    #[test]
    fn test_locals_rollback_recovery_id() {
        let mut locals = Locals::new();
        locals.grow_by_id("a", test_val_i32(0)).unwrap();
        locals.set_by_id("a", 1.into()).unwrap();
        locals.commit();

        locals.grow_by_id("b", test_val_i32(0)).unwrap();
        locals.set_by_id("b", 2.into()).unwrap();
        locals.rollback();

        locals.grow_by_id("c", test_val_i32(0)).unwrap();
        locals.set_by_id("a", 3.into()).unwrap();
        assert_eq!(locals_get_by_id(&locals, "a"), 3.into());
        assert_eq!(locals_get_by_id(&locals, "c"), 0.into());
        assert!(locals.set_by_id("b", 4.into()).is_err());
    }
}
