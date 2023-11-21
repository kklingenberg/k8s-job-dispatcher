//! Provides a wrapper around jaq to operate on JSON values with jq
//! filters.

use anyhow::{anyhow, Result};
use itertools::Itertools;
pub use jaq_interpret::Filter;
use jaq_interpret::{results::box_once, Ctx, FilterT, Native, ParseCtx, RcIter, RunPtr, Val};
use serde_json::Value;
use std::rc::Rc;
use tracing::warn;

const JQ_EXTENSIONS: &[(&str, usize, RunPtr)] = &[
    ("cuid2", 0, |_, _| {
        box_once(Ok(Val::str(cuid2::create_id())))
    }),
    // env is a standard filter missing from jaq
    ("env", 0, |_, _| {
        box_once(Ok(Val::obj(
            std::env::vars()
                .map(|(k, v)| (Rc::new(k), Val::str(v)))
                .collect(),
        )))
    }),
];

/// Provide native extensions to jaq.
fn jq_extensions() -> impl Iterator<Item = (String, usize, Native)> {
    JQ_EXTENSIONS
        .iter()
        .map(|&(name, arity, f)| (name.to_string(), arity, Native::new(f)))
}

/// Compile a filter.
pub fn compile(filter: &str) -> Result<Filter> {
    let mut defs = ParseCtx::new(Vec::new());
    defs.insert_natives(jaq_core::core());
    defs.insert_natives(jq_extensions());
    defs.insert_defs(jaq_std::std());
    let (f, errs) = jaq_parse::parse(filter, jaq_parse::main());
    if !errs.is_empty() {
        return Err(anyhow!(errs.into_iter().join("; ")));
    }
    let f = defs.compile(f.unwrap());
    if !defs.errs.is_empty() {
        return Err(anyhow!(defs.errs.into_iter().map(|(e, _)| e).join("; ")));
    }
    Ok(f)
}

/// Execute a compiled filter against an input, and produce the first
/// serde_json value.
pub fn first_result(filter: &Filter, input: Value) -> Option<Result<Value>> {
    let inputs = RcIter::new(core::iter::empty());
    let mut outputs = filter
        .run((Ctx::new([], &inputs), Val::from(input)))
        .map(|r| r.map(Value::from).map_err(|e| anyhow!(e.to_string())));
    let first_result = outputs.next();
    if outputs.next().is_some() {
        warn!("Filter returned more than one result; subsequent results are ignored");
    }
    first_result
}
