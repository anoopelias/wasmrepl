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

    pub fn set(&mut self, key: String, value: T) {
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

    pub fn commit(&mut self) {
        self.soft_values.drain().for_each(|(k, v)| {
            self.values.insert(k, v);
        });
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
        dict.set(String::from("a"), 1);
        dict.set(String::from("b"), 2);
        dict.set(String::from("c"), 3);
        dict.commit();
        assert_eq!(dict.get("a").unwrap(), 1);
        assert_eq!(dict.get("b").unwrap(), 2);
        assert_eq!(dict.get("c").unwrap(), 3);
    }

    #[test]
    fn test_commit() {
        let mut dict = Dict::new();
        dict.set(String::from("a"), 1);
        dict.set(String::from("b"), 2);
        dict.commit();

        dict.set(String::from("c"), 3);
        dict.commit();
        assert_eq!(dict.get("a").unwrap(), 1);
        assert_eq!(dict.get("b").unwrap(), 2);
    }

    #[test]
    fn test_rollback() {
        let mut dict = Dict::new();
        dict.set(String::from("a"), 1);
        dict.set(String::from("b"), 2);
        dict.commit();

        dict.set(String::from("c"), 3);
        dict.rollback();
        assert_eq!(dict.get("a").unwrap(), 1);
        assert_eq!(dict.get("b").unwrap(), 2);
        assert!(dict.get("c").is_err());
    }
}
