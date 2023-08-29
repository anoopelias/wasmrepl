use anyhow::Result;
pub trait IsSame {
    fn is_same(&self, other: &Self) -> Result<()>;
}
