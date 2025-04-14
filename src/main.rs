mod stream;
mod ca65_html_parser;

use std::{
    io::Read,
    collections::HashMap,
};
use serde::Serialize;
use stream::Stream;
use ca65_html_parser::KeywordInfo;

fn print_error_and_exit(msg: &str) {
    eprintln!("\x1b[31mERROR\x1b[0m {msg}");
    std::process::exit(1);
}

#[derive(Serialize)]
struct IndexedDocumentation {
    keys_to_doc: HashMap<String, KeywordInfo>,
    keys_with_shared_doc: HashMap<String, String>,
}

fn main() {
    let ca65_html_location = include_str!("../ca65-html-location.txt");
    let json_location = include_str!("../json-location.txt");

    // get contents of ca65.html
    let mut ca65_html_contents = String::new();
    let mut f = std::fs::File::open(ca65_html_location).expect("could not open ca65.html");
    f.read_to_string(&mut ca65_html_contents).expect("could not read ca65.html to string");

    // parse ca65.html to a <String, String> hashmap
    let ca65_html_stream = Stream::new(ca65_html_contents);
    let mut ca65_html_parser = ca65_html_parser::Ca65HtmlParser::new(ca65_html_stream);
    let ca65_doc = IndexedDocumentation {
        keys_to_doc: ca65_html_parser.parse_to_hashmap(),
        keys_with_shared_doc: HashMap::<String, String>::from([
            ("mac".to_string(), "macro".to_string()),
            ("endmac".to_string(), "endmacro".to_string()),
            ("delmac".to_string(), "delmacro".to_string()),
            ("exitmac".to_string(), "exitmacro".to_string()),
            ("ismnem".to_string(), "ismnemonic".to_string()),
            ("ref".to_string(), "referenced".to_string()),
            ("def".to_string(), "defined".to_string()),
            ("byt".to_string(), "byte".to_string()),
            ("refto".to_string(), "referto".to_string()),
            ("pagelen".to_string(), "pagelength".to_string()),
            ("undef".to_string(), "undefine".to_string()),
            ("fileopt".to_string(), "fopt".to_string()),
            ("endrep".to_string(), "endrepeat".to_string()),
        ]),
    };

    // for (k, v) in &hm {
    //     println!("{k} ::\n");
    //     println!("{v}-----------------\n\n\n\n");
    // }

    // write JSON-serialized data to output file
    if let Ok(json) = serde_json::to_string_pretty(&ca65_doc) {
        if std::fs::write(json_location, json).is_err() {
            print_error_and_exit(&format!("could not write to JSON file at {json_location}"));
        } else {
            println!("\x1b[32mSuccessfully wrote JSON to \x1b[0m{json_location}");
        }
    } else {
        print_error_and_exit("could not serialize markdown hashmap to JSON");
    }
}
