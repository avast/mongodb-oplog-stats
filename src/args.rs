//! Parsing of arguments for the tool.

use clap::value_t;
use clap::App;
use clap::Arg;
use clap::ArgMatches;
use rpassword;

use crate::error::Result;

/// Tool arguments.
#[derive(Debug)]
pub struct Args {
    /// Hostname for the MongoDB instance.
    pub host: String,

    /// TCP port on which the MongoDB instance is listening.
    pub port: u16,

    /// Username to use for authentication.
    pub username: Option<String>,

    /// Password to use for authentication.
    pub password: Option<String>,

    /// Name of the database to use for authentication.
    pub auth_db: Option<String>,

    /// Maximal number of documents in the oplog to process.
    pub limit: Option<u64>,

    /// Print statistics every time `N` documents have been processed.
    pub print_after: Option<u64>,
}

/// Parses tool arguments.
pub fn parse_args() -> Result<Args> {
    let matches = App::new("mongodb-oplog-stats")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Prints statistics about MongoDB oplog")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("host")
                .help("Resolvable hostname for the MongoDB instance to which to connect")
                .takes_value(true)
                .default_value("localhost")
                .display_order(1)
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("port")
                .help("TCP port on which the MongoDB instance listens for client connections")
                .takes_value(true)
                .default_value("27017")
                .display_order(2)
        )
        .arg(
            Arg::with_name("username")
                .short("u")
                .long("username")
                .value_name("username")
                .help("Username with which to authenticate to a MongoDB database that uses authentication")
                .takes_value(true)
                .display_order(3)
        )
        .arg(
            Arg::with_name("password")
                .short("p")
                .long("password")
                .value_name("password")
                .help("Password with which to authenticate to a MongoDB database that uses authentication")
                .long_help("Password with which to authenticate to a MongoDB database that uses authentication. Use in conjunction with the --username and --authenticationDatabase options. To prompt the user for the password, pass the --username option without --password or specify an empty string as the --password value, as in --password=\"\"")
                .takes_value(true)
                .display_order(4)
        )
        .arg(
            Arg::with_name("auth_db")
                .long("authenticationDatabase")
                .value_name("dbname")
                .help("Authentication database where the specified --username has been created")
                .long_help("Authentication database where the specified --username has been created. See https://docs.mongodb.com/manual/core/security-users/#user-authentication-database")
                .takes_value(true)
                .display_order(5)
        )
        .arg(
            Arg::with_name("limit")
                .short("l")
                .long("limit")
                .value_name("n")
                .help("Maximal number of documents in the oplog to process")
                .takes_value(true)
                .display_order(6)
        )
        .arg(
            Arg::with_name("print_after")
                .long("printAfter")
                .value_name("n")
                .help("Print statistics every time n documents have been processed")
                .takes_value(true)
                .display_order(7)
        )
        .get_matches();
    let limit = get_limit(&matches)?;
    let print_after = get_print_after(&matches)?;
    let host = get_host(&matches);
    let port = get_port(&matches)?;
    let username = get_username(&matches);
    let password = get_password(&matches, username.is_some())?;
    let auth_db = get_auth_db(&matches);

    Ok(Args {
        host,
        port,
        username,
        password,
        auth_db,
        limit,
        print_after,
    })
}

fn get_limit(matches: &ArgMatches) -> Result<Option<u64>> {
    if matches.is_present("limit") {
        Ok(Some(value_t!(matches.value_of("limit"), u64)?))
    } else {
        Ok(None)
    }
}

fn get_print_after(matches: &ArgMatches) -> Result<Option<u64>> {
    if matches.is_present("print_after") {
        let print_after = value_t!(matches.value_of("print_after"), u64)?;
        if print_after == 0 {
            clap::Error::with_description(
                "Value of --printAfter has to be positive",
                clap::ErrorKind::InvalidValue,
            )
            .exit()
        }
        Ok(Some(print_after))
    } else {
        Ok(None)
    }
}

fn get_host(matches: &ArgMatches) -> String {
    matches
        .value_of("host")
        .expect("should never happen (host should have a default value)")
        .to_owned()
}

fn get_port(matches: &ArgMatches) -> Result<u16> {
    Ok(value_t!(matches.value_of("port"), u16)?)
}

fn get_username(matches: &ArgMatches) -> Option<String> {
    matches.value_of("username").map(String::from)
}

fn get_password(matches: &ArgMatches, username_given: bool) -> Result<Option<String>> {
    let mut password = matches.value_of("password").map(String::from);
    if username_given && (password.is_none() || password == Some(String::new())) {
        password = Some(rpassword::read_password_from_tty(Some("Password: "))?);
    }
    Ok(password)
}

fn get_auth_db(matches: &ArgMatches) -> Option<String> {
    matches.value_of("auth_db").map(String::from)
}
