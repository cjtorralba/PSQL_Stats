# PSQL Stats
Author: [Christian Torralba]

## Description
The intended purpose of this project was to explore the Postgres crate that is a part of Rust.
The project is a simple CLI tool that allows the user to connect to a Postgres database and
run queries against it. The user can also specify a file that contains a list of previously established connection, should they wish 
to connect to a database that they have already connected to before. There are a number of commands that the user can run, ranging from getting
the uptime of the database, to viewing all public tables.

This program is purely intended to make connecting to a database for simple request easier, and to explore the Postgres crate.

### Usage
The program is run via command line.  
Usage: 
``` 
-H, --host <HOST>          Postgres Database Hostname
-U, --user <USER>          Postgres Database Username, will default to "postgres" if no username is provided [default: postgres]
-d, --dbname <DBNAME>      
-p, --port <PORT>          Postgres Database Port, will default to 5432 if no port is provided
-W, --password <PASSWORD>  Postgres Database Password
-l <LOAD>                  Name of previously saved connection
-h, --help                 Print help
```
Once the program has been loaded, a help menu appears, listing the possible command available to the user:
```
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
```

Each time a command is run, the user is also told the connection status, either connected, or disconnected.

### Functionality (Problems and Non-Problems)
I had many ideas for this project but as the term went on it was increasingly difficult to find time to work on it.
One big issue I ran into was the Postgres crate itself, the examples were not very clear on the website so I had a hard time figuring out how to use it.
I believe I also may have picked the wrong crate! The Postgres crate run synchronously, which not only means it could be slower, but everyone seemed to be using the tokio-postgres crate.
The Postgres datatype were also a little difficult to overcome, as many of them lacked documentation, or even access. I was unable to use the Postgres::Error type, as it was private, and I was unable to find a way to convert the error type that was returned from the Postgres crate to a String. I ended up creating an alternate PGError type that I would return in place of
the errors given by the Postgres crate.


### Testing
Unfortunately testing this project was near impossible, since it requires a database to connect to. I did not have the time to create a docker container to run a Postgres database, so I was unable to test the program. I did however test the program manually, and it seems to work as intended.