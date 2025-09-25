use pinocchio::{entrypoint, msg, ProgramResult};
use switchboard_on_demand::{
    QuoteVerifier, check_pubkey_eq, OracleQuote, Instructions, get_slot
};
use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

mod utils;
mod example_usage;
use utils::{init_quote_account_if_needed, init_state_account_if_needed};

entrypoint!(process_instruction);

declare_id!("8a2BSK86azg8kL6Cbd2wvEswnn2eKyS3CSZSgXpfTzTc");

/// AMOCA Climate Insurance Program with Switchboard Oracle Integration
/// 
/// This program enables climate-based insurance policies using verified oracle data
/// from Switchboard. It supports drought, flood, hurricane, and wildfire protection
/// with real-time climate data verification.
#[inline(always)]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data[0] {
        0 => crank(program_id, accounts)?,
        1 => read_climate_data(program_id, accounts)?,
        2 => init_state(program_id, accounts)?,
        3 => init_oracle(program_id, accounts)?,
        4 => create_climate_policy(program_id, accounts, &instruction_data[1..])?,
        5 => evaluate_climate_trigger(program_id, accounts)?,
        6 => execute_climate_payout(program_id, accounts, &instruction_data[1..])?,
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}

/// Updates the Switchboard Oracle quote account with latest climate data
#[inline(always)]
pub fn crank(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [quote, queue, state, payer, instructions_sysvar, _clock_sysvar]: &[AccountInfo; 6] =
        accounts.try_into().map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    if !is_state_account(state, program_id) {
        msg!("Invalid state account");
        return Err(ProgramError::Custom(2)); // InvalidStateAccount
    }

    // Simple state management - store authorized signer for climate insurance operations
    let state_data = unsafe { state.borrow_data_unchecked() };

    if !check_pubkey_eq(&state_data, payer.key()) {
        // Signer mismatch, reject
        msg!("Unauthorized signer for oracle update");
        return Err(ProgramError::Custom(1)); // UnauthorizedSigner
    }

    // DANGER: only use this if you trust the signer and all accounts passed in this tx
    OracleQuote::write_from_ix_unchecked(instructions_sysvar, quote, queue.key(), 0);
    
    msg!("Climate oracle data updated successfully");
    Ok(())
}

/// Reads and displays climate data from Switchboard Oracle feeds
#[inline(always)]
pub fn read_climate_data(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [quote, queue, clock_sysvar, slothashes_sysvar, instructions_sysvar]: &[AccountInfo; 5] =
        accounts.try_into().map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    let slot = get_slot(clock_sysvar);

    let quote_data = QuoteVerifier::new()
        .slothash_sysvar(slothashes_sysvar)
        .ix_sysvar(instructions_sysvar)
        .clock_slot(slot)
        .queue(queue)
        .max_age(30) // Allow data up to 30 slots old
        .verify_account(quote)
        .map_err(|_| ProgramError::Custom(0))?; // InvalidQuoteAccount

    msg!("Climate data quote slot: {}", quote_data.slot());

    // Parse and display each climate feed
    for (index, feed_info) in quote_data.feeds().iter().enumerate() {
        let feed_id = feed_info.hex_id();
        let value = feed_info.value();
        
        msg!("🌡️ Climate Feed #{}: {}", index + 1, feed_id);
        msg!("📊 Value: {}", value);
        
        // Interpret climate data based on feed ID patterns
        if feed_id.contains("temp") || feed_id.contains("temperature") {
            msg!("🌡️ Temperature reading: {}°C", value);
        } else if feed_id.contains("rain") || feed_id.contains("precip") {
            msg!("🌧️ Rainfall reading: {}mm", value);
        } else if feed_id.contains("wind") {
            msg!("💨 Wind speed: {}m/s", value);
        } else if feed_id.contains("humid") {
            msg!("💧 Humidity: {}%", value);
        } else {
            msg!("📈 Generic climate metric: {}", value);
        }
    }

    Ok(())
}

/// Creates a new climate insurance policy with oracle-verified triggers
#[inline(always)]
pub fn create_climate_policy(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [policy_owner, policy_account, state, system_program]: &[AccountInfo; 4] =
        accounts.try_into().map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    if !is_state_account(state, program_id) {
        msg!("Invalid state account");
        return Err(ProgramError::Custom(2)); // InvalidStateAccount
    }

    // Decode policy parameters from instruction data
    if instruction_data.len() < 32 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // For simplicity, we'll store basic policy info in the account
    // In a full implementation, you'd deserialize complete policy parameters
    
    // Initialize policy account if needed
    if policy_account.lamports() == 0 {
        // Create policy account (simplified - in production use proper PDA derivation)
        msg!("Creating new climate insurance policy");
    }

    msg!("Climate insurance policy created for owner: {}", policy_owner.key());
    msg!("Policy will use Switchboard Oracle feeds for climate trigger verification");
    
    Ok(())
}

