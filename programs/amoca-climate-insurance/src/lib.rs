use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, TokenAccount, Transfer}
};

declare_id!("8a2BSK86azg8kL6Cbd2wvEswnn2eKyS3CSZSgXpfTzTc");

/// AMOCA Climate Insurance Program
/// Provides parametric climate insurance with automated triggers
/// based on verifiable environmental data from oracles
#[program]
pub mod amoca_climate_insurance {
    use super::*;

    /// Initialize the global program state
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.bump = ctx.bumps.global_state;
        global_state.total_policies = 0;
        global_state.total_premiums_collected = 0;
        global_state.total_payouts = 0;
        global_state.is_paused = false;
        global_state.authority = ctx.accounts.authority.key();
        
        msg!("AMOCA Climate Insurance Program initialized");
        Ok(())
    }

    /// Create a new parametric climate insurance policy
    pub fn create_climate_policy(
        ctx: Context<CreateClimatePolicy>,
        params: PolicyParams,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Validate policy parameters
        require!(params.coverage_amount > 0, AmocaError::InvalidCoverageAmount);
        require!(params.end_timestamp > current_time, AmocaError::InvalidPolicyDuration);
        require!(params.premium_amount > 0, AmocaError::InvalidPremiumAmount);

        // Validate geographic bounds
        require!(
            params.geographic_bounds.latitude >= -90.0 && params.geographic_bounds.latitude <= 90.0,
            AmocaError::InvalidGeographicBounds
        );
        require!(
            params.geographic_bounds.longitude >= -180.0 && params.geographic_bounds.longitude <= 180.0,
            AmocaError::InvalidGeographicBounds
        );

        let policy = &mut ctx.accounts.policy;
        policy.bump = ctx.bumps.policy;
        policy.owner = ctx.accounts.owner.key();
        policy.status = PolicyStatus::Inactive;
        policy.policy_type = params.policy_type;
        policy.geographic_bounds = params.geographic_bounds;
        policy.trigger_thresholds = params.trigger_conditions;
        policy.coverage_amount = params.coverage_amount;
        policy.premium_amount = params.premium_amount;
        policy.start_timestamp = current_time;
        policy.end_timestamp = params.end_timestamp;
        policy.last_data_update = current_time;
        policy.monitoring_frequency = 3600; // 1 hour default
        policy.risk_score = 50; // Default medium risk
        policy.payout_calculation = PayoutFormula::LinearScale;
        policy.oracle_sources = params.oracle_sources;

        // Update global state
        let global_state = &mut ctx.accounts.global_state;
        global_state.total_policies = global_state.total_policies.checked_add(1)
            .ok_or(AmocaError::MathOverflow)?;

        msg!("Climate policy created for owner: {}", ctx.accounts.owner.key());
        msg!("Policy type: {:?}, Coverage: {}", params.policy_type, params.coverage_amount);

        Ok(())
    }

    /// Deposit premium to activate climate insurance policy
    pub fn deposit_premium(
        ctx: Context<DepositPremium>,
        _policy_id: u64,
        amount: u64,
    ) -> Result<()> {
        let policy = &mut ctx.accounts.policy;
        
        // Verify policy status
        require!(policy.status == PolicyStatus::Inactive, AmocaError::PolicyAlreadyActive);
        require!(amount >= policy.premium_amount, AmocaError::InsufficientPremium);

        // Transfer premium from user to risk pool
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.risk_pool_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Activate policy
        policy.status = PolicyStatus::Active;
        policy.premium_amount = amount;

        // Update global state
        let global_state = &mut ctx.accounts.global_state;
        global_state.total_premiums_collected = global_state.total_premiums_collected
            .checked_add(amount)
            .ok_or(AmocaError::MathOverflow)?;

        msg!("Premium deposited: {} for policy", amount);
        Ok(())
    }

    /// Submit climate data from authorized oracles
    pub fn submit_climate_data(
        ctx: Context<SubmitClimateData>,
        data_points: Vec<ClimateDataPoint>,
    ) -> Result<()> {
        let oracle_data = &mut ctx.accounts.oracle_data;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Validate oracle is authorized
        require!(oracle_data.is_active, AmocaError::OracleNotAuthorized);
        
        // Validate data points
        require!(!data_points.is_empty(), AmocaError::InvalidOracleData);
        require!(data_points.len() <= 10, AmocaError::TooManyDataPoints);

        for data_point in &data_points {
            // Check data recency (within last hour)
            require!(
                current_time - data_point.timestamp <= 3600,
                AmocaError::StaleOracleData
            );
            
            // Check confidence level
            require!(
                data_point.confidence_level >= 50,
                AmocaError::LowConfidenceData
            );
        }

        // Update oracle data
        oracle_data.last_update = current_time;
        oracle_data.data_points_count = oracle_data.data_points_count
            .checked_add(data_points.len() as u32)
            .ok_or(AmocaError::MathOverflow)?;

        // Update reputation based on data quality
        let avg_confidence: u8 = data_points.iter()
            .map(|dp| dp.confidence_level)
            .sum::<u8>() / data_points.len() as u8;
        
        oracle_data.reputation_score = (oracle_data.reputation_score as u16 + avg_confidence as u16) / 2;
        oracle_data.reputation_score = oracle_data.reputation_score.min(100);

        msg!("Climate data submitted: {} points from oracle", data_points.len());
        Ok(())
    }

    /// Evaluate climate triggers for a policy
    pub fn evaluate_climate_trigger(
        ctx: Context<EvaluateClimateTrigger>,
        _policy_id: u64,
    ) -> Result<()> {
        let policy = &mut ctx.accounts.policy;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Verify policy is active or monitoring
        require!(
            policy.status == PolicyStatus::Active || policy.status == PolicyStatus::Monitoring,
            AmocaError::PolicyNotActive
        );

        // Check if policy has expired
        require!(current_time <= policy.end_timestamp, AmocaError::PolicyExpired);

        // Evaluate trigger conditions (simplified logic)
        let trigger_met = evaluate_trigger_conditions(policy, &ctx.accounts.oracle_data)?;
        
        if trigger_met {
            policy.status = PolicyStatus::Triggered;
            msg!("Climate trigger conditions met for policy");
        } else {
            policy.status = PolicyStatus::Monitoring;
        }

        // Update last evaluation timestamp
        policy.last_data_update = current_time;

        msg!("Trigger evaluation completed");
        Ok(())
    }

    /// Execute automated climate payout
    pub fn execute_climate_payout(
        ctx: Context<ExecuteClimatePayout>,
        policy_id: u64,
        payout_amount: u64,
    ) -> Result<()> {
        let policy = &mut ctx.accounts.policy;
        
        // Verify policy is triggered
        require!(policy.status == PolicyStatus::Triggered, AmocaError::TriggerNotMet);
        
        // Validate payout amount
        require!(payout_amount > 0, AmocaError::InvalidPayoutAmount);
        require!(payout_amount <= policy.coverage_amount, AmocaError::ExcessivePayoutAmount);

        // Calculate payout based on parametric formula
        let calculated_payout = calculate_payout_amount(policy)?;
        require!(payout_amount <= calculated_payout, AmocaError::ExcessivePayoutAmount);

        // Execute payout transfer
        let seeds = &[
            b"risk_pool".as_ref(),
            &[ctx.accounts.global_state.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.risk_pool_token_account.to_account_info(),
            to: ctx.accounts.policyholder_token_account.to_account_info(),
            authority: ctx.accounts.risk_pool_pda.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, payout_amount)?;

        // Update policy status
        policy.status = PolicyStatus::Claimed;

        // Update global state
        let global_state = &mut ctx.accounts.global_state;
        global_state.total_payouts = global_state.total_payouts
            .checked_add(payout_amount)
            .ok_or(AmocaError::MathOverflow)?;

        msg!("Climate payout executed: {}", payout_amount);
        Ok(())
    }

    /// Pause the program (admin only)
    pub fn pause_program(ctx: Context<AdminAction>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.is_paused = true;
        msg!("Program paused by authority");
        Ok(())
    }

    /// Unpause the program (admin only)
    pub fn unpause_program(ctx: Context<AdminAction>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.is_paused = false;
        msg!("Program unpaused by authority");
        Ok(())
    }
}

