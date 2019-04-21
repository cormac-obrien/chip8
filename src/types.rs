use std::ops::Deref;

// TODO: un-pub the newtype interiors

#[derive(Debug, Copy, Clone)]
pub struct Addr(pub u16);

#[derive(Debug, Copy, Clone)]
pub struct RegId(pub u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Val(pub u8);

impl Deref for Val {
    type Target = u8;

    fn deref(&self) -> &u8 {
        &self.0
    }
}
