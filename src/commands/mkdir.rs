#[derive(Debug, PartialEq, Eq)]
pub struct Ls {
    pub args : Vec<String>
}

impl Ls {
    pub fn new(&self , args : Vec<String>) -> Self {
        Ls { args: args }
    }
}