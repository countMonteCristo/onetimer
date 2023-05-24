use std::collections::HashMap;
use std::fmt::Display;
use std::fs::OpenOptions;

use mysql::{params, prelude::Queryable};
use postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use sqlite::{ReadableWithIndex, State, Statement, Value};

use crate::api::ApiAddRequest;
use crate::logger::get_reporter;
use crate::utils::{now, ErrorStr, Result, ResultV};


const MODULE: &str = "DB";

const DB_SQLITE: &str = "sqlite";
const DB_MEMORY: &str = "memory";
const DB_FILE: &str = "file";
const DB_MYSQL: &str = "mysql";
const DB_PGSQL: &str = "postgresql";

const PREPARE_DB_SQL_QUERY: &str = "CREATE TABLE IF NOT EXISTS msg (id TEXT NOT NULL, data TEXT, max_clicks BIGINT NOT NULL, created BIGINT NOT NULL, lifetime BIGINT NOT NULL);";
const SELECT_BY_ID_SQL_QUERY: &str = "SELECT * FROM msg WHERE id = :id LIMIT 1";
const DELETE_BY_ID_SQL_QUERY: &str = "DELETE FROM msg WHERE id = :id";
const UPDATE_BY_ID_SQL_QUERY: &str = "UPDATE msg SET max_clicks = :max_clicks WHERE id = :id";
const INSERT_SQL_QUERY: &str = "INSERT INTO msg (id, data, max_clicks, created, lifetime) VALUES (:id, :data, :max_clicks, :created, :lifetime)";

const DELETE_BY_ID_PGSQL_QUERY: &str = "DELETE FROM msg WHERE id = $1";
const INSERT_PGSQL_QUERY: &str = "INSERT INTO msg (id, data, max_clicks, created, lifetime) VALUES ($1, $2, $3, $4, $5)";
const SELECT_BY_ID_PGSQL_QUERY: &str = "SELECT * FROM msg WHERE id = $1 LIMIT 1";
const UPDATE_BY_ID_PGSQL_QUERY: &str = "UPDATE msg SET max_clicks = $1 WHERE id = $2";

pub const SQLITE_ERROR: &str = "sqlite error";
pub const MYSQL_ERROR: &str = "mysql error";
pub const MEMORY_ERROR: &str = "memory error";
pub const IO_ERROR: &str = "io error";
pub const PGSQL_ERROR: &str = "pgsql error";

pub const NOT_FOUND_ERROR: &str = "not found";
pub const UNKNOWN_DB_TYPE_ERROR: &str = "unknown db kind";
pub const ALREADY_EXISTS_ERROR: &str = "already exists";
pub const DO_NOT_EXISTS_ERROR: &str = "do not exists";
pub const DELETE_ERROR: &str = "delete error";


pub trait DbEngine: Sync + Send {
    /// Insert data from ApiAddRequest with given id to database
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> ResultV;

    /// Get record from database by id
    fn get(&mut self, id: &String) -> Result<Record>;

    /// Delete record from database by id
    fn delete(&mut self, id: &String)-> ResultV;

    /// Update record in database
    fn update(&mut self, r: Record)-> ResultV;

    /// Create new instance of engine
    fn new(path: &String) -> Result<Self> where Self: Sized;

    /// Create new instance of engine in the heap
    fn new_boxed(path: &String) -> Result<Box<Self>> where Self: Sized {
        Self::new(path).map(|e| Box::new(e))
    }

    /// Prepare engine (create tables if needed)
    fn prepare(&mut self) -> ResultV;
}

trait Reportable {
    fn report(e: impl Display) -> ErrorStr;
}

pub struct DB {
    kind: String,
    engine: Box<dyn DbEngine>,
}

