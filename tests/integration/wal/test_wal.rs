use crate::common::{do_flush, TempDatabase};
use limbo_core::{Connection, LimboError, Result, StepResult};
use std::cell::RefCell;
use std::rc::Rc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[allow(clippy::arc_with_non_send_sync)]
#[test]
fn test_wal_checkpoint_result() -> Result<()> {
    maybe_setup_tracing();
    let tmp_db = TempDatabase::new("test_wal.db");
    let conn = tmp_db.connect_limbo();
    conn.execute("CREATE TABLE t1 (id text);")?;

    let res = execute_and_get_strings(&tmp_db, &conn, "pragma journal_mode;")?;
    assert_eq!(res, vec!["wal"]);

    conn.execute("insert into t1(id) values (1), (2);")?;
    do_flush(&conn, &tmp_db).unwrap();
    conn.execute("select * from t1;")?;
    do_flush(&conn, &tmp_db).unwrap();

    // checkpoint result should return > 0 num pages now as database has data
    let res = execute_and_get_ints(&tmp_db, &conn, "pragma wal_checkpoint;")?;
    println!("'pragma wal_checkpoint;' returns: {res:?}");
    assert_eq!(res.len(), 3);
    assert_eq!(res[0], 0); // checkpoint successfully
    assert!(res[1] > 0); // num pages in wal
    assert!(res[2] > 0); // num pages checkpointed successfully

    Ok(())
}

fn maybe_setup_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_thread_ids(true),
        )
        .with(EnvFilter::from_default_env())
        .try_init();
}

/// Execute a statement and get strings result
pub(crate) fn execute_and_get_strings(
    tmp_db: &TempDatabase,
    conn: &Rc<Connection>,
    sql: &str,
) -> Result<Vec<String>> {
    let statement = conn.prepare(sql)?;
    let stmt = Rc::new(RefCell::new(statement));
    let mut result = Vec::new();

    let mut stmt = stmt.borrow_mut();
    while let Ok(step_result) = stmt.step() {
        match step_result {
            StepResult::Row => {
                let row = stmt.row().unwrap();
                for el in row.get_values() {
                    result.push(format!("{el}"));
                }
            }
            StepResult::Done => break,
            StepResult::Interrupt => break,
            StepResult::IO => tmp_db.io.run_once()?,
            StepResult::Busy => tmp_db.io.run_once()?,
        }
    }
    Ok(result)
}

/// Execute a statement and get integers
pub(crate) fn execute_and_get_ints(
    tmp_db: &TempDatabase,
    conn: &Rc<Connection>,
    sql: &str,
) -> Result<Vec<i64>> {
    let statement = conn.prepare(sql)?;
    let stmt = Rc::new(RefCell::new(statement));
    let mut result = Vec::new();

    let mut stmt = stmt.borrow_mut();
    while let Ok(step_result) = stmt.step() {
        match step_result {
            StepResult::Row => {
                let row = stmt.row().unwrap();
                for value in row.get_values() {
                    let out = match value {
                        limbo_core::OwnedValue::Integer(i) => i,
                        _ => {
                            return Err(LimboError::ConversionError(format!(
                                "cannot convert {value} to int"
                            )))
                        }
                    };
                    result.push(*out);
                }
            }
            StepResult::Done => break,
            StepResult::Interrupt => break,
            StepResult::IO => tmp_db.io.run_once()?,
            StepResult::Busy => tmp_db.io.run_once()?,
        }
    }
    Ok(result)
}
