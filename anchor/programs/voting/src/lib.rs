#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("coUnmi3oBUtwtd9fjeAvSsJssXh5A5xyPbhpewyzRVF");

#[program]
pub mod voting {
    use super::*;

    pub fn initialize_poll(
        ctx: Context<InitializePoll>,
        poll_id: u64,
        description: String,
        poll_start: u64,
        poll_end: u64,
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        let clock = Clock::get()?;
        if poll_start > poll_end as u64 {
            return Err(ErrorCode::PollStartAfterEnd.into());
        }
        if poll_end <= clock.unix_timestamp as u64 {
            return Err(ErrorCode::PollEndInPast.into());
        }
        poll.poll_id = poll_id;
        poll.description = description;
        poll.poll_start = poll_start;
        poll.poll_end = poll_end;
        poll.candidate_amount = 0;
        poll.total_votes = 0;
        Ok(())
    }
    #[error_code]
    pub enum ErrorCode {
        #[msg("The poll end time must be in the future.")]
        PollEndInPast,
        #[msg("The poll start time must be before the end time.")]
        PollStartAfterEnd,
    }

    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>,
        candidate_name: String,
        _poll_id: u64,
    ) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate;
        let poll = &mut ctx.accounts.poll;
        poll.candidate_amount += 1;
        candidate.candidate_name = candidate_name;
        candidate.candidate_votes = 0;
        Ok(())
    }


    pub fn vote(ctx: Context<Vote>, _candidate_name: String, _poll_id: u64) -> Result<()> {
        // Check if the participant has already cast a ballot
        let participation_record = &mut ctx.accounts.participation_record;
        if participation_record.has_participated {
            return Err(error!(ProgramError::DuplicateVoteAttempt));
        }

        // Mark as participated and save reference
        participation_record.has_participated = true;
        participation_record.poll_reference = ctx.accounts.poll.key();

        let candidate = &mut ctx.accounts.candidate;
        candidate.candidate_votes += 1;
        let poll = &mut ctx.accounts.poll;
        poll.total_votes += 1;
        msg!("Voted for candidate: {}", candidate.candidate_name);
        msg!("Votes: {}", candidate.candidate_votes);
        Ok(())
    }
  }

  #[error_code]
  pub enum VotingError {
    #[msg("The poll has not started yet.")]
    PollNotStarted,

    #[msg("The poll has already ended.")]
    PollEnded,
  }

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, Poll>,

    #[account(
      mut,
      seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
      bump
    )]
    pub candidate: Account<'info, Candidate>,

    #[account(
      init_if_needed,
      payer = signer,
      space = 8 + ParticipationRecord::INIT_SPACE,
      seeds = [poll_id.to_le_bytes().as_ref(), signer.key().as_ref()],
      bump
    )]
    pub participation_record: Account<'info, ParticipationRecord>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump
      )]
    pub poll: Account<'info, Poll>,

    #[account(
      init,
      payer = signer,
      space = 8 + Candidate::INIT_SPACE,
      seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
      bump
    )]
    pub candidate: Account<'info, Candidate>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Candidate {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_votes: u64,
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
      init,
      payer = signer,
      space = 8 + Poll::INIT_SPACE,
      seeds = [poll_id.to_le_bytes().as_ref()],
      bump
    )]
    pub poll: Account<'info, Poll>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub poll_id: u64,
    #[max_len(200)]
    pub description: String,
    pub poll_start: u64,
    pub poll_end: u64,
    pub candidate_amount: u64,
    pub total_votes: u64,
}

#[account]
#[derive(InitSpace)]
pub struct ParticipationRecord {
    pub has_participated: bool,
    pub poll_reference: Pubkey,
}

#[error_code]
pub enum ProgramError {
    #[msg("This wallet has already participated in the current poll.")]
    DuplicateVoteAttempt,
}