use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Poll Not Started")]
    PollNotStarted,

    #[msg("Poll Already Ended")]
    PollAlreadyEnded,
}