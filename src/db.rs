use sqlite::{self, Statement, Value};

use crate::api::ApiAddRequest;


const PREPARE_DB_SQLITE_QUERY: &str = "CREATE TABLE IF NOT EXISTS msg (id TEXT, data TEXT, max_clicks INT);";
const SELECT_BY_ID_SQLITE_QUERY: &str = "SELECT * FROM msg WHERE id = :id LIMIT 1";
const DELETE_BY_ID_SQLITE_QUERY: &str = "DELETE FROM msg WHERE id = :id";
const UPDATE_BY_ID_SQLITE_QUERY: &str = "UPDATE msg SET max_clicks = :max_clicks WHERE id = :id";
const INSERT_SQLITE_QUERY: &str = "INSERT INTO msg (id, data, max_clicks) VALUES (:id, :data, :max_clicks)";

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
        match self.connection.prepare(query) {
            Ok(s) => Ok(s),
            Err(e) => {
                error!("[DB] Error while preparing query `{}`: {}", query, e);
                return Err(SQLITE_ERROR);
            }
        }
    }

    fn read_column<T: sqlite::ReadableWithIndex>(&self, stmt: &Statement, column: &str) -> Result<T, &'static str> {
        match stmt.read::<T, _>(column) {
            Ok(msg) => Ok(msg),
            Err(e) => {
                error!("[DB] Error while getting value from column `{}`: {}", column, e);
                return Err(SQLITE_ERROR);
            }
        }
    }

    fn check_ok(&self, stmt: &mut Statement) -> Result<(), &'static str> {
        match stmt.next() {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[DB] Error while doing delete: {}", e);
                Err(SQLITE_ERROR)
            }
        }
    }
}

impl DB for SqliteDB {
    fn select(&self, id: &String) -> Result<String, &'static str> {

        let mut stmt = self.prepare_statement(SELECT_BY_ID_SQLITE_QUERY)?;
        stmt.bind((":id", id.as_str())).unwrap();

        while let Ok(sqlite::State::Row) = stmt.next() {
            let msg = self.read_column::<String>(&stmt, "data")?;
            let max_clicks = self.read_column::<i64>(&stmt, "max_clicks")?;

            if max_clicks == 1 {
                let mut del_stmt = self.prepare_statement(DELETE_BY_ID_SQLITE_QUERY)?;
                del_stmt.bind((":id", id.as_str())).unwrap();
                self.check_ok(&mut del_stmt)?;
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
        let max_clicks = if msg.max_clicks <= 0 {1} else {msg.max_clicks};

        stmt.bind::<&[(_, Value)]>(&[
            (":id", id.as_str().into()),
            (":data", msg.data.as_str().into()),
            (":max_clicks", max_clicks.into())
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
