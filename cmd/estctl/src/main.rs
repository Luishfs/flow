use clap;
use estuary_json::schema;
use serde_json;
use serde_yaml;
use std::boxed::Box;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use url;

mod specs;

type Error = Box<dyn std::error::Error>;

fn main() {
    let matches = clap::App::new("Estuary CLI")
        .version("v0.1.0")
        .author("Estuary Technologies, Inc. \u{00A9}2020")
        .about("Command-line interface for working with Estuary projects")
        .subcommand(
            clap::SubCommand::with_name("build")
                .about("Build an Estuary specification tree into a bundle")
                .arg(
                    clap::Arg::with_name("root")
                        .short("r")
                        .long("root")
                        .takes_value(true)
                        .required(true)
                        .help("Path to specification which roots the hierarchy"),
                ),
        )
        .get_matches();

    let result: Result<(), Error> = match matches.subcommand() {
        ("build", Some(sub)) => do_build(sub),
        _ => Ok(()),
    };

    match result {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    };
}

mod bundle;

fn do_build(args: &clap::ArgMatches) -> Result<(), Error> {
    let root = args.value_of("root").unwrap();
    let root = fs::canonicalize(root)?;
    let root = url::Url::from_file_path(&root).unwrap();

    println!("root specification is {}", &root);

    let mut loader = bundle::loader::Loader::new(Box::new(DirectFileSystem {}));

    let specs = loader.load_node(root);

    /*

    let project = args.value_of("path").unwrap();
    let project = fs::canonicalize(project)?;

    let project_url = url::Url::from_file_path(&project).unwrap();

    let project_spec: specs::Root = {
        let f = fs::File::open(&project)?;
        let br = io::BufReader::new(f);
        serde_yaml::from_reader(br)?
    };

    let mut raw_schemas: BTreeMap<url::Url, Box<serde_json::Value>> = BTreeMap::new();

    for c in &project_spec.collections {
        let schema_url = project_url.join(&c.schema)?;

        let mut load_url = schema_url.clone();
        load_url.set_fragment(None);

        if raw_schemas.contains_key(&load_url) {
            continue;
        }

        let r = open_schema(&load_url)?;
        let r = io::BufReader::new(r);

        let raw_schema: serde_json::Value = {
            if load_url.path().ends_with(".yaml") {
                serde_yaml::from_reader(r)?
            } else if load_url.path().ends_with(".json") {
                serde_json::from_reader(r)?
            } else {
                panic!("unsupported schema extension '{}'", load_url);
            }
        };
        raw_schemas.insert(load_url.clone(), Box::new(raw_schema));

        let compiled_schema: schema::Schema<specs::Annotation> =
            schema::build::build_schema(load_url.clone(), &raw_schemas[&load_url])?;

        println!("loaded {:?}", load_url);
        //println!("");
        //println!("raw {:?}", &raw_schemas[&load_url]);
        //println!("");
        //println!("compiled {:?}", &compiled_schema);
    }
    */

    println!("specs: {:?}", specs);
    specs?;
    Ok(())
}

struct DirectFileSystem {}

impl bundle::loader::FileSystem for DirectFileSystem {
    fn open(&self, url: &url::Url) -> Result<Box<dyn io::Read>, Error> {
        match url.scheme() {
            "file" => {
                let path = url.to_file_path().unwrap();
                Ok(Box::new(fs::File::open(path)?))
            }
            _ => panic!("unknown schema '{}'", url.scheme()),
        }
    }
}
