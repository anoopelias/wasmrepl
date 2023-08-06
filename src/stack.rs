use anyhow::{Ok, Result, Error};

/// Stack with commit and rollback in constant time.
struct Stack {
    values: Vec<i32>,
    shrink_by: usize,
    temp_values: Vec<i32>,
}

impl Stack {
    fn new() -> Stack {
        Stack {
            values: vec![],
            shrink_by: 0,
            temp_values: vec![],
        }
    }

    fn push(&mut self, value: i32) {
        self.temp_values.push(value);
    }

    fn pop(&mut self) -> Result<i32> {
        if self.temp_values.len() == 0 {
           
            self.shrink_by += 1;
            if self.is_underflow() {
                return Err(Error::msg("Stack underflow"));
            }
            let idx = self.values.len() - self.shrink_by;
            Ok(self.values.get(idx).unwrap().clone())
        } else {
            Ok(self.temp_values.pop().unwrap())
        }
    }

    fn is_underflow(&self) -> bool {
        self.values.len() < self.shrink_by
    }

    fn commit(&mut self) -> Result<()> {

        if self.is_underflow() {
            return Err(Error::msg("Stack underflow"));
        }

        self.values.truncate(self.values.len() - self.shrink_by);
        self.values.append(&mut self.temp_values);
        self.shrink_by = 0;
        self.temp_values.clear();

        Ok(())
    }

    fn rollback(&mut self) {
        self.shrink_by = 0;
        self.temp_values.clear();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        assert_eq!(stack.pop().unwrap(), 2);
        assert_eq!(stack.pop().unwrap(), 1);
        assert!(stack.pop().is_err());
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
}