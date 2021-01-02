use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(rename = "AuthorID")]
    pub author_id: Box<str>,
    #[serde(rename = "Author")]
    pub author: Box<str>,
    #[serde(rename = "Date")]
    pub date: Box<str>,
    #[serde(rename = "Content")]
    pub content: Box<str>,
    #[serde(rename = "Attachments")]
    pub attachments: Box<str>,
    #[serde(rename = "Reactions")]
    pub reactions: Box<str>,
}

pub fn read_chats(exports: &Path) -> Result<Vec<(Box<str>, Box<[Record]>)>> {
    let mut files = Vec::new();

    for f in fs::read_dir(exports)? {
        let f = f?;
        let meta = f.metadata()?;

        if !meta.is_file() {
            continue;
        }

        let mut records = Vec::new();

        let path = f.path();

        let name = match path.file_stem() {
            Some(stem) => stem.to_string_lossy(),
            None => continue,
        };

        if let Ok(content) = fs::read(&path) {
            let reader = io::Cursor::new(&content[..]);

            for record in csv::Reader::from_reader(reader).deserialize() {
                let record: Record = record?;
                records.push(record);
            }
        }

        files.push((name.into(), records.into()));
    }

    Ok(files)
}
