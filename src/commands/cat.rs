#[derive(Debug, PartialEq, Eq)]
pub struct Cat {
    pub args : Vec<String>
}

impl Cat {
    pub fn new(&self , args : Vec<String>) -> Self {
        Cat { args: args }
    }
}