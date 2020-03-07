use flate2::read::ZlibDecoder;
use crate::repository::Repository;
use crate::repository::repo_path;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::iter::FromIterator;
use std::str;

#[derive(Debug)]
pub enum Object {
    Commit,
    Tree,
    Tag,
    Blob
}

#[derive(Debug)]
pub enum ObjectError {
    MalformedSha,
    MalformedObject,
    UnknownType,
}

impl Object {
    pub fn read(repo: &Repository, sha: Vec<char>) -> Result<Object, ObjectError> {
        let (dir, file_name) = sha.split_at(2);
        if dir.len() != 2 || file_name.is_empty() { return Err(ObjectError::MalformedSha) }

        let mut path_strings = vec![String::from("objects")];
        for slice in vec![dir, file_name] {
            path_strings.push(String::from_iter(slice))
        }
        let path = repo_path(repo, &path_strings);
        let mut file = File::open(&path).expect("Couldn't open object file.");
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).expect("Couldn't read object file");

        let arr: &[u8] = &buffer;
        let mut decoder = ZlibDecoder::new(arr);
        let mut decoded_contents: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut decoded_contents).expect("Couldn't decode object file.");

        let ascii_space: u8 = 32;
        let split: Vec<Vec<u8>> = split_like_string(&decoded_contents, ascii_space);
        let format_string = match split.first() {
            Some(v) => str::from_utf8(&v).unwrap(),
            None => return Err(ObjectError::MalformedObject)
        };

        return match format_string {
            "commit" => Ok(Object::Commit),
            "tree" => Ok(Object::Tree),
            "tag" => Ok(Object::Tag),
            "blob" => Ok(Object::Blob),
            _ => Err(ObjectError::UnknownType)
        }
    }
}

fn split_like_string<T: Copy>(vec: &Vec<T>, elem: T) -> Vec<Vec<T>> where T: PartialEq {
    let mut v: Vec<Vec<T>> = Vec::new();
    let mut cursor: Vec<T> = Vec::new();

    for e in vec {
        if *e == elem {
            v.push(cursor);
            cursor = Vec::new();
            continue;
        }

        cursor.push(*e);
    }

    v.push(cursor);

    return v;
}