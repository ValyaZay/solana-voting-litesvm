use anchor_lang::declare_program;
use anchor_lang::prelude::anchor_lang;
use anchor_litesvm::{AnchorLiteSVM, AssertionHelpers, Pubkey, Signer, TestHelpers};

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

#[test]
fn test_init_poll() {
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id: u64 = 1;
    let poll_pda = get_poll_pda(poll_id);

    let start_time: i64 = 0;
    let end_time: i64 = i64::MAX;
    let poll_name = "Test Poll".to_string();
    let poll_description = "A test poll for voting".to_string();

    let ix = ctx
        .program()
        .accounts(accounts::InitPoll {
            signer: user.pubkey(), 
            poll_account: poll_pda,
            system_program: anchor_lang::system_program::ID,
        })
        .args(args::InitPoll {
            _poll_id: poll_id,
            name: poll_name.clone(), 
            description: poll_description.clone(),
            start: start_time,
            end: end_time
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&user]).unwrap();

    result.assert_success();

    ctx.svm.assert_account_exists(&poll_pda);

    let poll_account: PollAccount = ctx.get_account(&poll_pda).unwrap();
    assert_eq!(poll_account.poll_name, poll_name);
    assert_eq!(poll_account.poll_description, poll_description);
    assert_eq!(poll_account.poll_voting_start, start_time);
    assert_eq!(poll_account.poll_voting_end, end_time);
    assert_eq!(poll_account.poll_candidates_amount, 0);
}