mod stream;
mod ca65_html_parser;

use std::{
    io::Read,
    collections::HashMap,
};
use serde::Serialize;
use stream::Stream;

fn print_error_and_exit(msg: &str) {
    eprintln!("\x1b[31mERROR\x1b[0m {msg}");
    std::process::exit(1);
}

#[derive(Serialize)]
struct Ca65Doc {
    keywords_to_markdown: HashMap<String, String>,
    duplicate_keywords_to_keywords: HashMap<String, String>,
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
    let ca65_doc = Ca65Doc {
        keywords_to_markdown: ca65_html_parser.parse_to_hashmap(),
        duplicate_keywords_to_keywords: HashMap::<String, String>::from([
            (".MAC".to_string(), ".MACRO".to_string()),
            (".ENDMAC".to_string(), ".ENDMACRO".to_string()),
            (".DELMAC".to_string(), ".DELMACRO".to_string()),
            (".EXITMAC".to_string(), ".EXITMACRO".to_string()),
            (".ISMNEM".to_string(), ".ISMNEMONIC".to_string()),
            (".REF".to_string(), ".REFERENCED".to_string()),
            (".DEF".to_string(), ".DEFINED".to_string()),
            (".BYT".to_string(), ".BYTE".to_string()),
            (".REFTO".to_string(), ".REFERTO".to_string()),
            (".PAGELEN".to_string(), ".PAGELENGTH".to_string()),
            (".UNDEF".to_string(), ".UNDEFINE".to_string()),
            (".FILEOPT".to_string(), ".FOPT".to_string()),
            (".ENDREP".to_string(), ".ENDREPEAT".to_string()),
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
