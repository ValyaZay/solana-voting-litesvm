use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Poll is not active")]
    PollNotActive
}