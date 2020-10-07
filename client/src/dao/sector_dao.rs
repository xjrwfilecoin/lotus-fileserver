use rusqlite::params;
use rusqlite::Statement;

use crate::db::{Db};
use crate::model::model::{Sector};
use rusqlite::Result;

pub struct SectorDao {
    db: Db
}

impl SectorDao {
    pub fn new() -> Result<SectorDao, String> {
        let result = Db::new();
        match result {
            Ok(data_db) => {
                Ok(Self {
                    db: data_db
                })
            }
            Err(err) => { Err(err) }
        }
    }




    pub fn list(&self) -> Result<Vec<Sector>, String> {
        self.db.list("select id, title,content from words", get_list)
    }

    pub fn add(&self, title: &String, content: &String) -> Result<usize> {
        self.db.execute("insert into words (title,content) values(?1,?2)", params![title,content])
    }
}

pub fn get_list(mut stmt: Statement) -> Result<Vec<Sector>, String> {
    let rows = stmt.query_map(params![], |row| {
        Ok(Sector {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
        })
    });
    match rows {
        Ok(rows) => {
            let mut vec: Vec<Sector> = Vec::new();
            for word in rows {
                // todo
                let word1 = word.unwrap();
                vec.push(word1);
                // println!("found person {:?}", person.unwrap())
            }
            Ok(vec)
        }
        Err(err) => { Err(err.to_string()) }
    }
}