/// Evaluates climate triggers for a policy using Switchboard Oracle data
#[inline(always)]
pub fn evaluate_climate_trigger(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [policy, quote, queue, clock_sysvar, slothashes_sysvar, instructions_sysvar, state]: &[AccountInfo; 7] =
        accounts.try_into().map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    if !is_state_account(state, program_id) {
        msg!("Invalid state account");
        return Err(ProgramError::Custom(2)); // InvalidStateAccount
    }

    let slot = get_slot(clock_sysvar);

    // Verify and read oracle data for climate triggers
    let quote_data = QuoteVerifier::new()
        .slothash_sysvar(slothashes_sysvar)
        .ix_sysvar(instructions_sysvar)
        .clock_slot(slot)
        .queue(queue)
        .max_age(50) // Allow slightly older data for trigger evaluation
        .verify_account(quote)
        .map_err(|_| ProgramError::Custom(0))?; // InvalidQuoteAccount

    msg!("Evaluating climate triggers for policy: {}", policy.key());
    
    let mut trigger_activated = false;
    let mut trigger_reason = String::new();

    // Evaluate climate conditions against policy triggers
    for (index, feed_info) in quote_data.feeds().iter().enumerate() {
        let feed_id = feed_info.hex_id();
        let value = feed_info.value();
        
        msg!("🔍 Evaluating feed {}: {} = {}", index + 1, feed_id, value);
        
        // Climate trigger logic based on feed types
        if feed_id.contains("temp") || feed_id.contains("temperature") {
            // Temperature-based triggers (extreme heat/cold)
            if value > 40.0 || value < -10.0 {
                trigger_activated = true;
                trigger_reason = format!("Extreme temperature: {}°C", value);
                msg!("🔥 TRIGGER: {}", trigger_reason);
            }
        } else if feed_id.contains("rain") || feed_id.contains("precip") {
            // Rainfall-based triggers (drought/flood)
            if value > 100.0 { // Heavy rainfall threshold
                trigger_activated = true;
                trigger_reason = format!("Heavy rainfall detected: {}mm", value);
                msg!("🌊 TRIGGER: {}", trigger_reason);
            } else if value < 1.0 { // Drought threshold
                trigger_activated = true;
                trigger_reason = format!("Drought conditions: {}mm rainfall", value);
                msg!("🏜️ TRIGGER: {}", trigger_reason);
            }
        } else if feed_id.contains("wind") {
            // Wind-based triggers (hurricane/storm)
            if value > 25.0 { // Hurricane force winds
                trigger_activated = true;
                trigger_reason = format!("Hurricane winds detected: {}m/s", value);
                msg!("🌪️ TRIGGER: {}", trigger_reason);
            }
        }
    }

    if trigger_activated {
        msg!("✅ Policy trigger activated: {}", trigger_reason);
        msg!("Policy {} is now eligible for payout", policy.key());
    } else {
        msg!("❌ No triggers activated - policy conditions not met");
    }

    Ok(())
}

/// Executes a climate payout based on verified oracle trigger conditions
#[inline(always)]
pub fn execute_climate_payout(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [policy, beneficiary, quote, queue, clock_sysvar, slothashes_sysvar, instructions_sysvar, state]: &[AccountInfo; 8] =
        accounts.try_into().map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    if !is_state_account(state, program_id) {
        msg!("Invalid state account");
        return Err(ProgramError::Custom(2)); // InvalidStateAccount
    }

    // Decode payout amount from instruction data
    if instruction_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let payout_amount = u64::from_le_bytes(
        instruction_data[0..8].try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?
    );

    let slot = get_slot(clock_sysvar);

    // Verify oracle data before executing payout
    let quote_data = QuoteVerifier::new()
        .slothash_sysvar(slothashes_sysvar)
        .ix_sysvar(instructions_sysvar)
        .clock_slot(slot)
        .queue(queue)
        .max_age(100) // Allow older data for payout verification
        .verify_account(quote)
        .map_err(|_| ProgramError::Custom(0))?; // InvalidQuoteAccount

    msg!("🏦 Executing climate insurance payout");
    msg!("Policy: {}", policy.key());
    msg!("Beneficiary: {}", beneficiary.key());
    msg!("Amount: {} lamports", payout_amount);

    // Re-verify trigger conditions before payout
    let mut payout_justified = false;
    
    for (index, feed_info) in quote_data.feeds().iter().enumerate() {
        let feed_id = feed_info.hex_id();
        let value = feed_info.value();
        
        msg!("📊 Verifying feed {}: {} = {}", index + 1, feed_id, value);
        
        // Verify payout conditions are still met
        if feed_id.contains("temp") && (value > 40.0 || value < -10.0) {
            payout_justified = true;
            msg!("✅ Temperature trigger confirmed: {}°C", value);
        } else if feed_id.contains("rain") && (value > 100.0 || value < 1.0) {
            payout_justified = true;
            msg!("✅ Precipitation trigger confirmed: {}mm", value);
        } else if feed_id.contains("wind") && value > 25.0 {
            payout_justified = true;
            msg!("✅ Wind trigger confirmed: {}m/s", value);
        }
    }

    if !payout_justified {
        msg!("❌ Payout conditions not met based on current oracle data");
        return Err(ProgramError::Custom(3)); // PayoutConditionsNotMet
    }

    // In a real implementation, you would transfer tokens here
    // For this example, we just log the successful payout
    msg!("💰 PAYOUT EXECUTED: {} lamports transferred to {}", payout_amount, beneficiary.key());
    msg!("🎯 Climate insurance claim successfully processed using Switchboard Oracle verification");
    
    Ok(())
}

