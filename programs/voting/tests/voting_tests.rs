use anchor_lang::declare_program;
use anchor_lang::prelude::anchor_lang;
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, AssertionHelpers, Pubkey, Signer, TestHelpers};
use solana_transaction::Address;

declare_program!(voting);

use self::voting::client::{accounts, args};
use self::voting::accounts::{CandidateAccount, PollAccount};

const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/voting.so");

fn setup() -> anchor_litesvm::AnchorContext {
    use anchor_lang::solana_program::clock::Clock;
    let mut ctx = AnchorLiteSVM::build_with_program(self::voting::ID, PROGRAM_BYTES);

    let clock = Clock {
        slot: 1000,
        epoch_start_timestamp: 0,
        epoch: 1,
        leader_schedule_epoch: 1, 
        unix_timestamp: 1000,
    };
    ctx.svm.set_sysvar(&clock);
    ctx
}

fn get_poll_pda(poll_id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"poll", &poll_id.to_le_bytes()], &self::voting::ID).0
}

fn get_candidate_pda(poll_id: u64, candidate_name: &str) -> Pubkey {
    Pubkey::find_program_address(&[&poll_id.to_le_bytes(), candidate_name.as_bytes()], &self::voting::ID).0
}

fn initialize_poll_helper(
    ctx: &mut AnchorContext,
    signer: &anchor_litesvm::Keypair,
    poll_id: u64,
    start_time: i64, 
    end_time: i64, 
    poll_name: &str,
    poll_description: &str,
) -> Address {
    let poll_pda = get_poll_pda(poll_id);

    let ix = ctx
        .program()
        .accounts(accounts::InitPoll {
            signer: signer.pubkey(), 
            poll_account: poll_pda,
            system_program: anchor_lang::system_program::ID,
        })
        .args(args::InitPoll {
            _poll_id: poll_id,
            name: poll_name.to_string(), 
            description: poll_description.to_string(),
            start: start_time,
            end: end_time
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[signer]).unwrap();

    result.assert_success();

    ctx.svm.assert_account_exists(&poll_pda);
    poll_pda
}

fn initialize_candidate_helper(ctx: &mut AnchorContext, signer: &anchor_litesvm::Keypair, poll_pda: Address, poll_id: u64, candidate_name: &str) -> Address {
    let candidate_pda = get_candidate_pda(poll_id, candidate_name);

    let ix = ctx
        .program()
        .accounts(accounts::InitCandidate {
            signer: signer.pubkey(),
            poll_account: poll_pda,
            candidate_account: candidate_pda,
            system_program: anchor_lang::system_program::ID,
        })
        .args(args::InitCandidate{
            _poll_id: poll_id,
            candidate_name: candidate_name.to_string(),
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[signer]).unwrap();
    result.assert_success();
    ctx.svm.assert_account_exists(&candidate_pda);
    candidate_pda
}

#[test]
fn test_init_poll() {
    let mut ctx = setup();
    let signer = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id: u64 = 1;
    let start_time: i64 = 0;
    let end_time: i64 = i64::MAX;
    let poll_name = "Test Poll";
    let poll_description = "A test poll for voting";

    let poll_pda = initialize_poll_helper(&mut ctx, &signer, poll_id, start_time, end_time, poll_name, poll_description);

    let poll_account: PollAccount = ctx.get_account(&poll_pda).unwrap();
    assert_eq!(poll_account.poll_name, poll_name);
    assert_eq!(poll_account.poll_description, poll_description);
    assert_eq!(poll_account.poll_voting_start, start_time);
    assert_eq!(poll_account.poll_voting_end, end_time);
    assert_eq!(poll_account.poll_candidates_amount, 0);
}

#[test]
fn test_init_candidates() {
    // Arrange
    let mut ctx = setup();
    let signer = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id = 1;
    let start_time: i64 = 0;
    let end_time: i64 = i64::MAX;
    let poll_name = "Test Poll";
    let poll_description = "A test poll for voting";

    let poll_pda = initialize_poll_helper(&mut ctx, &signer, poll_id, start_time, end_time, poll_name, poll_description);

    let candidate_name_1 = "Alice";
    let candidate_name_2 = "Bob";

    // Act
    let candidate_pda_1 = initialize_candidate_helper(&mut ctx, &signer, poll_pda, poll_id, candidate_name_1);
    let candidate_pda_2 = initialize_candidate_helper(&mut ctx, &signer, poll_pda, poll_id, candidate_name_2);

    // Assert
    let poll_account: PollAccount = ctx.get_account(&poll_pda).unwrap();
    assert_eq!(poll_account.poll_candidates_amount, 2);

    let candidate_account_1: CandidateAccount = ctx.get_account(&candidate_pda_1).unwrap();
    assert_eq!(candidate_account_1.candidate_name, candidate_name_1);
    assert_eq!(candidate_account_1.candidate_votes, 0);

    let candidate_account_2: CandidateAccount = ctx.get_account(&candidate_pda_2).unwrap();
    assert_eq!(candidate_account_2.candidate_name, candidate_name_2);
    assert_eq!(candidate_account_2.candidate_votes, 0);
}

#[test]
fn test_vote() {
    // Arrange
    let mut ctx = setup();
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id = 1;
    let start_time: i64 = 0;
    let end_time: i64 = i64::MAX;
    let poll_name = "Test Poll";
    let poll_description = "A test poll for voting";
    let poll_pda = initialize_poll_helper(&mut ctx, &authority, poll_id, start_time, end_time, poll_name, poll_description);

    let candidate_name = "Joe";
    let candidate_pda = initialize_candidate_helper(&mut ctx, &authority, poll_pda, poll_id, candidate_name);

    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    // Act
    let ix = ctx
        .program()
        .accounts(accounts::Vote {
            signer: voter.pubkey(),
            candidate_account: candidate_pda,
            poll_account: poll_pda,
        })
        .args(args::Vote{
            _poll_id: poll_id,
            _candidate_name: candidate_name.to_string(),
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&voter]).unwrap();

    // Assert
    result.assert_success();

    let candidate_account: CandidateAccount = ctx.get_account(&candidate_pda).unwrap();
    assert_eq!(candidate_account.candidate_votes, 1);
}