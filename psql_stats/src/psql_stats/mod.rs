use std::fs::{File, OpenOptions};
use std::io::Write;

use clap::Parser;
use postgres::row::Row;
use postgres::{Client, NoTls};
use serde_json::{json, Value};
use thiserror::Error;
use PGError::{DuplicateConnection, JSONOpenFileError, MatchNotFound, QueryError};

/// Wrapper for a postgres error, since we cannot create a "new" postgres::Error
/// This Error type contains the necessary error for this program, including:
/// `QueryError`: If there was an issue running a query on the database. <br>
/// `ConnectionError`: If we were unable to establish a connection to the database. <br>
/// `ClientEmpty`: If the `Client` in our `Connection` struct is none. <br>
/// `JSONOpenFileError`: If we were unable to open the json file. <br>
/// `DuplicateConnection`: If the users connection name already exists in the JSON File <br>
/// `MatchNotFound`: If the user wished to load a previously stored connection and the program was unable to find it.
#[derive(Error, Debug)]
pub enum PGError {
    /// Error for when we cannot communicate with the database, but there is an established connection
    #[error("Issue conversing with the database")]
    QueryError,

    /// Error for if the `client` is `None`
    #[error("Client is not been initialized")]
    ClientEmpty,

    #[error("Could not open JSON file.")]
    JSONOpenFileError,

    #[error("Duplicate connection name.")]
    DuplicateConnection,

    #[error("Match not found")]
    MatchNotFound,
}

/// Arguments for parsing from the command line \
/// Uses the clap crate
/// H - Hostname \
/// U - Username, defaults to "postgres" if no username is provided \
/// p - Port, defaults to 5432 if no port is provided \
/// W - Password
#[derive(Parser, Debug)]
pub(crate) struct Args {
    /// Postgres Database Hostname
    #[arg(short = 'H', long)]
    pub(crate) host: Option<String>,

    /// Postgres Database Username, will default to "postgres" if no username is provided
    #[arg(short = 'U', long, default_value = Some("postgres"))]
    pub(crate) user: Option<String>,

    #[arg(short = 'd', long)]
    pub(crate) dbname: Option<String>,

    /// Postgres Database Port, will default to 5432 if no port is provided
    #[arg(short = 'p', long)]
    pub(crate) port: Option<i16>,

    /// Postgres Database Password
    #[arg(short = 'W', long)]
    pub(crate) password: Option<String>,

    /// Name of previously saved connection
    #[arg(short = 'l')]
    pub(crate) load: Option<String>,
}

/// Connection struct containing necessary information to connect to a Postgres Database <br>
/// We are using a `Option<Client>` for the client since there may not always be an established connection.
///
#[derive(Default)]
pub struct Connection {
    pub(crate) client: Option<Client>,
    pub(crate) host: String,
    pub(crate) dbname: String,
    pub(crate) user: String,
    pub(crate) port: String,
    pub(crate) password: String,
}

impl Connection {
    ///
    pub fn new(host: String, dbname: String, uname: String, port: String, pword: String) -> Self {
        let client: Option<Client>;

        let port_num: u16; // Where we will put the converted port
                           // Port can range from 0 - 65535, the unsigned 16 bit int

        if port.is_empty() {
            // If no port was specified, default is 5432
            port_num = 5432;
        } else {
            match port.parse::<u16>() {
                Ok(parsed) => {
                    port_num = parsed;
                }

                Err(_) => {
                    println!("Port: {}", port);
                    eprintln!("Could not parse port. Using default 5432");
                    port_num = 5432;
                }
            }
        }

        // Creating connection string
        let connection_string: String = format!(
            "host={} dbname={} port={} user={} password={}",
            host, dbname, port_num, uname, pword
        );


        match Client::connect(&connection_string, NoTls) {
            Ok(c) => {
                client = Some(c);
                println!("Successfully connected");
            }
            Err(e) => {
                eprintln!("Connection Error: {}", e);
                client = None;
            }
        };

        Connection {
            client,
            host,
            dbname,
            user: uname,
            port,
            password: pword,
        }
    }

