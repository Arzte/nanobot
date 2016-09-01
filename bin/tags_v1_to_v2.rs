//! Converts version 1 of the tags (stored in a JSON file) to version 2 of the
//! tag storage (stored in a `SQLite3` database).
// ISC License (ISC)
//
// Copyright (c) 2016, Austin Hellyer <hello@austinhellyer.me>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.

extern crate rusqlite;
extern crate serde_json;
extern crate walkdir;

use rusqlite::Connection;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let json_dir = Path::new("./.talked");

    if !json_dir.exists() || !json_dir.is_dir() {
        panic!("json directory must exist (./.talked)");
    }

    let walker = WalkDir::new(json_dir.to_str().unwrap()).into_iter();

    let conn = Connection::open(Path::new("./database.db")).expect("invalid db path");

    let mut values = String::new();

    for entry in walker {
        let entry = entry.unwrap();

        let path = entry.path().to_str().expect("Error unwrapping path str");

        if !path.ends_with("tags.json") {
            continue;
        }

        let file = File::open(path).unwrap();

        let tags: BTreeMap<String, String> = serde_json::from_reader(file).unwrap();

        let server_id = {
            let split_by_s: Vec<&str> = path.split('s').collect();

            let split_by_slash: Vec<&str> = split_by_s[1].split('/').collect();

            split_by_slash[0]
        };

        // Since joining strings of vectors in rust is not that easy, just push
        // to a string.
        for (k, v) in &tags {
            values.push_str(&format!("(\"{}\", \"{}\", \"{}\"),", k, server_id, v));
        }
    }

    if !values.is_empty() {
        // Remove the last `,` from the string
        let _ = values.pop();

        let query = format!("INSERT INTO tags (key, server_id, value)
                             \
                             VALUES {}",
                            values);

        conn.execute(&query, &[]).expect("Failed to insert query");
    }

    println!("Query ran");
}
