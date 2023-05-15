use sqlite::{self, Statement, Value};

use crate::api::ApiAddRequest;
use crate::utils::{one, day_seconds};


const PREPARE_DB_SQLITE_QUERY: &str = "CREATE TABLE IF NOT EXISTS msg (id TEXT NOT NULL, data TEXT, max_clicks INT NOT NULL, created INT NOT NULL, lifetime INT NOT NULL);";
const SELECT_BY_ID_SQLITE_QUERY: &str = "SELECT (strftime('%s', 'now')-created>=lifetime) as expired, * FROM msg WHERE id = :id LIMIT 1";
const DELETE_BY_ID_SQLITE_QUERY: &str = "DELETE FROM msg WHERE id = :id";
const UPDATE_BY_ID_SQLITE_QUERY: &str = "UPDATE msg SET max_clicks = :max_clicks WHERE id = :id";
const INSERT_SQLITE_QUERY: &str = "INSERT INTO msg (id, data, max_clicks, created, lifetime) VALUES (:id, :data, :max_clicks, strftime('%s', 'now'), :lifetime)";

pub const SQLITE_ERROR: &str = "sqlite error";
pub const NOT_FOUND_ERROR: &str = "not found";

pub trait DB: Sync + Send {
    fn insert(&self, id: &String, msg: &ApiAddRequest) -> Result<bool, &'static str>;
    fn select(&self, id: &String) -> Result<String, &'static str>;
    fn prepare(&self);
    fn create(path: &str) -> Self where Self: Sized;
}

pub struct SqliteDB {
    connection: sqlite::ConnectionWithFullMutex,
}

impl SqliteDB {
    fn prepare_statement(&self, query: &str) -> Result<Statement<'_>, &'static str> {
        self.connection.prepare(query).or_else(|e| -> Result<Statement<'_>, &'static str> {
            error!("[DB] Error while preparing query `{}`: {}", query, e);
            Err(SQLITE_ERROR)
        })
    }

    fn read_column<T: sqlite::ReadableWithIndex>(&self, stmt: &Statement, column: &str) -> Result<T, &'static str> {
        stmt.read::<T, _>(column).or_else(|e| -> Result<T, &'static str>{
            error!("[DB] Error while getting value from column `{}`: {}", column, e);
            Err(SQLITE_ERROR)
        })
    }

    fn check_ok(&self, stmt: &mut Statement) -> Result<(), &'static str> {
        stmt.next().map(|_| ()).or_else(|e| -> Result<(), &'static str>{
            error!("[DB] Error while doing delete: {}", e);
            Err(SQLITE_ERROR)
        })
    }
}

impl DB for SqliteDB {
    fn select(&self, id: &String) -> Result<String, &'static str> {

        let mut stmt = self.prepare_statement(SELECT_BY_ID_SQLITE_QUERY)?;
        stmt.bind((":id", id.as_str())).unwrap();

        while let Ok(sqlite::State::Row) = stmt.next() {
            let msg = self.read_column::<String>(&stmt, "data")?;
            let max_clicks = self.read_column::<i64>(&stmt, "max_clicks")?;
            let expired = self.read_column::<i64>(&stmt, "expired")? == 1;

            if max_clicks == 1 || expired {
                let mut del_stmt = self.prepare_statement(DELETE_BY_ID_SQLITE_QUERY)?;
                del_stmt.bind((":id", id.as_str())).unwrap();
                self.check_ok(&mut del_stmt)?;
                if expired {
                    break;
                }
            } else {
                let mut upd_stmt = self.prepare_statement(UPDATE_BY_ID_SQLITE_QUERY)?;
                upd_stmt.bind::<&[(_, Value)]>(&[
                    (":max_clicks", (max_clicks - 1).into()),
                    (":id", id.as_str().into()),
                ][..]).unwrap();
                self.check_ok(&mut upd_stmt)?;
            }

            return Ok(msg);
        }
        Err(NOT_FOUND_ERROR)
    }

    fn insert(&self, id: &String, msg: &ApiAddRequest) -> Result<bool, &'static str> {
        let mut stmt = self.prepare_statement(INSERT_SQLITE_QUERY)?;
        let max_clicks = if msg.max_clicks <= 0 {one()} else {msg.max_clicks};
        let lifetime = if msg.lifetime <= 0 {day_seconds()} else {msg.lifetime};

        stmt.bind::<&[(_, Value)]>(&[
            (":id", id.as_str().into()),
            (":data", msg.data.as_str().into()),
            (":max_clicks", max_clicks.into()),
            (":lifetime", lifetime.into()),
        ][..]).unwrap();

        self.check_ok(&mut stmt)?;
        Ok(true)
    }

    fn prepare(&self) {
        self.connection.execute(PREPARE_DB_SQLITE_QUERY).unwrap();
    }

    fn create(path: &str) -> SqliteDB {
        SqliteDB{
            connection: sqlite::Connection::open_with_full_mutex(path).unwrap(),
        }
    }
}
