use rusqlite::Result;


use crate::model::model::{Sector};
use crate::dao::sector_dao::SectorDao;

pub struct SectorService {
    dao: SectorDao
}

impl SectorService {
    pub fn new() -> Result<SectorService, String> {
        let result = SectorDao::new();
        match result {
            Ok(work_dao) => {
                Ok(Self {
                    dao: work_dao
                })
            }
            Err(err) => { Err(err) }
        }
    }
    pub fn word_list(&self) -> Result<Vec<Sector>, String> {
        self.dao.list()
    }


    pub fn add(&self, title: &String, content: &String) -> Result<(), String> {
        let result = self.dao.add(title, content);
        match result {
            Ok(_updated) => { Ok(()) }
            Err(err) => { Err(err.to_string()) }
        }
    }
}