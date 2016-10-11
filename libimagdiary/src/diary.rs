//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

use std::cmp::Ordering;

use libimagstore::store::Store;
use libimagstore::storeid::IntoStoreId;
use libimagerror::trace::trace_error;

use chrono::offset::local::Local;
use chrono::Datelike;
use itertools::Itertools;
use chrono::naive::datetime::NaiveDateTime;

use entry::Entry;
use diaryid::DiaryId;
use error::DiaryError as DE;
use error::DiaryErrorKind as DEK;
use result::Result;
use iter::DiaryEntryIterator;
use is_in_diary::IsInDiary;

#[derive(Debug)]
pub struct Diary<'a> {
    store: &'a Store,
    name: &'a str,
}

impl<'a> Diary<'a> {

    pub fn open(store: &'a Store, name: &'a str) -> Diary<'a> {
        Diary {
            store: store,
            name: name,
        }
    }

    // create or get a new entry for today
    pub fn new_entry_today(&self) -> Result<Entry> {
        let dt  = Local::now();
        let ndt = dt.naive_local();
        let id  = DiaryId::new(String::from(self.name), ndt.year(), ndt.month(), ndt.day(), 0, 0);
        self.new_entry_by_id(id)
    }

    pub fn new_entry_by_id(&self, id: DiaryId) -> Result<Entry> {
        self.retrieve(id.with_diary_name(String::from(self.name)))
    }

    pub fn retrieve(&self, id: DiaryId) -> Result<Entry> {
        id.into_storeid()
            .and_then(|id| self.store.retrieve(id))
            .map(|fle| Entry::new(fle))
            .map_err(|e| DE::new(DEK::StoreWriteError, Some(Box::new(e))))
    }

    // Get an iterator for iterating over all entries
    pub fn entries(&self) -> Result<DiaryEntryIterator<'a>> {
        self.store
            .retrieve_for_module("diary")
            .map(|iter| DiaryEntryIterator::new(self.name, self.store, iter))
            .map_err(|e| DE::new(DEK::StoreReadError, Some(Box::new(e))))
    }

    pub fn delete_entry(&self, entry: Entry) -> Result<()> {
        if !entry.is_in_diary(self.name) {
            return Err(DE::new(DEK::EntryNotInDiary, None));
        }
        let id = entry.get_location().clone();
        drop(entry);

        self.store.delete(id)
            .map_err(|e| DE::new(DEK::StoreWriteError, Some(Box::new(e))))
    }

    pub fn get_youngest_entry(&self) -> Option<Result<Entry>> {
        match self.entries() {
            Err(e) => Some(Err(e)),
            Ok(entries) => {
                entries.sorted_by(|a, b| {
                    match (a, b) {
                        (&Ok(ref a), &Ok(ref b)) => {
                            let a : NaiveDateTime = a.diary_id().into();
                            let b : NaiveDateTime = b.diary_id().into();

                            a.cmp(&b)
                        },

                        (&Ok(_), &Err(ref e))  => {
                            trace_error(e);
                            Ordering::Less
                        },
                        (&Err(ref e), &Ok(_))  => {
                            trace_error(e);
                            Ordering::Greater
                        },
                        (&Err(ref e1), &Err(ref e2)) => {
                            trace_error(e1);
                            trace_error(e2);
                            Ordering::Equal
                        },
                    }
                }).into_iter().next()
            }
        }
    }

    pub fn name(&self) -> &'a str {
        &self.name
    }
}

