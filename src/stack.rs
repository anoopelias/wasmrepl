use anyhow::{Error, Ok, Result};

use crate::value::Value;

/// Stack with commit and rollback in constant time.
pub struct Stack {
    values: Vec<Value>,
    shrink_by: usize,
    soft_values: Vec<Value>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            values: vec![],
            shrink_by: 0,
            soft_values: vec![],
        }
    }

    pub fn push(&mut self, value: Value) {
        self.soft_values.push(value);
    }

    pub fn pop(&mut self) -> Result<Value> {
        if self.soft_values.len() == 0 {
            self.shrink_by += 1;
            self.check_underflow()?;
            let idx = self.values.len() - self.shrink_by;

            // We remove the value from the stack only when we commit.
            // Hence we can't handover the ownership of the popped item
            // just yet.
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

        let mut strs = vec![];

        let mut i = 0;
        while i < self.values.len() - self.shrink_by {
            strs.push(self.values[i].to_string());
            i += 1;
        }

        for value in self.soft_values.iter() {
            strs.push(value.to_string());
        }

        Ok(format!("[{}]", strs.join(", ")))
    }

    pub fn to_string(&self) -> String {
        let strs: Vec<String> = self.values.iter().map(|v| v.to_string()).collect();
        format!("[{}]", strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use crate::stack::Stack;
    use crate::value::test_utils::test_val_i32;

    #[test]
    fn test_stack() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        assert_eq!(stack.pop().unwrap(), test_val_i32(2));
        assert_eq!(stack.pop().unwrap(), test_val_i32(1));
        assert!(stack.pop().is_err());
        assert!(stack.to_soft_string().is_err());
    }

    #[test]
    fn test_stack_commit() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.commit().unwrap();
        assert_eq!(stack.pop().unwrap(), test_val_i32(2));
        assert_eq!(stack.pop().unwrap(), test_val_i32(1));
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_rollback() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.rollback();
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_grow_and_rollback() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.push(test_val_i32(3));
        stack.commit().unwrap();
        stack.push(test_val_i32(4));
        stack.push(test_val_i32(5));
        stack.rollback();
        assert_eq!(stack.pop().unwrap(), test_val_i32(3));
        assert_eq!(stack.pop().unwrap(), test_val_i32(2));
        assert_eq!(stack.pop().unwrap(), test_val_i32(1));
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_grow_and_commit() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.push(test_val_i32(3));
        stack.commit().unwrap();

        stack.push(test_val_i32(4));
        stack.push(test_val_i32(5));
        stack.commit().unwrap();
        assert_eq!(stack.pop().unwrap(), test_val_i32(5));
        assert_eq!(stack.pop().unwrap(), test_val_i32(4));
        assert_eq!(stack.pop().unwrap(), test_val_i32(3));
        assert_eq!(stack.pop().unwrap(), test_val_i32(2));
        assert_eq!(stack.pop().unwrap(), test_val_i32(1));
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_shrink_and_rollback() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.push(test_val_i32(3));
        stack.commit().unwrap();

        stack.pop().unwrap();
        stack.pop().unwrap();
        stack.rollback();

        assert_eq!(stack.pop().unwrap(), test_val_i32(3));
        assert_eq!(stack.pop().unwrap(), test_val_i32(2));
        assert_eq!(stack.pop().unwrap(), test_val_i32(1));
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_shrink_and_commit() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.push(test_val_i32(3));
        stack.commit().unwrap();

        stack.pop().unwrap();
        stack.pop().unwrap();
        stack.commit().unwrap();

        assert_eq!(stack.pop().unwrap(), test_val_i32(1));
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_stack_underflow_and_commit() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.commit().unwrap();

        stack.pop().unwrap();
        stack.pop().unwrap();
        assert!(stack.pop().is_err());

        assert!(stack.commit().is_err());
    }

    #[test]
    fn test_stack_to_string() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.commit().unwrap();
        assert_eq!(stack.to_string(), "[1, 2]");
    }

    #[test]
    fn test_stack_uncommited_to_string() {
        let mut stack = Stack::new();
        stack.push(test_val_i32(1));
        stack.push(test_val_i32(2));
        stack.commit().unwrap();
        stack.push(test_val_i32(3));
        assert_eq!(stack.to_string(), "[1, 2]");
        assert_eq!(stack.to_soft_string().unwrap(), "[1, 2, 3]");
    }
}
