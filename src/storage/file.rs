use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;

use super::parser::FileHeaderParser;
use storage::file_id::*;

#[derive(Debug)]
pub enum FileHeaderSpec {
    Null,
    Bool,
    Integer,
    UInteger,
    Float,
    Text,
    Key { name: &'static str, value_type: Box<FileHeaderSpec> },
    Map { keys: Vec<FileHeaderSpec> },
    Array { allowed_types: Vec<FileHeaderSpec> },
}

#[derive(Debug)]
#[derive(Clone)]
pub enum FileHeaderData {
    Null,
    Bool(bool),
    Integer(i64),
    UInteger(u64),
    Float(f64),
    Text(String),
    Key { name: String, value: Box<FileHeaderData> },
    Map { keys: Vec<FileHeaderData> },
    Array { values: Box<Vec<FileHeaderData>> },
}

impl Display for FileHeaderSpec {

    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            &FileHeaderSpec::Null       => write!(fmt, "NULL"),
            &FileHeaderSpec::Bool       => write!(fmt, "Bool"),
            &FileHeaderSpec::Integer    => write!(fmt, "Integer"),
            &FileHeaderSpec::UInteger   => write!(fmt, "UInteger"),
            &FileHeaderSpec::Float      => write!(fmt, "Float"),
            &FileHeaderSpec::Text       => write!(fmt, "Text"),
            &FileHeaderSpec::Key{name: ref n, value_type: ref vt} => {
                write!(fmt, "Key({:?}) -> {:?}", n, vt)
            }
            &FileHeaderSpec::Map{keys: ref ks} => {
                write!(fmt, "Map -> {:?}", ks)
            }
            &FileHeaderSpec::Array{allowed_types: ref at}  => {
                write!(fmt, "Array({:?})", at)
            }
        }
    }

}

pub struct MatchError<'a> {
    summary: String,
    expected: &'a FileHeaderSpec,
    found: &'a FileHeaderData
}

impl<'a> MatchError<'a> {

    pub fn new(s: String,
               ex: &'a FileHeaderSpec,
               found: &'a FileHeaderData) -> MatchError<'a> {
        MatchError {
            summary: s,
            expected: ex,
            found: found,
        }
    }

    pub fn format(&self) -> String {
        format!("MatchError: {:?}\nExpected: {:?}\nFound: {:?}\n",
               self.summary, self.expected, self.found)
    }
}

impl<'a> Error for MatchError<'a> {

    fn description(&self) -> &str {
        &self.summary[..]
    }

    fn cause(&self) -> Option<&Error> {
        None
    }

}

impl<'a> Debug for MatchError<'a> {

    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.format());
        Ok(())
    }

}

impl<'a> Display for MatchError<'a> {

    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.format());
        Ok(())
    }

}

pub fn match_header_spec<'a>(spec: &'a FileHeaderSpec, data: &'a FileHeaderData)
    -> Option<MatchError<'a>>
{
    debug!("Start matching:\n'{:?}'\non\n{:?}", spec, data);
    match (spec, data) {
        (&FileHeaderSpec::Null,     &FileHeaderData::Null)           => { }
        (&FileHeaderSpec::Bool,     &FileHeaderData::Bool(_))        => { }
        (&FileHeaderSpec::Integer,  &FileHeaderData::Integer(_))     => { }
        (&FileHeaderSpec::UInteger, &FileHeaderData::UInteger(_))    => { }
        (&FileHeaderSpec::Float,    &FileHeaderData::Float(_))       => { }
        (&FileHeaderSpec::Text,     &FileHeaderData::Text(_))        => { }

        (
            &FileHeaderSpec::Key{name: ref kname, value_type: ref vtype},
            &FileHeaderData::Key{name: ref n, value: ref val}
        ) => {
            debug!("Matching Key: '{:?}' == '{:?}', Value: '{:?}' == '{:?}'",
                    kname, n,
                    vtype, val);
            if kname != n {
                debug!("Keys not matching");
                unimplemented!();
            }
            return match_header_spec(&*vtype, &*val);
        }

        (
            &FileHeaderSpec::Map{keys: ref sks},
            &FileHeaderData::Map{keys: ref dks}
        ) => {
            debug!("Matching Map: '{:?}' == '{:?}'", sks, dks);

            for (s, d) in sks.iter().zip(dks.iter()) {
                let res = match_header_spec(s, d);
                if res.is_some() {
                    return res;
                }
            }
        }

        (
            &FileHeaderSpec::Array{allowed_types: ref vtypes},
            &FileHeaderData::Array{values: ref vs}
        ) => {
            debug!("Matching Array: '{:?}' == '{:?}'", vtypes, vs);
            for (t, v) in vtypes.iter().zip(vs.iter()) {
                let res = match_header_spec(t, v);
                if res.is_some() {
                    return res;
                }
            }
        }

        (k, v) => {
            return Some(MatchError::new(String::from("Expected type does not match found type"),
                                 k, v
                                 ))
        }
    }
    None
}

/*
 * Internal abstract view on a file. Does not exist on the FS and is just kept
 * internally until it is written to disk.
 */
pub struct File {
    header  : FileHeaderData,
    data    : String,
    id      : FileID,
}

impl File {

    pub fn new() -> File {
        let f = File {
            header: FileHeaderData::Null,
            data: String::from(""),
            id: File::get_new_file_id(),
        };
        debug!("Create new File object: {:?}", f);
        f
    }

    pub fn from_parser_result(id: FileID, header: FileHeaderData, data: String) -> File {
        let f = File {
            header: header,
            data: data,
            id: id,
        };
        debug!("Create new File object from parser result: {:?}", f);
        f
    }

    pub fn new_with_header(h: FileHeaderData) -> File {
        let f = File {
            header: h,
            data: String::from(""),
            id: File::get_new_file_id(),
        };
        debug!("Create new File object with header: {:?}", f);
        f
    }

    pub fn new_with_data(d: String) -> File {
        let f = File {
            header: FileHeaderData::Null,
            data: d,
            id: File::get_new_file_id(),
        };
        debug!("Create new File object with data: {:?}", f);
        f
    }

    pub fn new_with_content(h: FileHeaderData, d: String) -> File {
        let f = File {
            header: h,
            data: d,
            id: File::get_new_file_id(),
        };
        debug!("Create new File object with content: {:?}", f);
        f
    }

    pub fn contents(&self) -> (FileHeaderData, String) {
        (self.header.clone(), self.data.clone())
    }

    pub fn id(&self) -> FileID {
        self.id.clone()
    }

    fn get_new_file_id() -> FileID {
        use uuid::Uuid;
        Uuid::new_v4().to_hyphenated_string()
    }
}

impl Debug for File {

    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "File[{:?}] header: '{:?}', data: '{:?}')",
            self.id, self.header, self.data)
    }
}
