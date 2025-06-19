use crate::{Commande, parse};

pub fn Execute(commandes: Vec<Commande>) {
    for command in commandes {
        let com = command.Name.build_command(command.Args, command.Option);
        match com {
            Some(val) => {
                let res = val.execute();
                match res {
                    Ok(value) => println!("{:?}", value),
                    Err(e) => panic!("{e}"),
                }
            }
            None => panic!(),
        }
    }
}
