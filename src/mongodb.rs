//! Access to MongoDB.

use bson;
use bson::doc;
use failure::ResultExt;
use mongodb::options::auth::Credential;
use mongodb::options::ClientOptions;
use mongodb::options::FindOptions;
use mongodb::options::StreamAddress;
use mongodb::Client;
use mongodb::Collection;
use mongodb::Cursor;

use crate::args::Args;
use crate::error::Result;

/// Access to a MongoDB instance.
#[derive(Debug)]
pub struct MongoDB {
    /// Internal MongoDB access.
    client: Client,
}

impl MongoDB {
    /// Creates an access to a MongoDB instance from the tool arguments.
    pub fn from_args(args: &Args) -> Result<Self> {
        let client = Client::with_options(
            ClientOptions::builder()
                .credential(Some(
                    Credential::builder()
                        .username(args.username.clone())
                        .password(args.password.clone())
                        .source(args.auth_db.clone())
                        .build(),
                ))
                .hosts(vec![StreamAddress {
                    hostname: args.host.clone(),
                    port: Some(args.port),
                }])
                .build(),
        )
        .context("failed to create a database client")?;
        Ok(MongoDB { client })
    }

    /// Returns the total number of documents in the oplog.
    pub fn get_total_number_of_documents_in_oplog(&self) -> Result<u64> {
        let oplog = self.get_oplog_collection();
        let document_count = oplog
            .estimated_document_count(None)
            .context("oplog query failed")? as u64;
        Ok(document_count)
    }

    /// Returns a cursor for documents in the oplog.
    ///
    /// # Arguments
    ///
    /// * `limit` - The maximum number of documents to return.
    pub fn generate_documents_in_oplog(&self, limit: u64) -> Result<Cursor> {
        let find_options = FindOptions::builder()
            .limit(limit as i64)
            .sort(doc! {"$natural": -1})
            .build();
        let oplog = self.get_oplog_collection();
        let cursor = oplog
            .find(doc! {}, find_options)
            .context("oplog query failed")?;
        Ok(cursor)
    }

    /// Returns access to the collection representing the oplog.
    fn get_oplog_collection(&self) -> Collection {
        let db = self.client.database("local");
        db.collection("oplog.rs")
    }
}