impl DB {
    fn new_engine(kind: &String, path: &String) -> Result<Box<dyn DbEngine>> {
        match kind.as_str() {
            DB_SQLITE => Ok(SqliteEngine::new_boxed(path)?),
            DB_MEMORY => Ok(MemoryEngine::new_boxed(path)?),
            DB_FILE   => Ok(FileEngine::new_boxed(path)?),
            DB_MYSQL  => Ok(MysqlEngine::new_boxed(path)?),
            DB_PGSQL  => Ok(PostgresqlEngine::new_boxed(path)?),
            _ => {
                error!("[{}] Unknown database kind: {}", MODULE, kind);
                Err(UNKNOWN_DB_TYPE_ERROR)
            }
        }
    }
    pub fn new(typ: &String, path: &String) -> Result<DB> {
        Ok(DB{kind: typ.clone(), engine: Self::new_engine(typ, path)?})
    }
    pub fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> ResultV {
        self.engine.insert(id, msg)
    }
    pub fn select(&mut self, id: &String) -> Result<String> {
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
    pub fn prepare(&mut self) -> ResultV {
        let connected = self.engine.prepare();
        if connected.is_ok() {
            info!("[{}] Connected successfully to `{}` backend", MODULE, self.kind);
        } else {
            error!("[{}] Connection to `{}` backend failed", MODULE, self.kind);
        }
        connected
    }
    pub fn get_kind(&self) -> &String { &self.kind }
}


struct SqliteEngine {
    connection: sqlite::ConnectionWithFullMutex,
}
struct MemoryEngine {
    map: HashMap<String, Record>,
}
struct FileEngine {
    dir_path: String,
}
struct MysqlEngine {
    connection: mysql::PooledConn,
}
struct PostgresqlEngine {
    pool: r2d2::Pool<PostgresConnectionManager<NoTls>>,
}


impl Reportable for SqliteEngine {
    fn report(e: impl Display) -> ErrorStr {
        get_reporter(MODULE, "SQLite", SQLITE_ERROR)(e)
    }
}
impl Reportable for MemoryEngine {
    fn report(e: impl Display) -> ErrorStr {
        get_reporter(MODULE, "Memory", MEMORY_ERROR)(e)
    }
}
impl Reportable for FileEngine {
    fn report(e: impl Display) -> ErrorStr {
        get_reporter(MODULE, "IO", IO_ERROR)(e)
    }
}
impl Reportable for MysqlEngine {
    fn report(e: impl Display) -> ErrorStr {
        get_reporter(MODULE, "MySQL", MYSQL_ERROR)(e)
    }
}
impl Reportable for PostgresqlEngine {
    fn report(e: impl Display) -> ErrorStr {
        get_reporter(MODULE, "PostgreSQL", PGSQL_ERROR)(e)
    }
}


impl DbEngine for MemoryEngine {
    fn new(_path: &String) -> Result<Self> {
        Ok(MemoryEngine { map: HashMap::new() })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> ResultV {
        match self.map.insert(id.clone(), Record::new(id, msg)) {
            None => Ok(()),
            Some(_) => {
                // TODO: add logging here
                Err(ALREADY_EXISTS_ERROR)
            }
        }
    }
    fn delete(&mut self, id: &String) -> ResultV {
        self.map.remove(id).map(|_| ()).ok_or(DELETE_ERROR).map_err(Self::report)
    }
    fn get(&mut self, id: &String) -> Result<Record> {
        match self.map.get(id) {
            Some(v) => Ok(v.clone()),
            None => Err(NOT_FOUND_ERROR)
        }
    }
    fn update(&mut self, r: Record) -> ResultV {
        let id = r.id.clone();
        if !self.map.contains_key(&id) {
            return Err(NOT_FOUND_ERROR);
        }
        self.map.entry(id).and_modify(|rec| rec.max_clicks = r.max_clicks );
        Ok(())
    }
    fn prepare(&mut self) -> ResultV {
        Ok(())
    }

}
impl DbEngine for SqliteEngine {
    fn new(path: &String) -> Result<Self> {
        Ok(SqliteEngine {
            connection: sqlite::Connection::open_with_full_mutex(path.as_str()).map_err(Self::report)?,
        })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> ResultV {
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
    fn delete(&mut self, id: &String) -> ResultV {
        let mut del_stmt = self.prepare_statement(DELETE_BY_ID_SQL_QUERY)?;

        del_stmt.bind::<&[(_, Value)]>(&[
            (":id", id.as_str().into())
        ][..]).map_err(Self::report)?;

        self.check_ok(&mut del_stmt)
    }
    fn get(&mut self, id: &String) -> Result<Record> {
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
    fn update(&mut self, r: Record) -> ResultV {
        let mut upd_stmt = self.prepare_statement(UPDATE_BY_ID_SQL_QUERY)?;

        upd_stmt.bind::<&[(_, Value)]>(&[
            (":max_clicks", (r.max_clicks as i64).into()),
            (":id",         r.id.as_str().into()),
        ][..]).map_err(Self::report)?;

        self.check_ok(&mut upd_stmt)
    }
    fn prepare(&mut self) -> ResultV {
        self.connection.execute(PREPARE_DB_SQL_QUERY).map_err(Self::report)
    }
}
impl DbEngine for FileEngine {
    fn new(path: &String) -> Result<Self> {
        Ok(FileEngine { dir_path: path.clone() })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> ResultV {
        let filepath = self.get_filepath(id);
        if self.file_exists(&filepath) {
            // TODO: add logging here
            return Err(ALREADY_EXISTS_ERROR);
        }

        serde_json::to_writer(
            OpenOptions::new().write(true).create(true).open(filepath).map_err(Self::report)?,
            &Record::new(id, msg)
        ).map_err(Self::report)
    }
    fn delete(&mut self, id: &String) -> ResultV {
        let filepath = self.get_filepath(id);
        if !self.file_exists(&filepath) {
            // TODO: add logging here
            return Err(DO_NOT_EXISTS_ERROR);
        }

        std::fs::remove_file(filepath).map_err(Self::report)
    }
    fn get(&mut self, id: &String) -> Result<Record> {
        let filepath = self.get_filepath(id);
        if !self.file_exists(&filepath) {
            return Err(NOT_FOUND_ERROR);
        }

        serde_json::from_reader::<_, Record>(
            OpenOptions::new().read(true).open(filepath).map_err(Self::report)?
        ).map_err(Self::report)
    }
    fn update(&mut self, r: Record)-> ResultV {
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
    fn prepare(&mut self) -> ResultV {
        if !self.file_exists(&self.dir_path) {
            std::fs::create_dir(self.dir_path.clone()).map_err(Self::report)?;
        }
        Ok(())
    }
}
impl DbEngine for MysqlEngine {
    fn new(path: &String) -> Result<Self> {
        let pool = mysql::Pool::new(path.as_str()).map_err(Self::report)?;
        Ok(MysqlEngine{
            connection: pool.get_conn().map_err(Self::report)?
        })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> ResultV {
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
    fn delete(&mut self, id: &String) -> ResultV {
        self.connection.exec_drop(
            DELETE_BY_ID_SQL_QUERY,
            params!{
                "id" => id,
            }
        ).map_err(Self::report)
    }
    fn get(&mut self, id: &String) -> Result<Record> {
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
    fn update(&mut self, r: Record) -> ResultV {
        self.connection.exec_drop(
            UPDATE_BY_ID_SQL_QUERY,
            params!{
                "id" => r.id,
                "max_clicks" => r.max_clicks,
            },
        ).map_err(Self::report)
    }
    fn prepare(&mut self) -> ResultV {
        self.connection.query_drop(PREPARE_DB_SQL_QUERY).map_err(Self::report)
    }
}
impl DbEngine for PostgresqlEngine {
    fn new(path: &String) -> Result<Self> {
        let manager = PostgresConnectionManager::new(
            path.parse().map_err(Self::report)?,
            NoTls,
        );
        Ok(PostgresqlEngine{
            pool: r2d2::Pool::new(manager).map_err(Self::report)?,
        })
    }
    fn insert(&mut self, id: &String, msg: &ApiAddRequest) -> ResultV {
        self.client()?.execute(
            INSERT_PGSQL_QUERY,
            &[&id, &msg.get_data(), &(msg.get_max_clicks() as i64), &now(), &(msg.get_lifetime() as i64)]
        ).map(|_| ()).map_err(Self::report)
    }
    fn delete(&mut self, id: &String) -> ResultV {
        self.client()?.execute(DELETE_BY_ID_PGSQL_QUERY, &[id]).map(|_| ()).map_err(Self::report)
    }
    fn get(&mut self, id: &String) -> Result<Record> {
        let result = self.client()?.query(
            SELECT_BY_ID_PGSQL_QUERY,
            &[&id]
        ).map_err(Self::report)?;
        assert!(result.len() <= 1);
        match &result[..] {
            [first] => {
                let lifetime: i64 = first.get("lifetime");
                let clicks: i64 = first.get("max_clicks");
                Ok(Record{
                    id: first.get("id"),
                    data: first.get("data"),
                    max_clicks: clicks as u32,
                    created: first.get("created"),
                    lifetime: lifetime as u64
                })
            },
            _ => Err(NOT_FOUND_ERROR)
        }
    }
    fn update(&mut self, r: Record) -> ResultV {
        self.client()?.execute(
            UPDATE_BY_ID_PGSQL_QUERY,
            &[&(r.max_clicks as i64), &r.id]
        ).map(|_| ()).map_err(Self::report)
    }
    fn prepare(&mut self) -> ResultV {
        self.client()?.batch_execute(PREPARE_DB_SQL_QUERY).map_err(Self::report)
    }
}


impl SqliteEngine {
    fn prepare_statement(&self, query: &str) -> Result<Statement<'_>> {
        self.connection.prepare(query).map_err(Self::report)
    }
    fn read_column<T: ReadableWithIndex>(&self, stmt: &Statement, column: &str) -> Result<T> {
        stmt.read::<T, _>(column).map_err(Self::report)
    }
    fn check_ok(&self, stmt: &mut Statement) -> ResultV {
        stmt.next().map(|_| ()).map_err(Self::report)
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
impl PostgresqlEngine {
    fn client(&mut self) -> Result<r2d2::PooledConnection<PostgresConnectionManager<NoTls>>> {
        self.pool.get().map_err(Self::report)
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
