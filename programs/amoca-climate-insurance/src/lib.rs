use anchor_lang::prelude::*;

declare_id!("8a2BSK86azg8kL6Cbd2wvEswnn2eKyS3CSZSgXpfTzTc");

/// A simplified climate insurance program.
#[program]
pub mod amoca_climate_insurance {
    use super::*;

    /// Initializes the program's global state.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.bump = ctx.bumps.global_state;
        global_state.authority = ctx.accounts.authority.key();
        global_state.total_policies = 0;
        global_state.total_premiums_collected = 0;
        global_state.total_payouts = 0;
        global_state.is_paused = false;
        
        msg!("Program initialized");
        Ok(())
    }

    /// Creates a new insurance policy.
    pub fn create_policy(
        ctx: Context<CreatePolicy>,
        premium: u64,
        payout: u64,
        location: String,
        start_date: i64,
        end_date: i64,
    ) -> Result<()> {
        let policy = &mut ctx.accounts.policy;
        policy.policy_holder = ctx.accounts.policy_holder.key();
        policy.premium = premium;
        policy.payout = payout;
        policy.location = location;
        policy.start_date = start_date;
        policy.end_date = end_date;
        policy.state = PolicyState::Pending;
        policy.bump = ctx.bumps.policy;

        let global_state = &mut ctx.accounts.global_state;
        global_state.total_policies = global_state.total_policies.checked_add(1).unwrap();
        
        msg!("Policy created: {}", policy.key());
        Ok(())
    }

    /// Underwrites a policy, making it active.
    pub fn underwrite_policy(ctx: Context<UnderwritePolicy>) -> Result<()> {
        let policy = &mut ctx.accounts.policy;
        require!(policy.state == PolicyState::Pending, ErrorCode::PolicyNotPending);

        // In a real-world scenario, you'd have more complex logic here,
        // like checking the premium payment, risk assessment, etc.
        
        policy.state = PolicyState::Active;

        let global_state = &mut ctx.accounts.global_state;
        global_state.total_premiums_collected = global_state.total_premiums_collected.checked_add(policy.premium).unwrap();

        msg!("Policy underwritten and active: {}", policy.key());
        Ok(())
    }

    /// Triggers a payout for a policy.
    pub fn trigger_payout(ctx: Context<TriggerPayout>) -> Result<()> {
        let policy = &mut ctx.accounts.policy;
        require!(policy.state == PolicyState::Active, ErrorCode::PolicyNotActive);

        // In a real-world scenario, this would be called by a trusted oracle
        // that verifies a climate event occurred at the policy's location.
        
        policy.state = PolicyState::PaidOut;

        let global_state = &mut ctx.accounts.global_state;
        global_state.total_payouts = global_state.total_payouts.checked_add(policy.payout).unwrap();

        // Here you would transfer the payout amount from the program's vault
        // to the policyholder. This example omits the token transfer logic for simplicity.

        msg!("Payout triggered for policy: {}", policy.key());
        Ok(())
    }
}

/// Contains the program's global state.
#[account]
pub struct GlobalState {
    pub authority: Pubkey,
    pub total_policies: u64,
    pub total_premiums_collected: u64,
    pub total_payouts: u64,
    pub is_paused: bool,
    pub bump: u8,
}

/// Represents an insurance policy.
#[account]
pub struct Policy {
    pub policy_holder: Pubkey,
    pub premium: u64,
    pub payout: u64,
    pub location: String, // E.g., "lat,lon" or a geohash
    pub start_date: i64,
    pub end_date: i64,
    pub state: PolicyState,
    pub bump: u8,
}

/// Defines the state of an insurance policy.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PolicyState {
    Pending,
    Active,
    PaidOut,
    Expired,
}

/// Context for the `initialize` instruction.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 8 + 8 + 1 + 1,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}

/// Context for the `create_policy` instruction.
#[derive(Accounts)]
pub struct CreatePolicy<'info> {
    #[account(mut)]
    pub policy_holder: Signer<'info>,
    #[account(
        init,
        payer = policy_holder,
        space = 8 + 32 + 8 + 8 + 4 + 20 + 8 + 8 + 1 + 1, // Adjust space for location string
        seeds = [b"policy", policy_holder.key().as_ref(), global_state.total_policies.to_le_bytes().as_ref()],
        bump
    )]
    pub policy: Account<'info, Policy>,
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}

/// Context for the `underwrite_policy` instruction.
#[derive(Accounts)]
pub struct UnderwritePolicy<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub policy: Account<'info, Policy>,
}

/// Context for the `trigger_payout` instruction.
#[derive(Accounts)]
pub struct TriggerPayout<'info> {
    /// In a real app, this would be a trusted oracle service.
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub policy: Account<'info, Policy>,
}

/// Error codes for the program.
#[error_code]
pub enum ErrorCode {
    #[msg("The policy is not in a pending state.")]
    PolicyNotPending,
    #[msg("The policy is not in an active state.")]
    PolicyNotActive,
}

