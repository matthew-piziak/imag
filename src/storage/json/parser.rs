use serde_json::{Value, from_str};
use serde_json::error::Result as R;
use serde_json::Serializer;
use serde::ser::Serialize;
use serde::ser::Serializer as Ser;

use std::collections::HashMap;
use std::io::stdout;
use std::error::Error;

use super::super::parser::{FileHeaderParser, ParserError};
use super::super::file::{FileHeaderSpec, FileHeaderData};


pub struct JsonHeaderParser {
    spec: Option<FileHeaderSpec>,
}

impl JsonHeaderParser {

    pub fn new(spec: Option<FileHeaderSpec>) -> JsonHeaderParser {
        JsonHeaderParser {
            spec: spec
        }
    }

}

impl FileHeaderParser for JsonHeaderParser {

    fn read(&self, string: Option<String>)
        -> Result<FileHeaderData, ParserError>
    {
        if string.is_some() {
            let s = string.unwrap();
            debug!("Deserializing: {}", s);
            let fromstr : R<Value> = from_str(&s[..]);
            if let Ok(ref content) = fromstr {
                return Ok(visit_json(&content))
            }
            let oe = fromstr.err().unwrap();
            let s = format!("JSON parser error: {}", oe.description());
            let e = ParserError::short(&s[..], s.clone(), 0);
            Err(e)
        } else {
            Ok(FileHeaderData::Null)
        }
    }

    fn write(&self, data: &FileHeaderData) -> Result<String, ParserError> {
        let mut s = Vec::<u8>::new();
        {
            let mut ser = Serializer::pretty(&mut s);
            data.serialize(&mut ser);
        }

        String::from_utf8(s).or(
            Err(ParserError::short("Cannot parse utf8 bytes",
                                   String::from("<not printable>"),
                                   0)))
    }

}

// TODO: This function must be able to return a parser error
fn visit_json(v: &Value) -> FileHeaderData {
    match v {
        &Value::Null             => FileHeaderData::Null,
        &Value::Bool(b)          => FileHeaderData::Bool(b),
        &Value::I64(i)           => FileHeaderData::Integer(i),
        &Value::U64(u)           => FileHeaderData::UInteger(u),
        &Value::F64(f)           => FileHeaderData::Float(f),
        &Value::String(ref s)        => FileHeaderData::Text(s.clone()),
        &Value::Array(ref vec)       => {
            FileHeaderData::Array {
                values: Box::new(vec.clone().into_iter().map(|i| visit_json(&i)).collect())
            }
        },
        &Value::Object(ref btree)    => {
            let btree = btree.clone();
            FileHeaderData::Map{
                keys: btree.into_iter().map(|(k, v)|
                    FileHeaderData::Key {
                        name: k,
                        value: Box::new(visit_json(&v)),
                    }
                ).collect()
            }
        }
    }
}

impl Serialize for FileHeaderData {

    fn serialize<S>(&self, ser: &mut S) -> Result<(), S::Error>
        where S: Ser
    {
        match self {
            &FileHeaderData::Null               => {
                let o : Option<bool> = None;
                o.serialize(ser)
            },
            &FileHeaderData::Bool(ref b)            => b.serialize(ser),
            &FileHeaderData::Integer(ref i)         => i.serialize(ser),
            &FileHeaderData::UInteger(ref u)        => u.serialize(ser),
            &FileHeaderData::Float(ref f)           => f.serialize(ser),
            &FileHeaderData::Text(ref s)            => (&s[..]).serialize(ser),
            &FileHeaderData::Array{values: ref vs}  => vs.serialize(ser),
            &FileHeaderData::Map{keys: ref ks}      => {
                let mut hm = HashMap::new();

                for key in ks {
                    if let &FileHeaderData::Key{name: ref n, value: ref v} = key {
                        hm.insert(n, v);
                    } else {
                        panic!("Not a key: {:?}", key);
                    }
                }

                hm.serialize(ser)
            },
            &FileHeaderData::Key{name: ref n, value: ref v} => unreachable!(),

        }
    }

}

#[cfg(test)]
mod test {

    use super::JsonHeaderParser;
    use storage::parser::{FileHeaderParser, ParserError};
    use storage::file::{FileHeaderSpec, FileHeaderData};

    #[test]
    fn test_deserialization() {
        use storage::file::FileHeaderData as FHD;
        use storage::file::FileHeaderSpec as FHS;

        let text = String::from("{\"a\": 1, \"b\": -2}");
        let spec = FHS::Map {
            keys: vec![
                FHS::Key {
                    name: String::from("a"),
                    value_type: Box::new(FHS::UInteger)
                },
                FHS::Key {
                    name: String::from("b"),
                    value_type: Box::new(FHS::Integer)
                }
            ]
        };

        let parser = JsonHeaderParser::new(Some(spec));
        let parsed = parser.read(Some(text));
        assert!(parsed.is_ok(), "Parsed is not ok: {:?}", parsed);

        match parsed.ok() {
            Some(FHD::Map{keys: keys}) => {
                for k in keys {
                    match k {
                        FHD::Key{name: name, value: box value} => {
                            assert!(name == "a" || name == "b", "Key unknown");
                            match &value {
                                &FHD::UInteger(u) => assert_eq!(u, 1),
                                &FHD::Integer(i) => assert_eq!(i, -2),
                                _ => assert!(false, "Integers are not here"),
                            }
                        },
                        _ => assert!(false, "Key is not a Key"),
                    }
                }
            },

            _ => assert!(false, "Parsed is not a map"),
        }
    }

}
