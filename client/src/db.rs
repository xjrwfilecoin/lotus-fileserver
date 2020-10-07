use r2d2::{Pool,PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::ToSql;
use rusqlite::Statement;
use rusqlite::Result;

const DB_PATH: &'static str = "./client/doudou";

lazy_static! {

   // static ref DB_CONN:Mutex<Connection>=Mutex::new(Connection::open(path::Path::new("./actixWebSample/doudou")).unwrap());
    static ref  POOL:Pool<SqliteConnectionManager> = r2d2::Pool::new(SqliteConnectionManager::file(DB_PATH)).unwrap();

}

pub struct Db {
    conn: PooledConnection<SqliteConnectionManager>
}

impl Db {
    pub fn new() -> Result<Db, String> {
        // let POOL: Pool<SqliteConnectionManager> = r2d2::Pool::new(SqliteConnectionManager::file("./actixWebSample/doudou")).unwrap();
        let result_conn = POOL.get();
        match result_conn {
            Ok(connection) => {
                Ok(Self {
                    conn: connection
                })
            }
            Err(_err) => { Err(String::from(" db pool get connect fail")) }
        }
    }

    pub fn list<T>(&self, sql: &str, func: fn(stmt: Statement) -> Result<Vec<T>, String>) -> Result<Vec<T>, String> {
        let result_stmt = self.conn.prepare(sql);
        match result_stmt {
            Ok(stmt) => {
                let inner = func(stmt);
                match inner {
                    Ok(res) => { Ok(res) }
                    Err(err) => { Err(err) }
                }
            }
            Err(err) => { Err(err.to_string()) }
        }
    }

    pub fn execute<P>(&self, sql: &str, param: P) -> Result<usize> where P: IntoIterator, P::Item: ToSql {
        self.conn.execute(sql, param)
    }
}
