#[derive(Debug, PartialEq, Eq)]
pub struct Cd {
    pub args : Vec<String>
}

impl Cd {
    pub fn new(&self , args : Vec<String>) -> Self {
        Cd { args: args }
    }
}