use super::super::engine::Transaction;
use super::super::types::expression::{Environment, Expression};
use super::{Context, Effect, Executor, ResultSet};
use crate::Error;
use std::collections::BTreeMap;

/// An UPDATE executor
pub struct Update<T: Transaction> {
    /// The table to update
    table: String,
    /// The source of rows to update
    source: Box<dyn Executor<T>>,
    /// The expressions to update columns with
    /// FIXME Uses BTreeMap instead of HashMap for test stability
    expressions: BTreeMap<String, Expression>,
}

impl<T: Transaction> Update<T> {
    pub fn new(
        table: String,
        source: Box<dyn Executor<T>>,
        expressions: BTreeMap<String, Expression>,
    ) -> Box<Self> {
        Box::new(Self { table, source, expressions })
    }
}

impl<T: Transaction> Executor<T> for Update<T> {
    fn execute(self: Box<Self>, ctx: &mut Context<T>) -> Result<ResultSet, Error> {
        let mut source = self.source.execute(ctx)?;
        let table = ctx.txn.must_read_table(&self.table)?;
        let mut count = 0;
        while let Some(mut row) = source.next().transpose()? {
            let id = table.get_row_key(&row)?;
            let env = Environment::new(table.make_row_hashmap(row.clone()));
            for (field, expr) in &self.expressions {
                table.set_row_field(&mut row, field, expr.evaluate(&env)?)?;
            }
            ctx.txn.update(&table.name, &id, row)?;
            count += 1
        }
        Ok(ResultSet::from_effect(Effect::Update { count }))
    }
}
