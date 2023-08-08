use anyhow::{Error, Ok, Result};

/// Stack with commit and rollback in constant time.
pub struct Stack {
    values: Vec<i32>,
    shrink_by: usize,
    soft_values: Vec<i32>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            values: vec![],
            shrink_by: 0,
            soft_values: vec![],
        }
    }

    pub fn push(&mut self, value: i32) {
        self.soft_values.push(value);
    }

    pub fn pop(&mut self) -> Result<i32> {
        if self.soft_values.len() == 0 {
            self.shrink_by += 1;
            self.check_underflow()?;
            let idx = self.values.len() - self.shrink_by;
            Ok(self.values.get(idx).unwrap().clone())
        } else {
            Ok(self.soft_values.pop().unwrap())
        }
    }

    fn check_underflow(&self) -> Result<()> {
        if self.values.len() < self.shrink_by {
            return Err(Error::msg("Stack underflow"));
        }
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        self.check_underflow()?;

        self.values.truncate(self.values.len() - self.shrink_by);
        self.values.append(&mut self.soft_values);
        self.shrink_by = 0;
        self.soft_values.clear();

        Ok(())
    }

    pub fn rollback(&mut self) {
        self.shrink_by = 0;
        self.soft_values.clear();
    }

    #[allow(dead_code)]
    pub fn to_soft_string(&self) -> Result<String> {
        self.check_underflow()?;

        let mut values = self.values.clone();
        values.truncate(values.len() - self.shrink_by);
        values.extend(self.soft_values.clone());
        Ok(format!("{:?}", values))
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.values)
    }
}

#[cfg(test)]
mod tests {
    use crate::stack::Stack;

    #[test]
    fn test_stack() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        assert_eq!(stack.pop().unwrap(), 2);
        assert_eq!(stack.pop().unwrap(), 1);
        assert!(stack.pop().is_err());
        assert!(stack.to_soft_string().is_err());
    }

    #[test]
    fn test_stack_commit() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.commit().unwrap();
        assert_eq!(stack.pop().unwrap(), 2);
        assert_eq!(stack.pop().unwrap(), 1);
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_rollback() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.rollback();
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_grow_and_rollback() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        stack.commit().unwrap();
        stack.push(4);
        stack.push(5);
        stack.rollback();
        assert_eq!(stack.pop().unwrap(), 3);
        assert_eq!(stack.pop().unwrap(), 2);
        assert_eq!(stack.pop().unwrap(), 1);
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_grow_and_commit() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        stack.commit().unwrap();

        stack.push(4);
        stack.push(5);
        stack.commit().unwrap();
        assert_eq!(stack.pop().unwrap(), 5);
        assert_eq!(stack.pop().unwrap(), 4);
        assert_eq!(stack.pop().unwrap(), 3);
        assert_eq!(stack.pop().unwrap(), 2);
        assert_eq!(stack.pop().unwrap(), 1);
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_shrink_and_rollback() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        stack.commit().unwrap();

        stack.pop().unwrap();
        stack.pop().unwrap();
        stack.rollback();

        assert_eq!(stack.pop().unwrap(), 3);
        assert_eq!(stack.pop().unwrap(), 2);
        assert_eq!(stack.pop().unwrap(), 1);
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_shrink_and_commit() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        stack.commit().unwrap();

        stack.pop().unwrap();
        stack.pop().unwrap();
        stack.commit().unwrap();

        assert_eq!(stack.pop().unwrap(), 1);
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_underflow_and_commit() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.commit().unwrap();

        stack.pop().unwrap();
        stack.pop().unwrap();
        assert!(stack.pop().is_err());

        assert!(stack.commit().is_err());
    }

    #[test]
    fn test_stack_to_string() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.commit().unwrap();
        assert_eq!(stack.to_string(), "[1, 2]");
    }

    #[test]
    fn test_stack_uncommited_to_string() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.commit().unwrap();
        stack.push(3);
        assert_eq!(stack.to_string(), "[1, 2]");
        assert_eq!(stack.to_soft_string().unwrap(), "[1, 2, 3]");
    }
}
