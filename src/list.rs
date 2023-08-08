use std::collections::HashMap;

use anyhow::{Error, Result};
pub struct List {
    values: Vec<i32>,
    soft_len: usize,
    soft_values: HashMap<usize, i32>,
}

impl List {
    pub fn new() -> List {
        List {
            values: Vec::new(),
            soft_values: HashMap::new(),
            soft_len: 0,
        }
    }

    pub fn grow(&mut self) -> usize {
        self.soft_len += 1;
        self.values.len() + self.soft_len - 1
    }

    fn has_index(&self, index: usize) -> Result<()> {
        if index >= self.values.len() + self.soft_len {
            Err(Error::msg(format!("Index out of bounds: {}", index)))
        } else {
            Ok(())
        }
    }

    pub fn set(&mut self, index: usize, value: i32) -> Result<()> {
        self.has_index(index)?;
        self.soft_values.insert(index, value);
        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<i32> {
        self.has_index(index)?;
        match self.soft_values.get(&index) {
            Some(value) => Ok(*value),
            None => Ok(*self.values.get(index).unwrap_or(&0)),
        }
    }

    pub fn commit(&mut self) {
        for _ in 0..self.soft_len {
            self.values.push(0);
        }

        self.soft_values.drain().for_each(|(k, v)| {
            self.values[k] = v;
        });
        self.soft_len = 0;
    }

    pub fn rollback(&mut self) {
        self.soft_values.clear();
        self.soft_len = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::list::List;

    #[test]
    fn test_list_get_set() {
        let mut list = List::new();
        assert_eq!(list.grow(), 0);
        assert_eq!(list.grow(), 1);
        list.set(0, 1).unwrap();
        list.set(1, 2).unwrap();

        assert_eq!(list.get(0).unwrap(), 1);
        assert_eq!(list.get(1).unwrap(), 2);
        assert!(list.get(2).is_err());
    }

    #[test]
    fn test_list_get_set_commit() {
        let mut list = List::new();
        list.grow();
        list.grow();
        list.set(0, 1).unwrap();
        list.set(1, 2).unwrap();
        list.commit();

        assert_eq!(list.get(0).unwrap(), 1);
        assert_eq!(list.get(1).unwrap(), 2);
    }

    #[test]
    fn test_list_commit_grow() {
        let mut list = List::new();
        list.grow();
        list.grow();
        list.set(0, 1).unwrap();
        list.set(1, 2).unwrap();
        list.commit();

        assert_eq!(list.grow(), 2);
        assert_eq!(list.get(2).unwrap(), 0);
        list.set(2, 3).unwrap();
        list.set(0, 4).unwrap();
        assert_eq!(list.get(2).unwrap(), 3);
        assert_eq!(list.get(0).unwrap(), 4);
        assert!(list.get(3).is_err());
    }

    #[test]
    fn test_list_commit_rollback() {
        let mut list = List::new();
        list.grow();
        list.grow();
        list.set(0, 1).unwrap();
        list.set(1, 2).unwrap();
        list.commit();

        list.grow();
        list.set(2, 3).unwrap();
        list.set(0, 4).unwrap();
        list.rollback();

        assert_eq!(list.get(0).unwrap(), 1);
        assert_eq!(list.get(1).unwrap(), 2);
        assert!(list.get(2).is_err());
    }

    #[test]
    fn test_list_rollback_reuse() {
        let mut list = List::new();
        list.grow();
        list.grow();
        list.set(0, 1).unwrap();
        list.set(1, 2).unwrap();
        list.commit();

        list.grow();
        list.set(2, 3).unwrap();
        list.set(0, 4).unwrap();
        list.rollback();

        assert_eq!(list.grow(), 2);
        list.set(2, 5).unwrap();
        list.set(0, 6).unwrap();
        assert_eq!(list.get(2).unwrap(), 5);
        assert_eq!(list.get(0).unwrap(), 6);
        assert!(list.get(3).is_err());
    }
}
