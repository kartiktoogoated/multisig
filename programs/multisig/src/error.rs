use anchor_lang::prelude::*;

#[error_code]
pub enum MultisigError {
    #[msg("Threshold must be valid and ≤ number of owners.")]
    InvalidThreshold,
    #[msg("At least one owner is required.")]
    NoOwners,
    #[msg("Owners must be unique.")]
    DuplicateOwners,
    #[msg("The given owner is not part of this multisig.")]
    InvalidOwner,
}
