use std::collections::HashMap;
use std::fmt::Display;
use std::fs::OpenOptions;

use mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};
use sqlite::{Connection, ConnectionWithFullMutex, ReadableWithIndex, State, Statement, Value};
use mysql::{Pool, PooledConn, params};

use crate::api::ApiAddRequest;
use crate::utils::now;


const DB_SQLITE: &str = "sqlite";
const DB_MEMORY: &str = "memory";
const DB_FILE: &str = "file";
const DB_MYSQL: &str = "mysql";

const PREPARE_DB_SQL_QUERY: &str = "CREATE TABLE IF NOT EXISTS msg (id TEXT NOT NULL, data TEXT, max_clicks INT NOT NULL, created INT NOT NULL, lifetime INT NOT NULL);";
const SELECT_BY_ID_SQL_QUERY: &str = "SELECT * FROM msg WHERE id = :id LIMIT 1";
const DELETE_BY_ID_SQL_QUERY: &str = "DELETE FROM msg WHERE id = :id";
const UPDATE_BY_ID_SQL_QUERY: &str = "UPDATE msg SET max_clicks = :max_clicks WHERE id = :id";
const INSERT_SQL_QUERY: &str = "INSERT INTO msg (id, data, max_clicks, created, lifetime) VALUES (:id, :data, :max_clicks, :created, :lifetime)";

pub const SQLITE_ERROR: &str = "sqlite error";
pub const MYSQL_ERROR: &str = "mysql error";

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
    fn get(&mut self, id: &String) -> Result<Record, &'static str>;

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
    fn prepare(&mut self) -> Result<(), &'static str>;
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
            DB_MYSQL  => Ok(MysqlEngine::new_boxed(path)?),
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

    pub fn prepare(&mut self) -> Result<(), &'static str> {
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
    dir_path: String,
}

struct MysqlEngine {
    connection: PooledConn,
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
        self.map.remove(id).map(|_| ()).ok_or(DELETE_ERROR).map_err(Self::report)
    }
    fn get(&mut self, id: &String) -> Result<Record, &'static str> {
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
    fn prepare(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
}

impl DbEngine for SqliteEngine {
    fn new(path: &String) -> Result<Self, &'static str> {
        Ok(SqliteEngine {
            connection: Connection::open_with_full_mutex(path.as_str()).map_err(Self::report)?
        })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> Result<(), &'static str> {
        let mut stmt = self.prepare_statement(INSERT_SQL_QUERY)?;

        stmt.bind::<&[(_, Value)]>(&[
            (":id",         id.as_str().into()),
            (":data",       msg.get_data().as_str().into()),
            (":max_clicks", (msg.get_max_clicks() as i64).into()),
            (":created",    now().into()),
            (":lifetime",   (msg.get_lifetime() as i64).into()),
        ][..]).map_err(Self::report)?;

        self.check_ok(&mut stmt)
    }
    fn delete(&mut self, id: &String) -> Result<(), &'static str> {
        let mut del_stmt = self.prepare_statement(DELETE_BY_ID_SQL_QUERY)?;

        del_stmt.bind::<&[(_, Value)]>(&[
            (":id", id.as_str().into())
        ][..]).map_err(Self::report)?;

        self.check_ok(&mut del_stmt)
    }
    fn get(&mut self, id: &String) -> Result<Record, &'static str> {
        let mut stmt = self.prepare_statement(SELECT_BY_ID_SQL_QUERY)?;

        stmt.bind::<&[(_, Value)]>(&[
            (":id", id.as_str().into())
        ][..]).map_err(Self::report)?;

        while let Ok(State::Row) = stmt.next() {
            let rid = self.read_column::<String>(&stmt, "id")?;
            let msg: String = self.read_column::<String>(&stmt, "data")?;
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
        let mut upd_stmt = self.prepare_statement(UPDATE_BY_ID_SQL_QUERY)?;

        upd_stmt.bind::<&[(_, Value)]>(&[
            (":max_clicks", (r.max_clicks as i64).into()),
            (":id",         r.id.as_str().into()),
        ][..]).map_err(Self::report)?;

        self.check_ok(&mut upd_stmt)
    }
    fn prepare(&mut self) -> Result<(), &'static str> {
        self.connection.execute(PREPARE_DB_SQL_QUERY).map_err(Self::report)
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
            OpenOptions::new().write(true).create(true).open(filepath).map_err(Self::report)?,
            &Record::new(id, msg)
        ).map_err(Self::report)
    }
    fn delete(&mut self, id: &String) -> Result<(), &'static str> {
        let filepath = self.get_filepath(id);
        if !self.file_exists(&filepath) {
            return Err(DO_NOT_EXISTS_ERROR);
        }

        std::fs::remove_file(filepath).map_err(Self::report)
    }
    fn get(&mut self, id: &String) -> Result<Record, &'static str> {
        let filepath = self.get_filepath(id);
        if !self.file_exists(&filepath) {
            return Err(NOT_FOUND_ERROR);
        }

        serde_json::from_reader::<_, Record>(
            OpenOptions::new().read(true).open(filepath).map_err(Self::report)?
        ).map_err(Self::report)
    }
    fn update(&mut self, r: Record)-> Result<(), &'static str> {
        let filepath = self.get_filepath(&r.id);
        if !self.file_exists(&filepath) {
            return Err(NOT_FOUND_ERROR);
        }

        let mut record = serde_json::from_reader::<_, Record>(
            OpenOptions::new().read(true).open(filepath.clone()).map_err(Self::report)?
        ).map_err(Self::report)?;

        record.max_clicks = r.max_clicks;

        serde_json::to_writer(
            OpenOptions::new().write(true).open(filepath.clone()).map_err(Self::report)?,
            &record
        ).map_err(Self::report)
    }
    fn prepare(&mut self) -> Result<(), &'static str> {
        if !self.file_exists(&self.dir_path) {
            std::fs::create_dir(self.dir_path.clone()).map_err(Self::report)?;
        }
        Ok(())
    }
}

