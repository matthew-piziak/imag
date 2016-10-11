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

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::io::BufRead;
use std::result::Result as RResult;

use toml::Value;
use uuid::Uuid;

use task_hookrs::task::Task as TTask;
use task_hookrs::import::{import_task, import_tasks};

use libimagstore::store::{FileLockEntry, Store};
use libimagstore::storeid::{IntoStoreId, StoreIdIterator, StoreId};
use libimagerror::trace::MapErrTrace;
use libimagutil::debug_result::DebugResult;
use module_path::ModuleEntryPath;

use error::{TodoError, TodoErrorKind, MapErrInto};
use result::Result;

/// Task struct containing a `FileLockEntry`
#[derive(Debug)]
pub struct Task<'a>(FileLockEntry<'a>);

impl<'a> Task<'a> {

    /// Concstructs a new `Task` with a `FileLockEntry`
    pub fn new(fle: FileLockEntry<'a>) -> Task<'a> {
        Task(fle)
    }

    pub fn import<R: BufRead>(store: &'a Store, mut r: R) -> Result<(Task<'a>, String, Uuid)> {
        let mut line = String::new();
        r.read_line(&mut line);
        import_task(&line.as_str())
            .map_err_into(TodoErrorKind::ImportError)
            .map_dbg_err_str("Error while importing task")
            .map_err_dbg_trace()
            .and_then(|t| {
                let uuid = t.uuid().clone();
                t.into_task(store).map(|t| (t, line, uuid))
            })
    }

    /// Get a task from an import string. That is: read the imported string, get the UUID from it
    /// and try to load this UUID from store.
    ///
    /// Possible return values are:
    ///
    /// * Ok(Ok(Task))
    /// * Ok(Err(String)) - where the String is the String read from the `r` parameter
    /// * Err(_)          - where the error is an error that happened during evaluation
    ///
    pub fn get_from_import<R: BufRead>(store: &'a Store, mut r: R) -> Result<RResult<Task<'a>, String>>
    {
        let mut line = String::new();
        r.read_line(&mut line);
        Task::get_from_string(store, line)
    }

    /// Get a task from a String. The String is expected to contain the JSON-representation of the
    /// Task to get from the store (only the UUID really matters in this case)
    ///
    /// For an explanation on the return values see `Task::get_from_import()`.
    pub fn get_from_string(store: &'a Store, s: String) -> Result<RResult<Task<'a>, String>> {
        import_task(s.as_str())
            .map_err_into(TodoErrorKind::ImportError)
            .map_dbg_err_str("Error while importing task")
            .map_err_dbg_trace()
            .map(|t| t.uuid().clone())
            .and_then(|uuid| Task::get_from_uuid(store, uuid))
            .and_then(|o| match o {
                None    => Ok(Err(s)),
                Some(t) => Ok(Ok(t)),
            })
    }

    /// Get a task from an UUID.
    ///
    /// If there is no task with this UUID, this returns `Ok(None)`.
    pub fn get_from_uuid(store: &'a Store, uuid: Uuid) -> Result<Option<Task<'a>>> {
        ModuleEntryPath::new(format!("taskwarrior/{}", uuid))
            .into_storeid()
            .and_then(|store_id| store.get(store_id))
            .map(|o| o.map(Task::new))
            .map_err_into(TodoErrorKind::StoreError)
    }

    /// Same as Task::get_from_import() but uses Store::retrieve() rather than Store::get(), to
    /// implicitely create the task if it does not exist.
    pub fn retrieve_from_import<R: BufRead>(store: &'a Store, mut r: R) -> Result<Task<'a>> {
        let mut line = String::new();
        r.read_line(&mut line);
        Task::retrieve_from_string(store, line)
    }

    /// Retrieve a task from a String. The String is expected to contain the JSON-representation of
    /// the Task to retrieve from the store (only the UUID really matters in this case)
    pub fn retrieve_from_string(store: &'a Store, s: String) -> Result<Task<'a>> {
        Task::get_from_string(store, s)
            .and_then(|opt| match opt {
                Ok(task)    => Ok(task),
                Err(string) => import_task(string.as_str())
                    .map_err_into(TodoErrorKind::ImportError)
                    .map_dbg_err_str("Error while importing task")
                    .map_err_dbg_trace()
                    .and_then(|t| t.into_task(store)),
            })
    }

    pub fn delete_by_imports<R: BufRead>(store: &Store, r: R) -> Result<()> {
        use serde_json::ser::to_string as serde_to_string;
        use task_hookrs::status::TaskStatus;

        for (counter, res_ttask) in import_tasks(r).into_iter().enumerate() {
            match res_ttask {
                Ok(ttask) => {
                    if counter % 2 == 1 {
                        // Only every second task is needed, the first one is the
                        // task before the change, and the second one after
                        // the change. The (maybe modified) second one is
                        // expected by taskwarrior.
                        match serde_to_string(&ttask).map_err_into(TodoErrorKind::ImportError) {
                            // use println!() here, as we talk with TW
                            Ok(val) => println!("{}", val),
                            Err(e)  => return Err(e),
                        }

                        // Taskwarrior does not have the concept of deleted tasks, but only modified
                        // ones.
                        //
                        // Here we check if the status of a task is deleted and if yes, we delete it
                        // from the store.
                        if *ttask.status() == TaskStatus::Deleted {
                            match Task::delete_by_uuid(store, *ttask.uuid()) {
                                Ok(_)  => info!("Deleted task {}", *ttask.uuid()),
                                Err(e) => return Err(e),
                            }
                        }
                    } // end if c % 2
                },
                Err(e) => return Err(e).map_err_into(TodoErrorKind::ImportError),
            }
        }
        Ok(())
    }

    pub fn delete_by_uuid(store: &Store, uuid: Uuid) -> Result<()> {
        ModuleEntryPath::new(format!("taskwarrior/{}", uuid))
            .into_storeid()
            .and_then(|id| store.delete(id))
            .map_err(|e| TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e))))
    }

    pub fn all_as_ids(store: &Store) -> Result<StoreIdIterator> {
        store.retrieve_for_module("todo/taskwarrior")
            .map_err(|e| TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e))))
    }

