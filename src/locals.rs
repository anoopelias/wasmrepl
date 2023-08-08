#![allow(unused)]
use crate::{dict::Dict, list::List};
use anyhow::{Error, Result};

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

    pub fn grow(&mut self) -> usize {
        self.values.grow()
    }

    pub fn grow_by_id(&mut self, id: &str) -> Result<()> {
        // TODO: Check if id already exists
        let index = self.values.grow();
        self.ids.set(id, index);
        Ok(())
    }

    pub fn set(&mut self, index: usize, value: i32) -> Result<()> {
        self.values.set(index, value)
    }

    pub fn set_by_id(&mut self, id: &str, value: i32) -> Result<()> {
        let index = self.ids.get(id)?;
        self.set(index, value)
    }

    pub fn get(&self, index: usize) -> Result<i32> {
        self.values.get(index)
    }

    pub fn get_by_id(&self, id: &str) -> Result<i32> {
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
    use crate::locals::Locals;

    #[test]
    fn test_locals_set_get() {
        let mut locals = Locals::new();
        locals.grow();
        locals.grow();
        locals.set(0, 1).unwrap();
        locals.set(1, 2).unwrap();

        assert_eq!(locals.get(0).unwrap(), 1);
        assert_eq!(locals.get(1).unwrap(), 2);
    }

    #[test]
    fn test_locals_get() {
        let mut locals = Locals::new();
        locals.grow();
        assert_eq!(locals.get(0).unwrap(), 0);
    }

    #[test]
    fn test_locals_set_get_by_id() {
        let mut locals = Locals::new();
        locals.grow_by_id("a").unwrap();
        locals.grow_by_id("b").unwrap();
        locals.set_by_id("a", 1).unwrap();
        locals.set_by_id("b", 2).unwrap();

        assert_eq!(locals.get_by_id("a").unwrap(), 1);
        assert_eq!(locals.get_by_id("b").unwrap(), 2);
    }

    #[test]
    fn test_locals_gid_set_get() {
        let mut locals = Locals::new();
        locals.grow_by_id("a").unwrap();
        locals.grow_by_id("b").unwrap();
        locals.set(0, 1).unwrap();
        locals.set(1, 2).unwrap();

        assert_eq!(locals.get(0).unwrap(), 1);
        assert_eq!(locals.get(1).unwrap(), 2);
    }

    #[test]
    fn test_locals_get_error() {
        let mut locals = Locals::new();
        locals.grow();
        locals.set(0, 1).unwrap();

        assert!(locals.get(1).is_err());
    }

    #[test]
    fn test_locals_set_error() {
        let mut locals = Locals::new();
        locals.grow();
        locals.set(0, 1).unwrap();

        assert!(locals.set(1, 2).is_err());
    }

    #[test]
    fn test_locals_set_by_id_error() {
        let mut locals = Locals::new();
        locals.grow_by_id("a").unwrap();
        locals.set_by_id("a", 1).unwrap();

        assert!(locals.set_by_id("b", 2).is_err());
    }

    #[test]
    fn test_locals_get_by_id_error() {
        let mut locals = Locals::new();
        locals.grow_by_id("a").unwrap();
        locals.set_by_id("a", 1).unwrap();

        assert!(locals.get_by_id("b").is_err());
    }

    #[test]
    fn test_locals_commit() {
        let mut locals = Locals::new();
        locals.grow();
        locals.set(0, 1).unwrap();
        locals.commit();

        locals.grow();
        locals.set(0, 2).unwrap();
        locals.set(1, 4).unwrap();
        locals.commit();

        assert_eq!(locals.get(0).unwrap(), 2);
        assert_eq!(locals.get(1).unwrap(), 4);
        assert!(locals.get(2).is_err());
    }

    #[test]
    fn test_locals_commit_rollback() {
        let mut locals = Locals::new();
        locals.grow();
        locals.grow();
        locals.set(0, 1).unwrap();
        locals.set(1, 2).unwrap();
        locals.commit();

        locals.grow();
        locals.set(0, 3).unwrap();
        locals.set(2, 4).unwrap();
        locals.rollback();

        assert_eq!(locals.get(0).unwrap(), 1);
        assert_eq!(locals.get(1).unwrap(), 2);
        assert!(locals.get(2).is_err());
    }

    #[test]
    fn test_locals_commit_rollback_id() {
        let mut locals = Locals::new();
        locals.grow_by_id("a").unwrap();
        locals.grow_by_id("b").unwrap();
        locals.set_by_id("a", 1).unwrap();
        locals.set_by_id("b", 2).unwrap();
        locals.commit();

        locals.grow_by_id("c").unwrap();
        locals.set_by_id("a", 3).unwrap();
        locals.set_by_id("c", 4).unwrap();
        locals.rollback();

        assert_eq!(locals.get_by_id("a").unwrap(), 1);
        assert_eq!(locals.get_by_id("b").unwrap(), 2);
        assert!(locals.get_by_id("c").is_err());
    }

    #[test]
    fn test_locals_rollback_recovery() {
        let mut locals = Locals::new();
        locals.grow();
        locals.set(0, 1).unwrap();
        locals.commit();

        locals.grow();
        locals.set(1, 2).unwrap();
        locals.rollback();

        locals.grow();
        locals.set(0, 3).unwrap();
        assert_eq!(locals.get(0).unwrap(), 3);
        assert_eq!(locals.get(1).unwrap(), 0);
        assert!(locals.set(2, 4).is_err());
    }

    #[test]
    fn test_locals_rollback_recovery_id() {
        let mut locals = Locals::new();
        locals.grow_by_id("a").unwrap();
        locals.set_by_id("a", 1).unwrap();
        locals.commit();

        locals.grow_by_id("b").unwrap();
        locals.set_by_id("b", 2).unwrap();
        locals.rollback();

        locals.grow_by_id("c").unwrap();
        locals.set_by_id("a", 3).unwrap();
        assert_eq!(locals.get_by_id("a").unwrap(), 3);
        assert_eq!(locals.get_by_id("c").unwrap(), 0);
        assert!(locals.set_by_id("b", 4).is_err());
    }
}
