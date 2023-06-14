/// This is a term project for the Programming in Rust class at Portland State University, Spring 2023
/// Author: Christian Torralba
/// Teacher: Bart Massey
/// Version: 1.0
/// This program is used to view some basic statistics for postgres databases. That including the version,
/// uptime, public tables, installed extensions, and also includes a feature for custom querying. <br>
/// The entire point of this program is to make it a little easier to quickly connect to a database to see statistics.
/// There is also JSON integration, where previously created database connections can be stored in a file for later use.
/// Passwords are _NOT_ stored. <br>
///
/// I made this program because I enjoy working with databases, but I also occasionally wish that it would be a little
/// easier to connect to them to quickly get information.
///
/// This program makes use of multiple crates, including:
///     - [Clap](https://crates.io/crates/clap)
///         - This provides easy command line argument parsing, and makes life a lot easier!
///     - [Postgres](https://docs.rs/postgres/latest/postgres/)
///         - This is the backbone of the project. Provides all the necessary functions and types to connection to a postgres database
///     - [Colored](https://crates.io/crates/colored)
///         - For some pretty printing to the console.
///     - [serde_json](https://docs.rs/serde_json/latest/serde_json/)
///         - For our JSON integration
use clap::Parser;
use colored::Colorize;
use std::io;
use std::io::Write;
use std::time::Duration;

mod psql_stats;

use psql_stats::help_menu;
use psql_stats::welcome;
use psql_stats::Args;
use psql_stats::Connection;

fn main() {
    let args = Args::parse();

    let loaded_connection: Option<String> = args.load;

    let mut connection: Connection = Connection {
        client: None,
        host: "".to_string(),
        dbname: "".to_string(),
        user: "".to_string(),
        port: "".to_string(),
        password: "".to_string(),
    };

    if let Some(..) = loaded_connection {
        if let Some(..)  = args.password {
            connection = match connection.read_from_json(loaded_connection.unwrap(),args.password.as_ref().unwrap().to_string()) {
                Ok(mut c) => {
                    println!("Connection found, loading information.");
                    c.password = match args.password {
                        Some(s) => {
                            println!("Password provided.");
                            s
                        }
                        None => "".to_string(),
                    };
                    c
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }

        }
        connection.connect();
   } else {
        connection.host = match args.host {
            Some(s) => s,
            None => "localhost".to_string(),
        };

        // If DBName is none, then it will be set to the username, if that is none, then it is set
        // to DB Postgres
        connection.dbname = match args.dbname {
            Some(s) => s,
            None => match args.user {
                Some(ref user) => user.to_string(),
                None => "postgres".to_string(),
            },
        };

        connection.user = match args.user {
            Some(s) => s,
            None => {
                eprintln!("Error: need to specify a username");
                std::process::exit(1)
            }
        };

        connection.port = match args.port {
            Some(s) => s.to_string(),
            None => "5432".to_string(),
        };


        connection.password = match args.password {
            Some(s) => s,
            None => "".to_string(),
        };

        connection.connect();
    }

    welcome();
    help_menu();
    loop {
        print!("Connection status: ");
        match connection.client {
            Some(ref mut c) => match c.is_valid(Duration::new(3, 0)) {
                Ok(_) => {
                    println!("{}", "Connected".green().bold());
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    println!("{}", "Not Connected".red().bold());
                }
            },
            None => {
                println!("{}", "Not Connected".red());
            }
        }

        print!("Please enter an option: ");
        io::stdout().flush().expect("Could not flush");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Could not read input!");

        let command = input.trim();

        match command {
            // Exit program
            "0" => {
                println!("Exiting...");
                break;
            }

            // Save connection
            "1" => {
                println!("Please enter the what you wish to name this connection.");
                let mut conn_name_input: String = String::new();
                io::stdin()
                    .read_line(&mut conn_name_input)
                    .expect("Could not read input");
                let conn_name = conn_name_input.trim();

                match connection.write_to_json(conn_name.to_string()) {
                    Ok(b) => {
                        if b {
                            println!("Successfully saved your connection.")
                        }
                    }

                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }

            "2" => match connection.get_uptime() {
                Ok(_rows) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            },

            // Display current running version of postgres
            "3" => match connection.version() {
                Ok(row) => match row.try_get::<_, String>(0) {
                    Ok(v) => {
                        println!("Current running version: {}", v);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            },

            // Display all public tables
            "4" => match connection.get_all_public_tables() {
                Ok(rows) => {
                    println!("Public Tables: ");
                    for row in rows {
                        if let Ok(s) = row.try_get::<_, String>(0) {
                            println!("\t\u{25C6} {}", s);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            },

            // Display all extensions
            "5" => match connection.get_extensions() {
                Ok(rows) => {
                    println!("Installed extensions:");
                    for row in rows {
                        if let Ok(s) = row.try_get::<_, String>(0) {
                            println!("\t\u{25C6} {}", s);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            },

            "6" => {
                println!("Sorry, the custom query function not been implemented yet!");
            }

            // Attempt to reestablish connection
            "7" => {
                connection.connect();
            }

            // Load a connection
            "8" => {
                let mut connection_name = "".to_string();
                let mut connection_password = "".to_string();
                print!("Connection name: ");
                io::stdout().flush().expect("Could not flush");


                io::stdin()
                    .read_line(&mut connection_name)
                    .expect("Could not read input.");


                print!("Password: ");
                io::stdout().flush().expect("Could not flush");
                io::stdin()
                    .read_line(&mut connection_password)
                    .expect("Could not read input.");

                println!("Entered: {}", connection_name);

                match connection.read_from_json(connection_name.to_string(), connection_password.to_string()) {
                    Ok(_) => {
                        println!("Connection found, please enter the password: ");
                        io::stdin()
                            .read_line(&mut connection_password)
                            .expect("Could not read password");
                        connection.password = connection_password;
                        connection.connect();
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            _ => {
                help_menu();
            }
        }
    }
}
