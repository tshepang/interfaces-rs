extern crate gcc;
extern crate handlebars as hbs;
extern crate rustc_serialize;

use std::collections::BTreeMap;
use std::convert::From;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::exit;

use rustc_serialize::json::{Json, ToJson};

fn main() {
    // Template the file.
    if let Err(e) = template_file() {
        println!("Error creating `constants.c` from template");
        println!("-> {:?}", e);
        exit(1);
    }

    // Build the final library
    let mut cfg = gcc::Config::new();

    let path1 = Path::new("src").join("constants.c");
    let path2 = Path::new("src").join("helpers.c");
    cfg.file(&path1)
       .file(&path2)
       .compile("libinterfaces.a");
}

fn template_file() -> Result<(), Error> {
    // Open and read the file.
    let in_path = Path::new("src").join("constants.c.in");
    let mut f = try!(File::open(&in_path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));

    let mut handlebars = hbs::Handlebars::new();
    try!(handlebars.register_template_string("constants", s));

    let out_path = Path::new("src").join("constants.c");
    let mut f = try!(File::create(&out_path));

    let data = make_data();
    let context = hbs::Context::wraps(&data);
    try!(handlebars.renderw("constants", &context, &mut f));

    Ok(())
}

fn make_data() -> BTreeMap<String, Json> {
    let mut data = BTreeMap::new();

    // These constants are "dynamically" generated by compiling a C file that includes their value
    // and then including that in the final build.  See `constants.rs` for a function that can be
    // used to retrieve them.
    let names: &[&str] = &[
        // IOCTLs
        "SIOCGIFCONF",
        "SIOCGIFHWADDR",
        "SIOCGIFFLAGS",
        "SIOCSIFFLAGS",

        // Address families
        "AF_LINK",
        "AF_PACKET", // Only on Linux

        // Miscellaneous
        "sizeof(struct ifreq)",
    ];

    let snames = names
        .into_iter()
        .map(|x| String::from(*x))
        .collect::<Vec<String>>();
    data.insert("constants".to_string(), snames.to_json());

    data
}

#[derive(Debug)]
enum Error {
    IoError(io::Error),
    TemplateError(hbs::TemplateError),
    RenderError(hbs::RenderError)
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<hbs::TemplateError> for Error {
    fn from(e: hbs::TemplateError) -> Error {
        Error::TemplateError(e)
    }
}

impl From<hbs::RenderError> for Error {
    fn from(e: hbs::RenderError) -> Error {
        Error::RenderError(e)
    }
}
