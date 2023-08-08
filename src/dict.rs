#![allow(unused)]
use anyhow::{Error, Result};
use std::collections::HashMap;

/// This dict is essentially a HashMap on which the changes can be
/// commited or rolled back.

pub struct Dict<T: Copy> {
    values: HashMap<String, T>,
    soft_values: HashMap<String, T>,
}

impl<T: Copy> Dict<T> {
    pub fn new() -> Dict<T> {
        Dict {
            values: HashMap::new(),
            soft_values: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: T) {
        self.soft_values.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Result<T> {
        match self.soft_values.get(key) {
            Some(value) => Ok(*value),
            None => match self.values.get(key) {
                Some(value) => Ok(*value),
                None => Err(Error::msg(format!("Key not found: {}", key))),
            },
        }
    }

    /// Commit is transferring the ownership of values in `self.soft_values` to
    /// `self.values`. Hence we need ownership of self.
    /// TODO: What is idiomatic way of doing this?
    pub fn commit(mut self) -> Self {
        self.values.extend(self.soft_values);
        self.soft_values = HashMap::new();
        self
    }

    pub fn rollback(&mut self) {
        self.soft_values.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::dict::Dict;

    #[test]
    fn test_dict() {
        let mut dict = Dict::new();
        dict.set("a", 1);
        dict.set("b", 2);
        dict.set("c", 3);
        dict = dict.commit();
        assert_eq!(dict.get("a").unwrap(), 1);
        assert_eq!(dict.get("b").unwrap(), 2);
        assert_eq!(dict.get("c").unwrap(), 3);
    }

    #[test]
    fn test_commit() {
        let mut dict = Dict::new();
        dict.set("a", 1);
        dict.set("b", 2);
        dict = dict.commit();

        dict.set("c", 3);
        dict = dict.commit();
        assert_eq!(dict.get("a").unwrap(), 1);
        assert_eq!(dict.get("b").unwrap(), 2);
    }

    #[test]
    fn test_rollback() {
        let mut dict = Dict::new();
        dict.set("a", 1);
        dict.set("b", 2);
        dict = dict.commit();

        dict.set("c", 3);
        dict.rollback();
        assert_eq!(dict.get("a").unwrap(), 1);
        assert_eq!(dict.get("b").unwrap(), 2);
        assert!(dict.get("c").is_err());
    }
}
