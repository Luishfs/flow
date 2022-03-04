use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::models::accounts::Account;
use crate::models::id::Id;

/// A Credential is a proof of identity linked to an Account. This proof is
/// issued by an Identity Provider and provided by clients as a verification of
/// their identity.
///
/// We intend to use OpenID Connect to offload identity verification, but for
/// now we'll use the special "local" `issuer`. This is only valid in
/// development/test modes, but allows creating/verifying credential workflows
/// without needing an additional Identity Provider service.
///
/// A `Credential` is a Rust representation of the Postgres database understanding
/// of the credential.
#[derive(Debug, Deserialize, FromRow, Serialize)]
pub struct Credential {
    /// The `Account` this `Credential` belongs to.
    pub account_id: Id<Account>,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
    /// The time when this `Credential` expires and the `issuer` must provide a new secret.
    pub expires_at: DateTime<Utc>,
    /// Primary key for this record.
    pub id: Id<Credential>,
    /// The name of the Identity Provider that provided the `Credential`.
    pub issuer: String,
    /// The last recorded login to the system.
    pub last_authorized_at: DateTime<Utc>,
    /// The value which gets signed in session token. Generating a new value
    /// will automatically invalidate any previous sessions using this
    /// credential.
    pub session_token: String,
    /// The unique identifier for an actor within the scope of an `issuer`.
    pub subject: String,
    /// When this record was last updated.
    pub updated_at: DateTime<Utc>,
}

/// NewCredential represents the data required to insert a new Credential, with
/// remaining fields from `Credential` generated by Postgres.
#[derive(Debug, Deserialize, FromRow, Serialize)]
pub struct NewCredential {
    /// The `Account` this `Credential` belongs to.
    pub account_id: Id<Account>,
    /// The time when this `Credential` expires and the `issuer` must provide a new secret.
    pub expires_at: DateTime<Utc>,
    /// The name of the Identity Provider that provided the `Credential`.
    pub issuer: String,
    /// The last recorded login to the system.
    pub last_authorized_at: DateTime<Utc>,
    /// The unique identifier for an actor within the scope of an `issuer`.
    pub subject: String,
}
