use std::collections::HashMap;
use std::fs::OpenOptions;

use serde::{Deserialize, Serialize};
use sqlite::{Connection, ConnectionWithFullMutex, ReadableWithIndex, State, Statement, Value};

use crate::api::ApiAddRequest;
use crate::utils::now;


const DB_SQLITE: &str = "sqlite";
const DB_MEMORY: &str = "memory";
const DB_FILE: &str = "file";

const PREPARE_DB_SQLITE_QUERY: &str = "CREATE TABLE IF NOT EXISTS msg (id TEXT NOT NULL, data TEXT, max_clicks INT NOT NULL, created INT NOT NULL, lifetime INT NOT NULL);";
const SELECT_BY_ID_SQLITE_QUERY: &str = "SELECT * FROM msg WHERE id = :id LIMIT 1";
const DELETE_BY_ID_SQLITE_QUERY: &str = "DELETE FROM msg WHERE id = :id";
const UPDATE_BY_ID_SQLITE_QUERY: &str = "UPDATE msg SET max_clicks = :max_clicks WHERE id = :id";
const INSERT_SQLITE_QUERY: &str = "INSERT INTO msg (id, data, max_clicks, created, lifetime) VALUES (:id, :data, :max_clicks, :created, :lifetime)";

pub const SQLITE_ERROR: &str = "sqlite error";
pub const SQLITE_CREATE_TABLE_ERROR: &str = "sqlite create table error";
pub const BIND_ERROR: &str = "bind error";

pub const NOT_FOUND_ERROR: &str = "not found";
pub const UNKNOWN_DB_TYPE_ERROR: &str = "unknown db kind";
pub const ALREADY_EXISTS_ERROR: &str = "already exists";
pub const DO_NOT_EXISTS_ERROR: &str = "do not exists";
pub const DELETE_ERROR: &str = "delete error";

pub const IO_WRITE_ERROR: &str = "write error";
pub const IO_READ_ERROR: &str = "read error";
pub const IO_CREATE_ERROR: &str = "create error";


pub trait DbEngine: Sync + Send {
    /// Insert data from ApiAddRequest with given id to database
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> Result<(), &'static str>;

    /// Get record from database by id
    fn get(&self, id: &String) -> Result<Record, &'static str>;

    /// Delete record from database by id
    fn delete(&mut self, id: &String)-> Result<(), &'static str>;

    /// Update record in database
    fn update(&mut self, r: Record)-> Result<(), &'static str>;

    /// Create new instance of engine
    fn new(path: &String) -> Result<Self, &'static str> where Self: Sized;

    /// Create new instance of engine in the heap
    fn new_boxed(path: &String) -> Result<Box<Self>, &'static str> where Self: Sized {
        Self::new(path).map(|e| Box::new(e))
    }

    /// Prepare engine (create tables if needed)
    fn prepare(&self) -> Result<(), &'static str>;
}


pub struct DB {
    kind: String,
    engine: Box<dyn DbEngine>,
}

impl DB {
    fn new_engine(kind: &String, path: &String) -> Result<Box<dyn DbEngine>, &'static str> {
        match kind.as_str() {
            DB_SQLITE => Ok(SqliteEngine::new_boxed(path)?),
            DB_MEMORY => Ok(MemoryEngine::new_boxed(path)?),
            DB_FILE   => Ok(FileEngine::new_boxed(path)?),
            _ => Err(UNKNOWN_DB_TYPE_ERROR)
        }
    }

    pub fn new(typ: &String, path: &String) -> Result<DB, &'static str> {
        Ok(DB{kind: typ.clone(), engine: Self::new_engine(typ, path)?})
    }

    pub fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> Result<(), &'static str> {
        self.engine.insert(id, msg)
    }

    pub fn select(&mut self, id: &String) -> Result<String, &'static str> {
        let mut r = self.engine.get(id)?;
        let expired = r.expired();
        let data = r.data.clone();
        if r.max_clicks == 1 || expired {
            self.engine.delete(id)?;
            if expired {
                return Err(NOT_FOUND_ERROR);
            }
        } else {
            r.max_clicks -= 1;
            self.engine.update(r)?;
        }
        return Ok(data);
    }

    pub fn prepare(&self) -> Result<(), &'static str> {
        self.engine.prepare()
    }

    pub fn get_kind(&self) -> &String { &self.kind }
}


