use sqlite;


pub trait DB: Sync + Send {
    fn insert(&self, id: &String, msg: Vec<u8>) -> Result<bool, &'static str>;
    fn select(&self, id: &String) -> Result<String, &'static str>;
    fn prepare(&self);
    fn create(path: &str) -> Self where Self: Sized;
}

pub struct SqliteDB {
    connection: sqlite::ConnectionWithFullMutex,
}

impl DB for SqliteDB {
    fn select(&self, id: &String) -> Result<String, &'static str> {
        let query = "SELECT * FROM msg WHERE id = ? LIMIT 1";
        let mut statement = match self.connection.prepare(query) {
            Ok(s) => s,
            Err(e) => {
                error!("[DB] Error while preparing select: {}", e);
                return Err("sqlite error");
            }
        };
        statement.bind((1, id.as_str())).unwrap();

        while let Ok(sqlite::State::Row) = statement.next() {
            let msg = match statement.read::<String, _>("data") {
                Ok(msg) => msg,
                Err(e) => {
                    error!("[DB] Error while doing select: {}", e);
                    return Err("sqlite error");
                }
            };

            let query = "DELETE FROM msg WHERE id = ?";
            let mut del_statement = match self.connection.prepare(query) {
                Ok(s) => s,
                Err(e) => {
                    error!("[DB] Error while preparing delete: {}", e);
                    return Err("sqlite error");
                }
            };

            del_statement.bind((1, id.as_str())).unwrap();
            match del_statement.next() {
                Ok(_) => {},
                Err(e) => {
                    error!("[DB] Error while doing delete: {}", e);
                    return Err("sqlite error");
                }
            }

            return Ok(msg);
        }
        Err("not_found")
    }

    fn insert(&self, id: &String, msg: Vec<u8>) -> Result<bool, &'static str> {
        let query = "INSERT INTO msg VALUES (?, ?)";
        let mut statement = match self.connection.prepare(query) {
            Ok(s) => s,
            Err(e) => {
                error!("[DB] Error while creating insert statement: {}", e);
                return Err("sqlite error");
            }
        };
        statement.bind((1, id.as_str())).unwrap();
        statement.bind((2, String::from_utf8(msg).unwrap().as_str())).unwrap();
        match statement.next() {
            Ok(_) => {},
            Err(e) => {
                error!("[DB] Error while doing insert: {}", e);
                return Err("sqlite error")
            }
        }
        Ok(true)
    }

    fn prepare(&self) {
        let query = "CREATE TABLE IF NOT EXISTS msg (id TEXT, data TEXT);";
        self.connection.execute(query).unwrap();
    }

    fn create(path: &str) -> SqliteDB {
        SqliteDB{
            connection: sqlite::Connection::open_with_full_mutex(path).unwrap(),
        }
    }
}
