use crate::Commande;
use crate::parser::*;

pub fn execute(commandes: Vec<Commande>) {
    for command in commandes {
        match command.operator {
            ExecType::And => {
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
            ExecType::Background => {
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
            ExecType::Or => {
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
            ExecType::Pipe => {
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
            ExecType::Sync => {
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
    }
}
