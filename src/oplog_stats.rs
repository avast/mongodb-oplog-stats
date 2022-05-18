//! Statistics of a MongoDB oplog.

use std::collections::HashMap;

use bson::Document;
use failure::ResultExt;
use humansize::FileSize;
use prettytable::cell;
use prettytable::row;
use prettytable::Table;

use crate::error::Result;
use crate::utils::compute_percentage;

/// Type representing names of oplog entries.
type OplogEntryName = String;

/// Statistics for a single oplog entry.
#[derive(Debug)]
struct OplogEntryStats {
    /// Number of documents for the entry.
    doc_count: u64,

    /// Total size of documents for the entry.
    doc_total_size: u64,
}

impl OplogEntryStats {
    /// Creates a new, empty statistics for a single oplog entry.
    fn new() -> Self {
        OplogEntryStats {
            doc_count: 0,
            doc_total_size: 0,
        }
    }
}

/// Statistics of a MongoDB oplog.
#[derive(Debug)]
pub struct OplogStats {
    /// Mapping of the name of an entry to its statistics.
    stats: HashMap<OplogEntryName, OplogEntryStats>,

    /// Number of processed documents so far.
    processed_doc_count: u64,
}

impl OplogStats {
    /// Returns a new, empty oplog statistics.
    pub fn new() -> Self {
        OplogStats {
            stats: HashMap::new(),
            processed_doc_count: 0,
        }
    }

    /// Updates the statistics with the given oplog document.
    pub fn update(&mut self, doc: &Document) -> Result<()> {
        let doc_size = self
            .doc_size(doc)
            .context("failed to get the size of a document")?;
        let entry_name = self
            .entry_name_for_doc(doc)
            .context("failed to get an entry name for a document")?;

        let mut value = self
            .stats
            .entry(entry_name)
            .or_insert_with(OplogEntryStats::new);
        value.doc_count += 1;
        value.doc_total_size += doc_size;

        self.processed_doc_count += 1;
        Ok(())
    }

    /// Returns the number of processed documents so far.
    pub fn get_processed_doc_count(&self) -> u64 {
        self.processed_doc_count
    }

    /// Have we processed at least one document?
    pub fn processed_at_least_one_doc(&self) -> bool {
        self.processed_doc_count > 0
    }

    /// Prints the statistics in nicely formatted table to the standard output.
    pub fn print(&self) {
        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row!["Entry", "Documents", "Total size", "Share (%)"]);
        for (entry_name, entry_stats) in self.most_common() {
            table.add_row(row![
                entry_name,
                r->entry_stats.doc_count,
                r->self.format_total_doc_size(entry_stats.doc_total_size),
                r->self.format_share_for(entry_name),
            ]);
        }
        table.printstd();
    }

    /// Formats the given document size into a human-readable string.
    fn format_total_doc_size(&self, size: u64) -> String {
        size.file_size(humansize::file_size_opts::CONVENTIONAL)
            .expect("can fail only on negative sizes")
    }

    /// Formats share for an entry with the given name.
    fn format_share_for(&self, entry_name: &str) -> String {
        let share = self.share_for(entry_name);
        if share < 0.01 {
            "< 0.01".into()
        } else {
            format!("{:.2}", share)
        }
    }

    /// Returns the size of the given document.
    fn doc_size(&self, doc: &Document) -> Result<u64> {
        let mut bytes = Vec::new();
        doc.to_writer(&mut bytes)
            .context("failed to serialize document into bytes to compute its size")?;
        Ok(bytes.len() as u64)
    }

    /// Returns the name of an entry for the given oplog document.
    fn entry_name_for_doc(&self, doc: &Document) -> Result<OplogEntryName> {
        // Database and collection which the oplog entry applies to.
        let ns = doc
            .get_str("ns")
            .context("missing 'ns' entry in oplog document")?
            .to_owned();

        // Performed operation (e.g. "i" for insertion).
        let op = doc
            .get_str("op")
            .context("missing 'op' entry in oplog document")?
            .to_owned();

        let name = format!("{}:{}", ns, op);
        Ok(name)
    }

    /// Returns oplog entries ordered by their share (in a descending order).
    fn most_common(&self) -> Vec<(&OplogEntryName, &OplogEntryStats)> {
        let mut stats_vec: Vec<_> = self.stats.iter().collect();
        stats_vec.sort_by(|(_, v1), (_, v2)| v2.doc_total_size.cmp(&v1.doc_total_size));
        stats_vec
    }

    /// Returns a share (percentage) for an entry with the given name.
    fn share_for(&self, entry_name: &str) -> f64 {
        let entry = self
            .stats
            .get(entry_name)
            .expect("trying to obtain share for a non-existing entry");
        compute_percentage(
            entry.doc_total_size as f64,
            self.total_doc_size_for_all_entries() as f64,
        )
    }

    /// Returns the total size of all documents for all entries.
    fn total_doc_size_for_all_entries(&self) -> u64 {
        self.stats.values().map(|v| v.doc_total_size).sum()
    }
}
