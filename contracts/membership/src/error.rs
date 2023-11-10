use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    ParseReplyError(#[from] ParseReplyError),
    #[error("Not enough initial members")]
    NotEnoughInitialMembers,
    #[error("Not enough required acceptances")]
    NotEnoughRequiredAcceptances,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Missing data")]
    MissingData,
    #[error("Unknown reply id")]
    UnrecognizedReplyId(u64),
    #[error("Already voted on this proposal")]
    AlreadyVoted,
    #[error("Cannot propose a member")]
    AlreadyAMember,
}
