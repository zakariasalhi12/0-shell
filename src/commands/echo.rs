pub struct Echo {
    args: Vec<String>,
}

impl Echo {

    pub fn new(args : Vec<String>) -> Self {
        Echo{args: args}
    }

    pub fn execute(&self) {
        let text  = self.args.join(" ");
        println!("{text}");
    }
}