struct SqliteEngine {
    connection: ConnectionWithFullMutex,
}

struct MemoryEngine {
    map: HashMap<String, Record>,
}

struct FileEngine {
    dir_path: String
}


impl DbEngine for MemoryEngine {
    fn new(_path: &String) -> Result<Self, &'static str> {
        Ok(MemoryEngine { map: HashMap::new() })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> Result<(), &'static str> {
        match self.map.insert(id.clone(), Record::new(id, msg)) {
            None => Ok(()),
            Some(_) => Err(ALREADY_EXISTS_ERROR)
        }
    }
    fn delete(&mut self, id: &String) -> Result<(), &'static str> {
        self.map.remove(id).map(|_| ()).ok_or(DELETE_ERROR)
    }
    fn get(&self, id: &String) -> Result<Record, &'static str> {
        match self.map.get(id) {
            Some(v) => Ok(v.clone()),
            None => Err(NOT_FOUND_ERROR)
        }
    }
    fn update(&mut self, r: Record) -> Result<(), &'static str> {
        let id = r.id.clone();
        if !self.map.contains_key(&id) {
            return Err(NOT_FOUND_ERROR);
        }
        self.map.entry(id).and_modify(|rec| rec.max_clicks = r.max_clicks );
        Ok(())
    }
    fn prepare(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

impl DbEngine for SqliteEngine {
    fn new(path: &String) -> Result<Self, &'static str> {
        Ok(
            SqliteEngine {
                connection: Connection::open_with_full_mutex(
                    path.as_str()
                ).map_err(|e| {
                    error!("[DB] Cannot connect to sql database: {}", e);
                    "connection error"
                })?
            }
        )
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> Result<(), &'static str> {
        let mut stmt = self.prepare_statement(INSERT_SQLITE_QUERY)?;

        stmt.bind::<&[(_, Value)]>(&[
            (":id",         id.as_str().into()),
            (":data",       msg.get_data().as_str().into()),
            (":max_clicks", (msg.get_max_clicks() as i64).into()),
            (":created",    now().into()),
            (":lifetime",   (msg.get_lifetime() as i64).into()),
        ][..]).map_err(|_| BIND_ERROR )?;

        self.check_ok(&mut stmt)
    }
    fn delete(&mut self, id: &String) -> Result<(), &'static str> {
        let mut del_stmt = self.prepare_statement(DELETE_BY_ID_SQLITE_QUERY)?;

        del_stmt.bind::<&[(_, Value)]>(&[
            (":id", id.as_str().into())
        ][..]).map_err(|_| BIND_ERROR )?;

        self.check_ok(&mut del_stmt)
    }
    fn get(&self, id: &String) -> Result<Record, &'static str> {
        let mut stmt = self.prepare_statement(SELECT_BY_ID_SQLITE_QUERY)?;

        stmt.bind::<&[(_, Value)]>(&[
            (":id", id.as_str().into())
        ][..]).map_err(|_| BIND_ERROR )?;

        while let Ok(State::Row) = stmt.next() {
            let rid = self.read_column::<String>(&stmt, "id")?;
            let msg = self.read_column::<String>(&stmt, "data")?;
            let max_clicks = self.read_column::<i64>(&stmt, "max_clicks")? as u32;
            let created = self.read_column::<i64>(&stmt, "created")?;
            let lifetime = self.read_column::<i64>(&stmt, "lifetime")? as u64;

            return Ok(Record{
                id: rid, data: msg, max_clicks, created, lifetime
            });
        }
        Err(NOT_FOUND_ERROR)
    }
    fn update(&mut self, r: Record) -> Result<(), &'static str> {
        let mut upd_stmt = self.prepare_statement(UPDATE_BY_ID_SQLITE_QUERY)?;

        upd_stmt.bind::<&[(_, Value)]>(&[
            (":max_clicks", (r.max_clicks as i64).into()),
            (":id",         r.id.as_str().into()),
        ][..]).map_err(|_| BIND_ERROR )?;

        self.check_ok(&mut upd_stmt)
    }
    fn prepare(&self) -> Result<(), &'static str> {
        self.connection.execute(PREPARE_DB_SQLITE_QUERY).map_err(
            |_| SQLITE_CREATE_TABLE_ERROR
        )
    }
}