    pub fn all(store: &Store) -> Result<TaskIterator> {
        Task::all_as_ids(store)
            .map(|iter| TaskIterator::new(store, iter))
    }

}

impl<'a> Deref for Task<'a> {
    type Target = FileLockEntry<'a>;

    fn deref(&self) -> &FileLockEntry<'a> {
        &self.0
    }

}

impl<'a> DerefMut for Task<'a> {

    fn deref_mut(&mut self) -> &mut FileLockEntry<'a> {
        &mut self.0
    }

}

/// A trait to get a `libimagtodo::task::Task` out of the implementing object.
pub trait IntoTask<'a> {

    /// # Usage
    /// ```ignore
    /// use std::io::stdin;
    ///
    /// use task_hookrs::task::Task;
    /// use task_hookrs::import::import;
    /// use libimagstore::store::{Store, FileLockEntry};
    ///
    /// if let Ok(task_hookrs_task) = import(stdin()) {
    ///     // Store is given at runtime
    ///     let task = task_hookrs_task.into_filelockentry(store);
    ///     println!("Task with uuid: {}", task.flentry.get_header().get("todo.uuid"));
    /// }
    /// ```
    fn into_task(self, store : &'a Store) -> Result<Task<'a>>;

}

impl<'a> IntoTask<'a> for TTask {

    fn into_task(self, store : &'a Store) -> Result<Task<'a>> {
        let uuid     = self.uuid();
        ModuleEntryPath::new(format!("taskwarrior/{}", uuid))
            .into_storeid()
            .map_err_into(TodoErrorKind::StoreIdError)
            .and_then(|id| {
                store.retrieve(id)
                    .map_err_into(TodoErrorKind::StoreError)
                    .and_then(|mut fle| {
                        {
                            let mut hdr = fle.get_header_mut();
                            let read = hdr.read("todo").map_err_into(TodoErrorKind::StoreError);
                            if try!(read).is_none() {
                                try!(hdr
                                    .set("todo", Value::Table(BTreeMap::new()))
                                    .map_err_into(TodoErrorKind::StoreError));
                            }

                            try!(hdr.set("todo.uuid", Value::String(format!("{}",uuid)))
                                 .map_err_into(TodoErrorKind::StoreError));
                        }

                        // If none of the errors above have returned the function, everything is fine
                        Ok(Task::new(fle))
                    })
            })
    }

}

trait FromStoreId {
    fn from_storeid<'a>(&'a Store, StoreId) -> Result<Task<'a>>;
}

impl<'a> FromStoreId for Task<'a> {

    fn from_storeid<'b>(store: &'b Store, id: StoreId) -> Result<Task<'b>> {
        match store.retrieve(id) {
            Err(e) => Err(TodoError::new(TodoErrorKind::StoreError, Some(Box::new(e)))),
            Ok(c)  => Ok(Task::new( c )),
        }
    }
}

pub struct TaskIterator<'a> {
    store: &'a Store,
    iditer: StoreIdIterator,
}

impl<'a> TaskIterator<'a> {

    pub fn new(store: &'a Store, iditer: StoreIdIterator) -> TaskIterator<'a> {
        TaskIterator {
            store: store,
            iditer: iditer,
        }
    }

}

impl<'a> Iterator for TaskIterator<'a> {
    type Item = Result<Task<'a>>;

    fn next(&mut self) -> Option<Result<Task<'a>>> {
        self.iditer.next().map(|id| Task::from_storeid(self.store, id))
    }
}