// Helper functions

/// Evaluate trigger conditions based on policy and oracle data
fn evaluate_trigger_conditions(
    policy: &ClimatePolicy,
    _oracle_account: &UncheckedAccount,
) -> Result<bool> {
    // Simplified trigger evaluation logic
    // In production, this would:
    // 1. Read data from multiple oracle feeds
    // 2. Compare against trigger thresholds
    // 3. Apply consensus mechanisms
    // 4. Calculate confidence scores
    
    // For demonstration, return based on risk score
    Ok(policy.risk_score > 80)
}

/// Calculate payout amount based on parametric formula
fn calculate_payout_amount(policy: &ClimatePolicy) -> Result<u64> {
    match policy.payout_calculation {
        PayoutFormula::LinearScale => {
            // Linear payout based on risk score
            let payout_percentage = if policy.risk_score > 80 {
                std::cmp::min(100, policy.risk_score as u64)
            } else {
                0
            };
            Ok((policy.coverage_amount * payout_percentage) / 100)
        },
        PayoutFormula::StepFunction => {
            // Step function payout
            if policy.risk_score > 90 {
                Ok(policy.coverage_amount)
            } else if policy.risk_score > 70 {
                Ok(policy.coverage_amount / 2)
            } else {
                Ok(0)
            }
        },
        _ => Ok(0), // Other formulas not implemented
    }
}

