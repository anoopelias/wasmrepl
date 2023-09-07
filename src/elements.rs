use crate::{dict::Dict, list::List, model::Index};
use anyhow::Result;

pub struct Elements<T> {
    values: List<T>,
    ids: Dict<usize>,
}

impl<T> Elements<T> {
    pub fn new() -> Elements<T> {
        Elements {
            values: List::new(),
            ids: Dict::new(),
        }
    }

    pub fn grow(&mut self, value: T) -> usize {
        self.values.grow(value)
    }

    // TODO: We can get away with only one parameter here.
    pub fn grow_by_id(&mut self, id: &str, value: T) -> Result<()> {
        // TODO: Check if id already exists
        let index = self.values.grow(value);
        self.ids.set(id, index);
        Ok(())
    }

    fn set_by_num(&mut self, index: usize, value: T) -> Result<()> {
        self.values.set(index, value)
    }

    fn set_by_id(&mut self, id: &str, value: T) -> Result<()> {
        let index = self.ids.get(id)?;
        self.set_by_num(index, value)
    }

    pub fn set(&mut self, index: &Index, value: T) -> Result<()> {
        match index {
            Index::Id(id) => self.set_by_id(id, value),
            Index::Num(index) => self.set_by_num(*index as usize, value),
        }
    }

    fn get_by_num(&self, index: usize) -> Result<&T> {
        self.values.get(index)
    }

    fn get_by_id(&self, id: &str) -> Result<&T> {
        let index = self.ids.get(id)?;
        self.get_by_num(index)
    }

    pub fn get(&self, index: &Index) -> Result<&T> {
        match index {
            Index::Id(id) => self.get_by_id(id),
            Index::Num(index) => self.get_by_num(*index as usize),
        }
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
    use crate::elements::Elements;

    fn elements_get_by_id<T: Clone>(elements: &Elements<T>, id: &str) -> T {
        elements.get_by_id(id).unwrap().clone()
    }

    fn elements_get<T: Clone>(elements: &Elements<T>, index: usize) -> T {
        elements.get_by_num(index).unwrap().clone()
    }

    #[test]
    fn test_elements_grow_set_get() {
        let mut elements = Elements::new();
        elements.grow(0);
        elements.set_by_num(0, 1).unwrap();
        assert_eq!(elements_get(&elements, 0), 1);
    }

    #[test]
    fn test_elements_set_get_by_id() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", 0).unwrap();
        elements.set_by_id("a", 1).unwrap();
        assert_eq!(elements_get_by_id(&elements, "a"), 1);
    }

    #[test]
    fn test_elements_gid_set_get() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", 0).unwrap();
        elements.set_by_num(0, 1).unwrap();

        assert_eq!(elements_get(&elements, 0), 1);
    }

    #[test]
    fn test_elements_set_by_id_error() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", 0).unwrap();
        elements.set_by_id("a", 1).unwrap();

        assert!(elements.set_by_id("b", 2).is_err());
    }

    #[test]
    fn test_elements_get_by_id_error() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", 0).unwrap();
        elements.set_by_id("a", 1).unwrap();

        assert!(elements.get_by_id("b").is_err());
    }

    #[test]
    fn test_elements_commit() {
        let mut elements = Elements::new();
        elements.grow(0);
        elements.set_by_num(0, 1).unwrap();
        elements.commit();

        elements.grow(0);
        elements.set_by_num(0, 2).unwrap();
        elements.set_by_num(1, 4).unwrap();
        elements.commit();

        assert_eq!(elements_get(&elements, 0), 2);
        assert_eq!(elements_get(&elements, 1), 4);
        assert!(elements.get_by_num(2).is_err());
    }

    #[test]
    fn test_elements_commit_rollback() {
        let mut elements = Elements::new();
        elements.grow(0);
        elements.grow(0);
        elements.set_by_num(0, 1).unwrap();
        elements.set_by_num(1, 2).unwrap();
        elements.commit();

        elements.grow(0);
        elements.set_by_num(0, 3).unwrap();
        elements.set_by_num(2, 4).unwrap();
        elements.rollback();

        assert_eq!(elements_get(&elements, 0), 1);
        assert_eq!(elements_get(&elements, 1), 2);
        assert!(elements.get_by_num(2).is_err());
    }

    #[test]
    fn test_elements_commit_rollback_id() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", 0).unwrap();
        elements.grow_by_id("b", 0).unwrap();
        elements.set_by_id("a", 1).unwrap();
        elements.set_by_id("b", 2).unwrap();
        elements.commit();

        elements.grow_by_id("c", 0).unwrap();
        elements.set_by_id("a", 3).unwrap();
        elements.set_by_id("c", 4).unwrap();
        elements.rollback();

        assert_eq!(elements_get_by_id(&elements, "a"), 1);
        assert_eq!(elements_get_by_id(&elements, "b"), 2);
        assert!(elements.get_by_id("c").is_err());
    }

    #[test]
    fn test_elements_rollback_recovery() {
        let mut elements = Elements::new();
        elements.grow(0);
        elements.set_by_num(0, 1).unwrap();
        elements.commit();

        elements.grow(0);
        elements.set_by_num(1, 2).unwrap();
        elements.rollback();

        elements.grow(0);
        elements.set_by_num(0, 3).unwrap();
        assert_eq!(elements_get(&elements, 0), 3);
        assert_eq!(elements_get(&elements, 1), 0);
    }

    #[test]
    fn test_elements_rollback_recovery_id() {
        let mut elements = Elements::new();
        elements.grow_by_id("a", 0).unwrap();
        elements.set_by_id("a", 1).unwrap();
        elements.commit();

        elements.grow_by_id("b", 0).unwrap();
        elements.set_by_id("b", 2).unwrap();
        elements.rollback();

        elements.grow_by_id("c", 0).unwrap();
        elements.set_by_id("a", 3).unwrap();
        assert_eq!(elements_get_by_id(&elements, "a"), 3);
        assert_eq!(elements_get_by_id(&elements, "c"), 0);
    }

    #[test]
    fn test_elements_get_by_index() {}
}
