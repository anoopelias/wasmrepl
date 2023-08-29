use crate::{dict::Dict, list::List, utils::IsSame};
use anyhow::Result;

pub struct Elements<T: IsSame> {
    values: List<T>,
    ids: Dict<usize>,
}

impl<T: IsSame> Elements<T> {
    pub fn new() -> Elements<T> {
        Elements {
            values: List::new(),
            ids: Dict::new(),
        }
    }

    pub fn grow(&mut self, value: T) -> usize {
        self.values.grow(value)
    }

    pub fn grow_by_id(&mut self, id: &str, value: T) -> Result<()> {
        // TODO: Check if id already exists
        let index = self.values.grow(value);
        self.ids.set(id, index);
        Ok(())
    }

    pub fn set(&mut self, index: usize, value: T) -> Result<()> {
        self.get(index)?.is_same(&value)?;
        self.values.set(index, value)
    }

    pub fn set_by_id(&mut self, id: &str, value: T) -> Result<()> {
        let index = self.ids.get(id)?;
        self.set(index, value)
    }

    pub fn get(&self, index: usize) -> Result<&T> {
        self.values.get(index)
    }

    pub fn get_by_id(&self, id: &str) -> Result<&T> {
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
    use crate::{
        elements::{Elements, IsSame},
        test_utils::test_val_i32,
    };

    fn elements_get_by_id<T: IsSame + Clone>(elements: &Elements<T>, id: &str) -> T {
        elements.get_by_id(id).unwrap().clone()
    }

    fn elements_get<T: IsSame + Clone>(elements: &Elements<T>, index: usize) -> T {
        elements.get(index).unwrap().clone()
    }

    #[test]
    fn test_elements_set_get() {
        let mut elements = Elements::new();
        elements.grow(test_val_i32(0));
        elements.grow(test_val_i32(0));
        elements.set(0, 1.into()).unwrap();
        elements.set(1, 2.into()).unwrap();

        assert_eq!(elements_get(&elements, 0), 1.into());
        assert_eq!(elements_get(&elements, 1), 2.into());
    }

    #[test]
    fn test_elements_get() {
        let mut elements = Elements::new();
        elements.grow(test_val_i32(0));
        assert_eq!(elements_get(&elements, 0), 0.into());
    }

    #[test]
    fn test_elements_set_get_by_id() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", test_val_i32(0)).unwrap();
        elements.grow_by_id("b", test_val_i32(0)).unwrap();
        elements.set_by_id("a", 1.into()).unwrap();
        elements.set_by_id("b", 2.into()).unwrap();

        assert_eq!(elements_get_by_id(&elements, "a"), 1.into());
        assert_eq!(elements_get_by_id(&elements, "b"), 2.into());
    }

    #[test]
    fn test_elements_gid_set_get() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", test_val_i32(0)).unwrap();
        elements.grow_by_id("b", test_val_i32(0)).unwrap();
        elements.set(0, 1.into()).unwrap();
        elements.set(1, 2.into()).unwrap();

        assert_eq!(elements_get(&elements, 0), 1.into());
        assert_eq!(elements_get(&elements, 1), 2.into());
    }

    #[test]
    fn test_elements_get_error() {
        let mut elements = Elements::new();
        elements.grow(test_val_i32(0));
        elements.set(0, 1.into()).unwrap();

        assert!(elements.get(1).is_err());
    }

    #[test]
    fn test_elements_set_error() {
        let mut elements = Elements::new();
        elements.grow(test_val_i32(0));
        elements.set(0, 1.into()).unwrap();

        assert!(elements.set(1, 2.into()).is_err());
    }

    #[test]
    fn test_elements_set_by_id_error() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", test_val_i32(0)).unwrap();
        elements.set_by_id("a", 1.into()).unwrap();

        assert!(elements.set_by_id("b", 2.into()).is_err());
    }

    #[test]
    fn test_elements_get_by_id_error() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", test_val_i32(0)).unwrap();
        elements.set_by_id("a", 1.into()).unwrap();

        assert!(elements.get_by_id("b").is_err());
    }

    #[test]
    fn test_elements_commit() {
        let mut elements = Elements::new();
        elements.grow(test_val_i32(0));
        elements.set(0, 1.into()).unwrap();
        elements.commit();

        elements.grow(test_val_i32(0));
        elements.set(0, 2.into()).unwrap();
        elements.set(1, 4.into()).unwrap();
        elements.commit();

        assert_eq!(elements_get(&elements, 0), 2.into());
        assert_eq!(elements_get(&elements, 1), 4.into());
        assert!(elements.get(2).is_err());
    }

    #[test]
    fn test_elements_commit_rollback() {
        let mut elements = Elements::new();
        elements.grow(test_val_i32(0));
        elements.grow(test_val_i32(0));
        elements.set(0, 1.into()).unwrap();
        elements.set(1, 2.into()).unwrap();
        elements.commit();

        elements.grow(test_val_i32(0));
        elements.set(0, 3.into()).unwrap();
        elements.set(2, 4.into()).unwrap();
        elements.rollback();

        assert_eq!(elements_get(&elements, 0), 1.into());
        assert_eq!(elements_get(&elements, 1), 2.into());
        assert!(elements.get(2).is_err());
    }

    #[test]
    fn test_elements_commit_rollback_id() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", test_val_i32(0)).unwrap();
        elements.grow_by_id("b", test_val_i32(0)).unwrap();
        elements.set_by_id("a", 1.into()).unwrap();
        elements.set_by_id("b", 2.into()).unwrap();
        elements.commit();

        elements.grow_by_id("c", test_val_i32(0)).unwrap();
        elements.set_by_id("a", 3.into()).unwrap();
        elements.set_by_id("c", 4.into()).unwrap();
        elements.rollback();

        assert_eq!(elements_get_by_id(&elements, "a"), 1.into());
        assert_eq!(elements_get_by_id(&elements, "b"), 2.into());
        assert!(elements.get_by_id("c").is_err());
    }

    #[test]
    fn test_elements_rollback_recovery() {
        let mut elements = Elements::new();
        elements.grow(test_val_i32(0));
        elements.set(0, 1.into()).unwrap();
        elements.commit();

        elements.grow(test_val_i32(0));
        elements.set(1, 2.into()).unwrap();
        elements.rollback();

        elements.grow(test_val_i32(0));
        elements.set(0, 3.into()).unwrap();
        assert_eq!(elements_get(&elements, 0), 3.into());
        assert_eq!(elements_get(&elements, 1), 0.into());
        assert!(elements.set(2, 4.into()).is_err());
    }

    #[test]
    fn test_elements_rollback_recovery_id() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", test_val_i32(0)).unwrap();
        elements.set_by_id("a", 1.into()).unwrap();
        elements.commit();

        elements.grow_by_id("b", test_val_i32(0)).unwrap();
        elements.set_by_id("b", 2.into()).unwrap();
        elements.rollback();

        elements.grow_by_id("c", test_val_i32(0)).unwrap();
        elements.set_by_id("a", 3.into()).unwrap();
        assert_eq!(elements_get_by_id(&elements, "a"), 3.into());
        assert_eq!(elements_get_by_id(&elements, "c"), 0.into());
        assert!(elements.set_by_id("b", 4.into()).is_err());
    }
}
