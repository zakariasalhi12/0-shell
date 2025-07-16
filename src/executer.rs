use crate::{Commande};

pub fn execute(commandes: Vec<Commande>) {
    for command in commandes {
        let com = command.name.build_command(command.args, command.option);
        match com {
            Some(val) => {
                let res = val.execute();
                match res {
                    Ok(_) => {}
                    Err(e) => println!("{e}\r"),
                }
            }
            None => panic!(),
        }
    }
}
