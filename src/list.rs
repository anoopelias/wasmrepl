use std::collections::HashMap;

use anyhow::{Error, Result};

use crate::value::Value;
pub struct List {
    values: Vec<Value>,
    soft_len: usize,
    soft_values: HashMap<usize, Value>,
}

impl List {
    pub fn new() -> List {
        List {
            values: Vec::new(),
            soft_values: HashMap::new(),
            soft_len: 0,
        }
    }

    pub fn grow(&mut self, value: Value) -> usize {
        self.soft_len += 1;
        let index = self.values.len() + self.soft_len - 1;
        self.soft_values.insert(index, value);
        index
    }

    fn has_index(&self, index: usize) -> Result<()> {
        if index >= self.values.len() + self.soft_len {
            Err(Error::msg(format!("Index out of bounds: {}", index)))
        } else {
            Ok(())
        }
    }

    pub fn set(&mut self, index: usize, value: Value) -> Result<()> {
        self.get(index)?.is_same(&value)?;
        self.soft_values.insert(index, value);
        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<&Value> {
        self.has_index(index)?;
        match self.soft_values.get(&index) {
            Some(value) => Ok(value),
            None => Ok(self.values.get(index).unwrap()),
        }
    }

    pub fn commit(&mut self) {
        for _ in 0..self.soft_len {
            self.values.push(0.into());
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
    use crate::{list::List, value::test_utils::test_val_i32};

    #[test]
    fn test_list_get_set() {
        let mut list = List::new();
        assert_eq!(list.grow(test_val_i32(0)), 0);
        assert_eq!(list.grow(test_val_i32(0)), 1);
        list.set(0, 1.into()).unwrap();
        list.set(1, 2.into()).unwrap();

        assert_eq!(list.get(0).unwrap().clone(), 1.into());
        assert_eq!(list.get(1).unwrap().clone(), 2.into());
        assert!(list.get(2).is_err());
    }

    #[test]
    fn test_list_get_set_commit() {
        let mut list = List::new();
        list.grow(test_val_i32(0));
        list.grow(test_val_i32(0));
        list.set(0, 1.into()).unwrap();
        list.set(1, 2.into()).unwrap();
        list.commit();

        assert_eq!(list.get(0).unwrap().clone(), 1.into());
        assert_eq!(list.get(1).unwrap().clone(), 2.into());
    }

    #[test]
    fn test_list_commit_grow() {
        let mut list = List::new();
        list.grow(test_val_i32(0));
        list.grow(test_val_i32(0));
        list.set(0, 1.into()).unwrap();
        list.set(1, 2.into()).unwrap();
        list.commit();

        assert_eq!(list.grow(test_val_i32(0)), 2);
        assert_eq!(list.get(2).unwrap().clone(), 0.into());
        list.set(2, 3.into()).unwrap();
        list.set(0, 4.into()).unwrap();
        assert_eq!(list.get(2).unwrap().clone(), 3.into());
        assert_eq!(list.get(0).unwrap().clone(), 4.into());
        assert!(list.get(3).is_err());
    }

    #[test]
    fn test_list_commit_rollback() {
        let mut list = List::new();
        list.grow(test_val_i32(0));
        list.grow(test_val_i32(0));
        list.set(0, 1.into()).unwrap();
        list.set(1, 2.into()).unwrap();
        list.commit();

        list.grow(test_val_i32(0));
        list.set(2, 3.into()).unwrap();
        list.set(0, 4.into()).unwrap();
        list.rollback();

        assert_eq!(list.get(0).unwrap().clone(), 1.into());
        assert_eq!(list.get(1).unwrap().clone(), 2.into());
        assert!(list.get(2).is_err());
    }

    #[test]
    fn test_list_rollback_reuse() {
        let mut list = List::new();
        list.grow(test_val_i32(0));
        list.grow(test_val_i32(0));
        list.set(0, 1.into()).unwrap();
        list.set(1, 2.into()).unwrap();
        list.commit();

        list.grow(test_val_i32(0));
        list.set(2, 3.into()).unwrap();
        list.set(0, 4.into()).unwrap();
        list.rollback();

        assert_eq!(list.grow(test_val_i32(0)), 2);
        list.set(2, 5.into()).unwrap();
        list.set(0, 6.into()).unwrap();
        assert_eq!(list.get(2).unwrap().clone(), 5.into());
        assert_eq!(list.get(0).unwrap().clone(), 6.into());
        assert!(list.get(3).is_err());
    }
}