// Account validation structs

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: PolicyParams)]
pub struct CreateClimatePolicy<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + ClimatePolicy::INIT_SPACE,
        seeds = [b"policy", owner.key().as_ref(), &params.policy_id.to_le_bytes()],
        bump
    )]
    pub policy: Account<'info, ClimatePolicy>,
    
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump,
        constraint = !global_state.is_paused @ AmocaError::ProgramPaused
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(policy_id: u64)]
pub struct DepositPremium<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"policy", owner.key().as_ref(), &policy_id.to_le_bytes()],
        bump = policy.bump,
        constraint = policy.owner == owner.key() @ AmocaError::Unauthorized
    )]
    pub policy: Account<'info, ClimatePolicy>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == owner.key() @ AmocaError::Unauthorized
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub risk_pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump,
        constraint = !global_state.is_paused @ AmocaError::ProgramPaused
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SubmitClimateData<'info> {
    #[account(mut)]
    pub oracle_provider: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"oracle", oracle_provider.key().as_ref()],
        bump = oracle_data.bump,
        constraint = oracle_data.provider == oracle_provider.key() @ AmocaError::Unauthorized
    )]
    pub oracle_data: Account<'info, OracleData>,
    
    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump,
        constraint = !global_state.is_paused @ AmocaError::ProgramPaused
    )]
    pub global_state: Account<'info, GlobalState>,
}

#[derive(Accounts)]
#[instruction(policy_id: u64)]
pub struct EvaluateClimateTrigger<'info> {
    pub evaluator: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"policy", policy.owner.as_ref(), &policy_id.to_le_bytes()],
        bump = policy.bump
    )]
    pub policy: Account<'info, ClimatePolicy>,
    
    /// CHECK: Oracle data account for trigger evaluation
    pub oracle_data: UncheckedAccount<'info>,
    
    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump,
        constraint = !global_state.is_paused @ AmocaError::ProgramPaused
    )]
    pub global_state: Account<'info, GlobalState>,
}

#[derive(Accounts)]
#[instruction(policy_id: u64)]
pub struct ExecuteClimatePayout<'info> {
    pub executor: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"policy", policy.owner.as_ref(), &policy_id.to_le_bytes()],
        bump = policy.bump
    )]
    pub policy: Account<'info, ClimatePolicy>,
    
    #[account(mut)]
    pub policyholder_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub risk_pool_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Risk pool PDA signer
    #[account(
        seeds = [b"risk_pool"],
        bump = global_state.bump
    )]
    pub risk_pool_pda: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump,
        constraint = !global_state.is_paused @ AmocaError::ProgramPaused
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AdminAction<'info> {
    #[account(
        constraint = authority.key() == global_state.authority @ AmocaError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
}

// Data structures

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub bump: u8,
    pub authority: Pubkey,
    pub total_policies: u64,
    pub total_premiums_collected: u64,
    pub total_payouts: u64,
    pub is_paused: bool,
}

