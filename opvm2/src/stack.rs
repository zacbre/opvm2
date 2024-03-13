use extism_pdk::{FromBytes, Json};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, FromBytes)]
#[encoding(Json)]
pub struct Stack<T>(Vec<T>);

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.last()
    }

    pub fn to_vec(&self) -> &Vec<T> {
        &self.0
    }
}