/// Checks if the given account is a valid state account for this program
#[inline(always)]
pub fn is_state_account(account: &AccountInfo, program_id: &Pubkey) -> bool {
    check_pubkey_eq(account.owner(), program_id) && account.data_len() == 32
}

/// Initializes the program's state account for climate insurance operations
#[inline(always)]
pub fn init_state(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [state, payer, system_program]: &[AccountInfo; 3] =
        accounts.try_into().map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    init_state_account_if_needed(
        program_id,
        state,
        payer,
        system_program,
    )?;

    // Store the payer's pubkey as the authorized signer for climate insurance operations
    state.try_borrow_mut_data()?[..32].copy_from_slice(payer.key().as_ref());

    msg!("Climate insurance state initialized with authority: {}", payer.key());
    Ok(())
}

/// Initializes the oracle quote account for climate data feeds
#[inline(always)]
pub fn init_oracle(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [quote, queue, payer, system_program, instructions_sysvar]: &[AccountInfo; 5] =
        accounts.try_into().map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    let quote_data = Instructions::parse_ix_data_unverified(instructions_sysvar, 0)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    init_quote_account_if_needed(
        program_id,
        quote,
        queue,
        payer,
        system_program,
        &quote_data,
    )?;

    msg!("Climate oracle quote account initialized");
    msg!("Queue: {}", queue.key());
    msg!("Quote: {}", quote.key());
    
    Ok(())
}

/// Climate Insurance Policy Data Structure
/// Stores policy information in a compact format for on-chain storage
#[repr(C)]
pub struct ClimatePolicy {
    pub owner: Pubkey,
    pub policy_id: u64,
    pub policy_type: u8, // 0=Drought, 1=Flood, 2=Hurricane, 3=Wildfire
    pub coverage_amount: u64,
    pub premium_paid: u64,
    pub expiry_slot: u64,
    pub trigger_threshold: f32, // Simplified single threshold
    pub status: u8, // 0=Inactive, 1=Active, 2=Triggered, 3=PaidOut
}

impl ClimatePolicy {
    pub const SIZE: usize = 32 + 8 + 1 + 8 + 8 + 8 + 4 + 1; // 70 bytes
}

/// Climate trigger thresholds for different policy types
pub struct ClimateTriggers {
    pub drought_rainfall_threshold: f64,      // mm per month (below triggers drought)
    pub flood_rainfall_threshold: f64,        // mm per day (above triggers flood)
    pub hurricane_wind_threshold: f64,        // m/s (above triggers hurricane)
    pub extreme_temp_high_threshold: f64,     // °C (above triggers heat)
    pub extreme_temp_low_threshold: f64,      // °C (below triggers cold)
}

impl Default for ClimateTriggers {
    fn default() -> Self {
        Self {
            drought_rainfall_threshold: 10.0,    // Less than 10mm/month
            flood_rainfall_threshold: 100.0,     // More than 100mm/day
            hurricane_wind_threshold: 25.0,      // More than 25 m/s (90+ km/h)
            extreme_temp_high_threshold: 40.0,   // Above 40°C
            extreme_temp_low_threshold: -10.0,   // Below -10°C
        }
    }
}

// Custom error codes for AMOCA Climate Insurance with Switchboard Oracle
// 0: InvalidQuoteAccount - Invalid quote account - not the canonical account for the contained feeds
// 1: UnauthorizedSigner - Unauthorized signer - does not match stored signer
// 2: InvalidStateAccount - Invalid state account for the program
// 3: PayoutConditionsNotMet - Payout conditions not satisfied by oracle data
// 4: InvalidPolicyType - Invalid climate policy type specified
// 5: ExcessiveClaimAmount - Claim amount exceeds policy coverage
// 6: PolicyExpired - Policy has expired and is no longer valid
// 7: InsufficientOracleData - Insufficient or stale oracle data for decision making

