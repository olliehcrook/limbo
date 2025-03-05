use std::fmt::Debug;

use crate::mvcc::database::{LogRecord, Result};
use crate::mvcc::errors::DatabaseError;

#[derive(Debug)]
pub enum Storage {
    Noop,
}

impl Storage {
    pub fn new_noop() -> Self {
        Self::Noop
    }
}

impl Storage {
    pub fn log_tx(&self, _m: LogRecord) -> Result<()> {
        match self {
            Self::Noop => (),
        }
        Ok(())
    }

    pub fn read_tx_log(&self) -> Result<Vec<LogRecord>> {
        match self {
            Self::Noop => Err(DatabaseError::Io(
                "cannot read from Noop storage".to_string(),
            )),
        }
    }
}
