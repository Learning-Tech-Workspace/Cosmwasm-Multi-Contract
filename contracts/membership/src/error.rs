use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Not enough initial members")]
    NotEnoughInitialMembers,
    #[error("Not enough required acceptances")]
    NotEnoughRequiredAcceptances,
    #[error("Unauthorized")]
    Unauthorized,
}