impl DbEngine for FileEngine {
    fn new(path: &String) -> Result<Self, &'static str> {
        Ok(FileEngine { dir_path: path.clone() })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> Result<(), &'static str> {
        let filepath = self.get_filepath(id);
        if self.file_exists(&filepath) {
            return Err(ALREADY_EXISTS_ERROR);
        }

        serde_json::to_writer(
            OpenOptions::new().write(true).create(true).open(filepath).map_err(|_|"open file error")?,
            &Record::new(id, msg)
        ).map_err(|_| IO_WRITE_ERROR)
    }
    fn delete(&mut self, id: &String) -> Result<(), &'static str> {
        let filepath = self.get_filepath(id);
        if !self.file_exists(&filepath) {
            return Err(DO_NOT_EXISTS_ERROR);
        }

        std::fs::remove_file(filepath).map_err(|_| DELETE_ERROR)
    }
    fn get(&self, id: &String) -> Result<Record, &'static str> {
        let filepath = self.get_filepath(id);
        if !self.file_exists(&filepath) {
            return Err(NOT_FOUND_ERROR);
        }

        serde_json::from_reader::<_, Record>(
            OpenOptions::new().read(true).open(filepath).map_err(|_|"open file error")?
        ).map_err(|_| IO_READ_ERROR)
    }
    fn update(&mut self, r: Record)-> Result<(), &'static str> {
        let filepath = self.get_filepath(&r.id);
        if !self.file_exists(&filepath) {
            return Err(NOT_FOUND_ERROR);
        }

        let mut record = serde_json::from_reader::<_, Record>(
            OpenOptions::new().read(true).open(filepath.clone()).map_err(|_|"open file error")?
        ).map_err(|_| IO_READ_ERROR)?;

        record.max_clicks = r.max_clicks;

        serde_json::to_writer(
            OpenOptions::new().write(true).open(filepath.clone()).map_err(|_|"open file error")?,
            &record
        ).map_err(|_| IO_WRITE_ERROR)
    }
    fn prepare(&self) -> Result<(), &'static str> {
        if !self.file_exists(&self.dir_path) {
            std::fs::create_dir(self.dir_path.clone()).map_err(|_| IO_CREATE_ERROR)?;
        }
        Ok(())
    }
}


impl SqliteEngine {
    fn prepare_statement(&self, query: &str) -> Result<Statement<'_>, &'static str> {
        self.connection.prepare(query).or_else(|e| -> Result<Statement<'_>, &'static str> {
            error!("[DB] Error while preparing query `{}`: {}", query, e);
            Err(SQLITE_ERROR)
        })
    }
    fn read_column<T: ReadableWithIndex>(&self, stmt: &Statement, column: &str) -> Result<T, &'static str> {
        stmt.read::<T, _>(column).or_else(|e| -> Result<T, &'static str>{
            error!("[DB] Error while getting value from column `{}`: {}", column, e);
            Err(SQLITE_ERROR)
        })
    }
    fn check_ok(&self, stmt: &mut Statement) -> Result<(), &'static str> {
        stmt.next().map(|_| ()).or_else(|e| -> Result<(), &'static str>{
            error!("[DB] Error while executing SQL: {}", e);
            Err(SQLITE_ERROR)
        })
    }
}

impl FileEngine {
    fn get_filepath(&self, id: &String) -> String {
        format!("{}/{}", self.dir_path, id)
    }
    fn file_exists(&self, filepath: &String) -> bool {
        std::path::Path::new(filepath.as_str()).exists()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    id: String,
    data: String,
    max_clicks: u32,
    created: i64,
    lifetime: u64,
}

impl Record {
    pub fn new(id: &String, msg: &ApiAddRequest) -> Self {
        Record{
            id: id.clone(),
            data: msg.get_data().clone(),
            max_clicks: msg.get_max_clicks(),
            created: now(),
            lifetime: msg.get_lifetime(),
        }
    }
    fn expired(&self) -> bool {
        now() - self.created > (self.lifetime as i64)
    }
}
