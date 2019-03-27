extern crate clap;
use clap::{App, Arg, SubCommand};

extern crate rustdag_lib;

mod server;

fn main() {
    let matches = App::new("RustDAG CLI")
        .version("0.1")
        .author("Colin Moore <colin@moore.one>")
        .about("Command line function for RustDAG")
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .help("Set server address")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("deploy")
                .version("0.1")
                .author("Colin Moore <colin@moore.one>")
                .about("Deploy Smart Contracts")
                .arg(
                    Arg::with_name("INPUT")
                        .help("Sets the input file to use")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("run")
                .version("0.1")
                .author("Colin Moore <colin@moore.one>")
                .about("Run Smart Contract Function")
                .arg(
                    Arg::with_name("contract")
                        .short("c")
                        .long("contract")
                        .help("Set contract address")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("FUNCTION")
                        .help("Function name to call")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("FN_ARGS")
                        .help("Arguments to contract function")
                        .min_values(1)
                        .index(2),
                ),
        )
        .get_matches();

    let server = server::Server::new(
        matches
            .value_of("server")
            .unwrap_or("http://localhost:4200"),
    );

    if let Some(matches) = matches.subcommand_matches("deploy") {
        let filename = matches.value_of("INPUT").unwrap();
        let contract_id = server.deploy_contract(filename);
        println!("Contract ID: {}", contract_id);
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let contract_id = matches
            .value_of("contract")
            .unwrap()
            .parse::<u64>()
            .expect("Contract must be a valid integer");
        let function_name = matches.value_of("FUNCTION").unwrap();
        let args: Vec<_> = matches.values_of("FN_ARGS").unwrap().collect();
        server.run_contract(
            contract_id,
            function_name.into(),
            args.into_iter().map(|x| x.into()).collect::<Vec<String>>(),
        );
    }
}
