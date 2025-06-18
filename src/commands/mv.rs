#[derive(Debug, PartialEq, Eq)]
pub struct Mv {
    pub args: Vec<String>,
}

impl Mv {
    pub fn new(&self, args: Vec<String>) -> Self {
        Mv { args: args }
    }
}
