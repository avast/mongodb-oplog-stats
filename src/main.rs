//! A tool for obtaining of statistics about a replica-set oplog of a MongoDB
//! instance.

use std::process;

use chrono::Local;
use failure::ResultExt;
use indicatif::ProgressBar;

mod args;
mod mongodb;
mod error;
mod oplog_stats;
mod utils;

use crate::args::parse_args;
use crate::mongodb::MongoDB;
use crate::error::print_error;
use crate::error::Result;
use crate::oplog_stats::OplogStats;

/// Returns the document limit to use for querying the oplog and generating
/// statistics.
///
/// # Arguments
///
/// * `user_limit` - A limit provided by the user.
/// * `mongodb` - MongoDB access to determine the number of documents in the oplog
///    when the user did not specify any limit.
fn limit_to_use(user_limit: Option<u64>, mongodb: &MongoDB) -> Result<u64> {
    let limit = match user_limit {
        Some(limit) => limit,
        None => mongodb
            .get_total_number_of_documents_in_oplog()
            .context("failed to get the number of documents in the oplog")?,
    };
    Ok(limit)
}

/// Obtains statistics about the oplog of the given MongoDB instance and stores
/// them to `oplog_stats`.
///
/// # Arguments
///
/// * `oplog_stats` - Statistics to fill.
/// * `mongodb` - Access to a MongoDB instance.
/// * `limit` - Maximal number of documents to process.
/// * `print_after` - When given, print the statistics after each N processed
///   documents.
fn obtain_oplog_stats(
    oplog_stats: &mut OplogStats,
    mongodb: &MongoDB,
    limit: u64,
    print_after: Option<u64>,
) -> Result<()> {
    let pbar = ProgressBar::new(limit);
    for result in mongodb.generate_documents_in_oplog(limit)? {
        let doc = result.context("failed to get a document from the oplog")?;
        oplog_stats
            .update(&doc)
            .context("failed to add info from an oplog document")?;
        pbar.inc(1);

        let processed_doc_count = oplog_stats.get_processed_doc_count();
        if let Some(print_after) = print_after {
            if processed_doc_count % print_after == 0 {
                println!();
                println!(
                    "Processed {} documents at {}",
                    processed_doc_count,
                    Local::now()
                );
                oplog_stats.print();
                println!();
            }
        }
    }
    pbar.finish();
    Ok(())
}

/// Runs the tool.
fn run() -> Result<()> {
    let args = parse_args().context("failed to parse program argument")?;
    let mongodb = MongoDB::from_args(&args)?;

    let limit = limit_to_use(args.limit, &mongodb)?;
    println!("Obtaining stats (limit: {})...", limit);

    let mut oplog_stats = OplogStats::new();
    match obtain_oplog_stats(&mut oplog_stats, &mongodb, limit, args.print_after) {
        Ok(_) => {
            println!(
                "Final stats after processing {} documents:",
                oplog_stats.get_processed_doc_count()
            );
            oplog_stats.print();
            Ok(())
        }
        Err(err) => {
            if oplog_stats.processed_at_least_one_doc() {
                println!("Obtaining failed; showing last stats:");
                oplog_stats.print();
            }
            Err(err)
        }
    }
}

/// The entry-point of the tool.
fn main() {
    if let Err(err) = run() {
        print_error(&err);
        process::exit(1);
    }
}
