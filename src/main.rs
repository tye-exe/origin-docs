use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> () {
    // Parses the origins
    let origins = origin_docs_gen::parse_origins();
    // Parses the powers
    let powers = origin_docs_gen::parse_powers(&origins);

    // Parses the powers & formats them & origins into markdown.
    let (data_prefix, data) = origin_docs_gen::format_into_markdown(&origins, powers);

    // Opens the file & writes the data
    let result = File::create("Origins.md").and_then(|file| {
        let mut writer = BufWriter::new(file);

        writer.write(data_prefix.as_bytes())
            .and_then(|_| {
                writer.write(data.as_bytes())
            })
    });

    if result.is_err() {
        eprintln!("Unable to write data to \"Origins.md\" : {}", result.err().unwrap());
    }
}