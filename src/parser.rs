pub struct Commande {
    pub Type: Stdcommands, //if this commande should run async (case of &) or sync (case of && or ; or | )
    pub Name: String, // "ls"
    pub Option: String, //"-f j"
    pub Args: Vec<String>,
}

pub enum Stdcommands{
    echo,
    cd,
    ls,
    pwd,
    cat,
    cp,
    rm,
    mv,
    mkdir,
    exit
}

pub fn parse(input: &str) -> [] {

}

pub fn matcher

// ls & mkdir && cd
