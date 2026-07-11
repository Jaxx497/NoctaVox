use anyhow::Result;
use rusqlite::params;

use crate::{
    Database,
    database::queries::{GET_SESSION_PREFIX, SET_SESSION_STATE},
};

impl Database {
    pub fn save_snapshot(&mut self, snapshot: Vec<(&'static str, String)>) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(SET_SESSION_STATE)?;
            for (key, value) in snapshot {
                stmt.execute(params![key, value])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_snapshot(&mut self, prefix: &'static str) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(GET_SESSION_PREFIX)?;

        Ok(stmt
            .query_map(params![prefix], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(Result::ok)
            .collect())
    }
}
