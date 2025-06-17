pub struct Commande {
    pub Type: String, //if this commande should run async (case of &) or sync (case of && or ; or | )
    pub Name: String, // "ls"
    pub Option: String, //"-f j"
    pub Args: Vec<String>,
}

pub fn parse(input: &str) -> [] {

}

// ls & mkdir && cd  