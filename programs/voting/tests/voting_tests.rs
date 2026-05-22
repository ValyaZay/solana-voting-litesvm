use anchor_lang::declare_program;
use anchor_lang::prelude::anchor_lang;
use anchor_lang::solana_program::clock::Clock;
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, AssertionHelpers, Pubkey, Signer, TestHelpers, TransactionResult};
use solana_keypair::Keypair;
use solana_transaction::Address;

declare_program!(voting);

use self::voting::client::{accounts, args};
use self::voting::accounts::{CandidateAccount, PollAccount};

const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/voting.so");

fn setup() -> anchor_litesvm::AnchorContext {
    let ctx = AnchorLiteSVM::build_with_program(self::voting::ID, PROGRAM_BYTES);
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

fn vote_helper(ctx: &mut AnchorContext, voter: &Keypair, candidate_pda: Address, poll_pda: Address, poll_id: u64, candidate_name: &str) -> TransactionResult {
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
    ctx.execute_instruction(ix, &[&voter]).unwrap()
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
    let result = vote_helper(&mut ctx, &voter, candidate_pda, poll_pda, poll_id, candidate_name);
    result.assert_success();
    
    // Assert
    let candidate_account: CandidateAccount = ctx.get_account(&candidate_pda).unwrap();
    assert_eq!(candidate_account.candidate_votes, 1);
}

#[test]
fn vote_fails_before_poll_start() {
    // Arrange
    let mut ctx = setup();

    // set current timestamp < poll_start
    let current_timestamp = 1000;
    let poll_start_timestamp = 1500;
    let poll_end_timestamp = 2000;

    let mut current_clock = ctx.svm.get_sysvar::<Clock>();
    current_clock.unix_timestamp = current_timestamp;
    ctx.svm.set_sysvar(&current_clock);

    // init poll and candidate
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let poll_id = 1;
    let start_time: i64 = poll_start_timestamp;
    let end_time: i64 = poll_end_timestamp;
    let poll_name = "Test Poll";
    let poll_description = "A test poll for voting";
    let poll_pda = initialize_poll_helper(&mut ctx, &authority, poll_id, start_time, end_time, poll_name, poll_description);

    let candidate_name = "Joe";
    let candidate_pda = initialize_candidate_helper(&mut ctx, &authority, poll_pda, poll_id, candidate_name);

    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    // Act, Assert
    let current_clock = ctx.svm.get_sysvar::<Clock>();
    assert!(poll_start_timestamp > current_clock.unix_timestamp, "Poll start_time < current_timestamp");
    println!("current_timestamp {}", current_clock.unix_timestamp);

    vote_helper(&mut ctx, &voter, candidate_pda, poll_pda, poll_id, candidate_name)
        .assert_failure()
        .assert_anchor_error("PollNotStarted");
}

#[test]
fn vote_fails_after_poll_end() {
    // Arrange
    let mut ctx = setup();

    // set current timestamp < poll_start
    let current_timestamp = 3000;
    let poll_start_timestamp = 1500;
    let poll_end_timestamp = 2000;

    let mut current_clock = ctx.svm.get_sysvar::<Clock>();
    current_clock.unix_timestamp = current_timestamp;
    ctx.svm.set_sysvar(&current_clock);

    // init poll and candidate
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let poll_id = 1;
    let start_time: i64 = poll_start_timestamp;
    let end_time: i64 = poll_end_timestamp;
    let poll_name = "Test Poll";
    let poll_description = "A test poll for voting";
    let poll_pda = initialize_poll_helper(&mut ctx, &authority, poll_id, start_time, end_time, poll_name, poll_description);

    let candidate_name = "Joe";
    let candidate_pda = initialize_candidate_helper(&mut ctx, &authority, poll_pda, poll_id, candidate_name);

    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    // Act, Assert
    let current_clock = ctx.svm.get_sysvar::<Clock>();
    assert!(poll_end_timestamp < current_clock.unix_timestamp, "Poll end_time > current_timestamp");
    println!("current_timestamp {}", current_clock.unix_timestamp);

    vote_helper(&mut ctx, &voter, candidate_pda, poll_pda, poll_id, candidate_name)
        .assert_failure()
        .assert_anchor_error("PollAlreadyEnded");
}