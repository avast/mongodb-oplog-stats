//! Access to MongoDB.

use bson::doc;
use bson::Document;
use failure::ResultExt;
use mongodb::options::ClientOptions;
use mongodb::options::Credential;
use mongodb::options::FindOptions;
use mongodb::options::ServerAddress;
use mongodb::sync::Client;
use mongodb::sync::Collection;
use mongodb::sync::Cursor;

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
        // Use authentication credential only if authentication has been
        // requested by passing a username. This allows us to connect to
        // databases that have disabled authentication.
        let credential = if args.username.is_some() {
            Some(
                Credential::builder()
                    .username(args.username.clone())
                    .password(args.password.clone())
                    .source(args.auth_db.clone())
                    .build(),
            )
        } else {
            None
        };

        let client = Client::with_options(
            ClientOptions::builder()
                .credential(credential)
                .hosts(vec![ServerAddress::Tcp {
                    host: args.host.clone(),
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
    pub fn generate_documents_in_oplog(&self, limit: u64) -> Result<Cursor<Document>> {
        let find_options = FindOptions::builder()
            .limit(limit as i64)
            .sort(doc! {"$natural": -1i32})
            .build();
        let oplog = self.get_oplog_collection();
        let cursor = oplog
            .find(doc! {}, find_options)
            .context("oplog query failed")?;
        Ok(cursor)
    }

    /// Returns access to the oplog.
    fn get_oplog_collection(&self) -> Collection<Document> {
        let db = self.client.database("local");
        db.collection("oplog.rs")
    }
}
