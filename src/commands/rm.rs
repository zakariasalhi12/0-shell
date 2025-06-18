#[derive(Debug, PartialEq, Eq)]
pub struct Rm {
    pub args : Vec<String>
}

impl Rm {
    pub fn new(&self , args : Vec<String>) -> Self {
        Rm { args: args }
    }
}