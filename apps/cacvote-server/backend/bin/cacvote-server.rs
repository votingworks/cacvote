//! `cacvote-server` is the application server for the CACvote voting system. It
//! provides coordination among various types of clients, of which there could
//! be many of each type. There is expected to be a single `cacvote-server`
//! instance.
//!
//! CACvote Server uses Postgres as its database server and SQLx to connect to it.
//! See the README at the repository root for more information on setup.

#![warn(
    clippy::all,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_debug_implementations,
    missing_docs
)]
#![deny(unreachable_pub)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]

use cacvote_server::{app, config::Config, db, log};
use clap::Parser;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = dotenvy::from_filename(".env.local");
    dotenvy::dotenv()?;
    let config = Config::parse();
    log::setup(&config)?;
    let pool = db::setup(&config).await?;

    app::run(app::setup(pool, config.load_ca_cert()?).await, &config).await
}
