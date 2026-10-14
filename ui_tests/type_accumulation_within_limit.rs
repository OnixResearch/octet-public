#![warn(type_accumulation)]

pub struct Thing;

impl Thing { pub fn a(&self) {} }
impl Thing { pub fn b(&self) {} }

fn main() {}
