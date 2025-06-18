#[derive(Debug, PartialEq, Eq)]
pub struct Pwd {
    pub args : Vec<String>
}

impl Pwd {
    pub fn new(&self , args : Vec<String>) -> Self {
        Pwd { args: args }
    }
}