#[account]
#[derive(InitSpace)]
pub struct ClimatePolicy {
    pub bump: u8,
    pub owner: Pubkey,
    pub status: PolicyStatus,
    pub policy_type: ClimateRiskType,
    pub geographic_bounds: GeoBounds,
    pub trigger_thresholds: TriggerConditions,
    #[max_len(5)]
    pub oracle_sources: Vec<Pubkey>,
    pub monitoring_frequency: u32,
    pub last_data_update: i64,
    pub risk_score: u8,
    pub payout_calculation: PayoutFormula,
    pub coverage_amount: u64,
    pub premium_amount: u64,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

#[account]
#[derive(InitSpace)]
pub struct OracleData {
    pub bump: u8,
    pub provider: Pubkey,
    pub oracle_type: OracleType,
    pub reputation_score: u16,
    pub last_update: i64,
    pub is_active: bool,
    pub data_points_count: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct PolicyParams {
    pub policy_id: u64,
    pub policy_type: ClimateRiskType,
    pub geographic_bounds: GeoBounds,
    pub trigger_conditions: TriggerConditions,
    #[max_len(5)]
    pub oracle_sources: Vec<Pubkey>,
    pub coverage_amount: u64,
    pub premium_amount: u64,
    pub end_timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum PolicyStatus {
    Inactive,
    Active,
    Monitoring,
    Triggered,
    Claimed,
    Expired,
}

impl Default for PolicyStatus {
    fn default() -> Self {
        Self::Inactive
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug)]
pub enum ClimateRiskType {
    DroughtProtection,
    FloodInsurance,
    HurricaneCoverage,
    AgriculturalClimate,
    WildfireProtection,
    SeaLevelRise,
    ExtremeTemperature,
}

impl Default for ClimateRiskType {
    fn default() -> Self {
        Self::DroughtProtection
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct GeoBounds {
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64, // Coverage radius in kilometers
}

impl Default for GeoBounds {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            radius: 100.0,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct TriggerConditions {
    pub rainfall_threshold: Option<f64>, // mm per measurement period
    pub temperature_threshold: Option<f64>, // degrees Celsius
    pub wind_speed_threshold: Option<f64>, // mph
    pub water_level_threshold: Option<f64>, // meters above normal
    pub fire_proximity_threshold: Option<f64>, // kilometers
    pub measurement_period: u32, // days
    pub minimum_duration: u32, // hours the condition must persist
}

impl Default for TriggerConditions {
    fn default() -> Self {
        Self {
            rainfall_threshold: None,
            temperature_threshold: None,
            wind_speed_threshold: None,
            water_level_threshold: None,
            fire_proximity_threshold: None,
            measurement_period: 7,
            minimum_duration: 24,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ClimateDataPoint {
    pub data_type: ClimateDataType,
    pub location: GeographicCoordinate,
    pub value: f64,
    pub timestamp: i64,
    pub confidence_level: u8, // 0-100 data quality score
    pub source_id: Pubkey, // Oracle provider identifier
    #[max_len(32)]
    pub verification_hash: Vec<u8>, // Cryptographic proof
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ClimateDataType {
    Temperature,
    Rainfall,
    WindSpeed,
    Humidity,
    WaterLevel,
    FireDetection,
    VegetationIndex,
    AtmosphericPressure,
}

impl Default for ClimateDataType {
    fn default() -> Self {
        Self::Temperature
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct GeographicCoordinate {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

impl Default for GeographicCoordinate {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: None,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum OracleType {
    ChainlinkWeather,
    PythSatellite,
    NasaModis,
    WeatherStation,
    IoTSensor,
    SwitchboardNetwork,
}

impl Default for OracleType {
    fn default() -> Self {
        Self::ChainlinkWeather
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum PayoutFormula {
    LinearScale,
    StepFunction,
    Exponential,
    Composite,
}

impl Default for PayoutFormula {
    fn default() -> Self {
        Self::LinearScale
    }
}

// Error definitions

#[error_code]
pub enum AmocaError {
    #[msg("Invalid coverage amount")]
    InvalidCoverageAmount,
    #[msg("Invalid policy duration")]
    InvalidPolicyDuration,
    #[msg("Invalid premium amount")]
    InvalidPremiumAmount,
    #[msg("Invalid geographic bounds")]
    InvalidGeographicBounds,
    #[msg("Policy already active")]
    PolicyAlreadyActive,
    #[msg("Insufficient premium")]
    InsufficientPremium,
    #[msg("Oracle not authorized")]
    OracleNotAuthorized,
    #[msg("Invalid oracle data")]
    InvalidOracleData,
    #[msg("Too many data points")]
    TooManyDataPoints,
    #[msg("Stale oracle data")]
    StaleOracleData,
    #[msg("Low confidence data")]
    LowConfidenceData,
    #[msg("Policy not active")]
    PolicyNotActive,
    #[msg("Policy expired")]
    PolicyExpired,
    #[msg("Trigger conditions not met")]
    TriggerNotMet,
    #[msg("Invalid payout amount")]
    InvalidPayoutAmount,
    #[msg("Excessive payout amount")]
    ExcessivePayoutAmount,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Program is paused")]
    ProgramPaused,
}