impl DbEngine for MysqlEngine {
    fn new(path: &String) -> Result<Self, &'static str> {
        let pool = Pool::new(path.as_str()).map_err(Self::report)?;
        Ok(MysqlEngine{
            connection: pool.get_conn().map_err(Self::report)?
        })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> Result<(), &'static str> {
        self.connection.exec_drop(
            INSERT_SQL_QUERY,
            params!{
                "id"=>id,
                "data"=>msg.get_data(),
                "max_clicks"=>msg.get_max_clicks(),
                "created" => now(),
                "lifetime" => msg.get_lifetime(),
            },
        ).map_err(Self::report)
    }
    fn delete(&mut self, id: &String) -> Result<(), &'static str> {
        self.connection.exec_drop(
            DELETE_BY_ID_SQL_QUERY,
            params!{
                "id" => id,
            }
        ).map_err(Self::report)
    }
    fn get(&mut self, id: &String) -> Result<Record, &'static str> {
        let result = self.connection.exec_map(
            SELECT_BY_ID_SQL_QUERY,
            params!{
                "id" => id,
            },
            |(id, data, max_clicks, created, lifetime)| Record{
                id, data, max_clicks, created, lifetime
            }
        ).map_err(Self::report)?;
        assert!(result.len() <= 1);
        match &result[..] {
            [first] => Ok(first.to_owned()),
            _ => Err(NOT_FOUND_ERROR)
        }
    }
    fn update(&mut self, r: Record) -> Result<(), &'static str> {
        self.connection.exec_drop(
            UPDATE_BY_ID_SQL_QUERY,
            params!{
                "id" => r.id,
                "max_clicks" => r.max_clicks,
            },
        ).map_err(Self::report)
    }
    fn prepare(&mut self) -> Result<(), &'static str> {
        self.connection.query_drop(PREPARE_DB_SQL_QUERY).map_err(Self::report)
    }
}


impl SqliteEngine {
    fn prepare_statement(&self, query: &str) -> Result<Statement<'_>, &'static str> {
        self.connection.prepare(query).map_err(Self::report)
    }
    fn read_column<T: ReadableWithIndex>(&self, stmt: &Statement, column: &str) -> Result<T, &'static str> {
        stmt.read::<T, _>(column).map_err(Self::report)
    }
    fn check_ok(&self, stmt: &mut Statement) -> Result<(), &'static str> {
        stmt.next().map(|_| ()).map_err(Self::report)
    }
    fn report(e: sqlite::Error) -> &'static str {
        error!("[DB] SQLite error: {}", e);
        SQLITE_ERROR
    }
}

impl FileEngine {
    fn get_filepath(&self, id: &String) -> String {
        format!("{}/{}", self.dir_path, id)
    }
    fn file_exists(&self, filepath: &String) -> bool {
        std::path::Path::new(filepath.as_str()).exists()
    }
    fn report<E: Display>(e: E) -> &'static str {
        error!("[DB] IO error: {}", e);
        SQLITE_ERROR
    }
}

impl MysqlEngine {
    fn report(e: mysql::Error) -> &'static str {
        error!("[DB] MySql error: {}", e);
        MYSQL_ERROR
    }
}

impl MemoryEngine {
    fn report<E: Display>(e: E) -> &'static str {
        error!("[DB] Memory error: {}", e);
        SQLITE_ERROR
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