    /// Attempts to create a connection to the Postgres Database using information from the Connection String
    /// This function does not return anything, but will print out an error in the case that the connection was not
    /// successfull.
    pub fn connect(&mut self) {
        let connection_string = format!(
            "user={} host={} dbname={} password={} port={}",
            &self.user, &self.host, &self.dbname, &self.password, &self.port
        );
        self.client = match Client::connect(&connection_string, NoTls) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Error: {}", e);
                None
            }
        };
    }

    /// Runs a query to get the version of the Postgres Database
    /// Returns a `Row` of the version
    /// If there is an error, returns a `PGError`
    /// If the client is None, returns a `PGError`
    pub fn version(&mut self) -> Result<Row, PGError> {
        match &mut self.client {
            Some(ref mut s) => match s.query_one("SELECT version()", &[]) {
                Ok(r) => Ok(r),

                Err(_) => Err(QueryError),
            },
            None => {
                println!("Client was empty");
                Err(PGError::ClientEmpty)?
            }
        }
    }

    /// Runs a query to get all the know extensions of a Postgres Database
    /// Checks to ensure the `client` is actually connected to the data base
    /// If `client` is `None`, this function returns a `PGError` <br>
    /// On success this function returns a `Vec<Row>`, rows containing query information.
    pub fn get_extensions(&mut self) -> Result<Vec<Row>, PGError> {
        match &mut self.client {
            Some(ref mut client) => {
                let query_string = r#"
               SELECT current_database() AS db, name, installed_version, default_version
               FROM pg_available_extensions
               WHERE installed_version IS NOT NULL
               AND default_version IS NOT NULL
               AND installed_version != default_version
                "#;
                match client.query(query_string, &[]) {
                    Ok(r) => Ok(r),
                    Err(_) => Err(QueryError),
                }
            }

            None => {
                eprintln!("Could not query version");
                Err(PGError::ClientEmpty)?
            }
        }
    }

    /// This function runs a query to find the uptime of a given database <br>
    /// Returns a `Result<Vec<Row>, PGError>` <br>
    /// Errors in the case that the query was not succesfull or the `client` was `None`
    pub fn get_uptime(&mut self) -> Result<Vec<Row>, PGError> {
        let uptime_query = r#"
      SELECT date_trunc('second', current_timestamp - pg_postmaster_start_time()) as uptime;
      "#;
        match &mut self.client {
            Some(ref mut client) => match client.query(uptime_query, &[]) {
                Ok(r) => Ok(r),

                Err(_) => {
                    eprintln!("Couldnt query");
                    Err(QueryError)
                }
            },

            None => Err(PGError::ClientEmpty)?,
        }
    }

    /// This function allows the user to run a custom query, by taking a string. <br>
    /// Note: This function may return rows containing types not compatible with this program.
    ///
    #[allow(dead_code)]
    pub fn custom_query(&mut self, query: String) -> Result<Vec<Row>, PGError> {
        match &mut self.client {
            Some(ref mut c) => match c.query(&query, &[]) {
                Ok(r) => Ok(r),
                Err(_) => Err(QueryError),
            },

            // Client is empty, cannot run a query
            None => Err(PGError::ClientEmpty),
        }
    }

    /// This function will retrieve all tables with a public schema. It will return a `Vec<Row>`, with
    /// each row containing the name of the public table. <br>
    /// This function will return a `PGError` in the case that the query was unsucessfull or the `client`
    /// was `None`
    pub fn get_all_public_tables(&mut self) -> Result<Vec<Row>, PGError> {
        // Query to get all public tables in DB
        let table_query = r#"
            select table_name from information_schema.tables where table_schema='public';
        "#;

        match &mut self.client {
            Some(ref mut c) => match c.query(table_query, &[]) {
                Ok(r) => Ok(r),
                Err(_) => Err(QueryError),
            },

            // Client is empty, cannot run a query
            None => Err(PGError::ClientEmpty)?,
        }
    }

    /// Writes information from `Connection` to JSON, with the desired `Connection Name` specified by the user.
    /// Giving credit to GitHub Copilot on some of the logic for this function.
    pub fn write_to_json(&mut self, connection_name: String) -> Result<bool, PGError> {
        let json_string = json!({
                "connection_name": connection_name,
                "host": &self.host,
                "port": &self.port,
                "user": &self.user,
                "dbname": &self.dbname

        });

        let file_path = "./db_connections.json";

        let mut stored_connections = match File::open(file_path) {
            Ok(_file) => {
                let text = std::fs::read_to_string(file_path).unwrap();
                serde_json::from_str::<Value>(&text).unwrap()
            }
            Err(_) => return Err(JSONOpenFileError),
        };

        let connections_array = stored_connections
            .as_object_mut()
            .unwrap()
            .entry("connections")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .unwrap();

        let connection_names: Vec<&str> = connections_array
            .iter()
            .map(|conn| conn["connection_name"].as_str().unwrap())
            .collect();

        if connection_names.contains(&&connection_name[..]) {
            return Err(DuplicateConnection);
        }

        connections_array.push(json_string);

        let full_json_string =
            serde_json::to_string(&stored_connections).expect("Failed to serialize connections");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .expect("Failed to open file");

        file.write_all(full_json_string.as_bytes())
            .expect("Failed to write to file");

        Ok(true)
    }

    /// Attempts to read a connection from JSON file using specified name: `connection_name`
    /// Returns a new `Connection` if one could be matched, otherwise it will return an `Error`
    /// The `Connection` being returned has no password field, so the `Client` will be `None`
    pub fn read_from_json(&mut self, connection_name: String, password: String) -> Result<Connection, PGError> {
        let connection_values = {
            let text = std::fs::read_to_string("./db_connections.json").unwrap();
            serde_json::from_str::<Value>(&text).unwrap()
        };

        let num_elements = connection_values["connections"].as_array().unwrap().len();

        for index in 0..num_elements {
            if connection_values["connections"][index]["connection_name"]
                .as_str()
                .unwrap()
                == connection_name
            {
                // Returning new connection with no password field, this will make it so the Client become None
                self.dbname = connection_values["connections"][index]["dbname"].to_string();
                self.user = connection_values["connections"][index]["dbname"].to_string();
                self.port = connection_values["connections"][index]["dbname"].as_str().unwrap().to_string();
                self.host = connection_values["connections"][index]["dbname"].to_string();
                self.password = password.to_string();

                return Ok(Connection::new(
                    connection_values["connections"][index]["host"].to_string(),
                    connection_values["connections"][index]["dbname"].to_string(),
                    connection_values["connections"][index]["user"].to_string(),
                    connection_values["connections"][index]["port"].as_str().unwrap().to_string(),
                    password,
                ));
            }
        }
        Err(MatchNotFound)
    }
}

/// Prints out a welcome message including the author of this program, the name, and the version
pub fn welcome() {
    let welcome = r#"
    ==================================================
    =  PSQL Tools - A command line Postgres toolkit  =
    =  Author: Christian Torralba                    =
    =  Version: 1.0                                  =
    ==================================================
    "#;

    println!("{}", welcome);
}

/// Prints out the available options for the user to input
pub fn help_menu() {
    let help_string = r#"
    Help Menu:
    =   0 - Exit the program
    =   1 - Save your connection information to a file
    =   2 - Get the Uptime of your database
    =   3 - Get the Version of your database
    =   4 - List all public tables in your database
    =   5 - List all installed extensions
    =   6 - Run a custom query
    =   7 - Attempt to restablish connection to database
    =   8 - Attemp to load a connection from a file
    "#;
    println!("{}", help_string);
}
