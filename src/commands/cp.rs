#[derive(Debug, PartialEq, Eq)]
pub struct Cp {
    pub args : Vec<String>
}

impl Cp {
    pub fn new(&self , args : Vec<String>) -> Self {
        Cp { args: args }
    }
}