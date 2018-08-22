use clap::{App, Arg, ArgMatches};

static mut INSULA_ARGS: Option<ArgMatches> = None;

pub fn parse_args<'a>() -> &'a ArgMatches<'a>{
    let insula_app = App::new("insula")
        .arg(Arg::with_name("firmware")
             .long("fw")
             .help("Firmware path.")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("memory_mb")
             .long("memory_mb")
             .help("The amount of RAM memory, in MB.")
             .takes_value(true)
             .required(false)
             .default_value("128"));

    unsafe {
        INSULA_ARGS = Some(insula_app.get_matches());
    }

    get_args()
}

pub fn get_args<'a>() -> &'a ArgMatches<'a> {
    unsafe {
        match INSULA_ARGS {
            Some(ref args) => args,
            None => panic!("Please call parse_args before get_args.")
        }
    }
}
