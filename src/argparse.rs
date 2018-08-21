use clap::{App, Arg, ArgMatches};

// static mut InsulaApp: Option<App> = None;
static mut INSULA_ARGS: Option<ArgMatches> = None;


pub fn parse_args() {
    let insula_app = App::new("insula")
        .arg(Arg::with_name("firmware")
             .help("Firmware path.")
             .takes_value(true)
             .required(true));

    unsafe {
        INSULA_ARGS = Some(insula_app.get_matches());  
    }   
}

pub fn get_args<'a>() -> &'a ArgMatches<'a> {
    unsafe {
        match INSULA_ARGS {
            Some(ref args) => args,
            None => panic!("Please call parse_args before get_args.")
        }
    }  
}