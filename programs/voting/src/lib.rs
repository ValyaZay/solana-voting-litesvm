use anchor_lang::prelude::*;

pub use error::Errors;
pub mod error;

declare_id!("3b9t3AxeTPFqoNCAaWcocGCwADe4nm2cJ4u1TPXWVf6a");

#[program]
pub mod voting {
    use super::*;

    pub fn init_poll(ctx: Context<InitPoll>, _poll_id: u64, start: i64, end: i64, name: String, description: String,) -> Result<()> {
        let poll = &mut ctx.accounts.poll_account;
        poll.poll_name = name;
        poll.poll_description = description;
        poll.poll_voting_start = start;
        poll.poll_voting_end = end;

        Ok(())
    }

    pub fn init_candidate(ctx: Context<InitCandidate>, _poll_id: u64, candidate_name: String,) -> Result<()> {
        ctx.accounts.candidate_account.candidate_name = candidate_name;
        ctx.accounts.poll_account.poll_candidates_amount += 1;

        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, _poll_id: u64, _candidate_name: String) -> Result<()> {
        let poll_account = &ctx.accounts.poll_account;
        let now = Clock::get()?.unix_timestamp;
        require!(now >= poll_account.poll_voting_start, Errors::PollNotStarted);
        require!(now <= poll_account.poll_voting_end, Errors::PollAlreadyEnded);
        
        ctx.accounts.candidate_account.candidate_votes += 1;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitPoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_account: Account<'info, PollAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate_name: String)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
        bump,
    )]
    pub candidate_account: Account<'info, CandidateAccount>,

    #[account(
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_account: Account<'info, PollAccount>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate_name: String)]
pub struct InitCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut, 
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + CandidateAccount::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
        bump,
    )]
    pub candidate_account: Account<'info, CandidateAccount>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    #[max_len(32)]
    pub poll_name: String,
    #[max_len(280)]
    pub poll_description: String,
    pub poll_voting_start: i64,
    pub poll_voting_end: i64,
    pub poll_candidates_amount: u64,
}

#[account]
#[derive(InitSpace)]
pub struct CandidateAccount {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_votes: u64,
}
