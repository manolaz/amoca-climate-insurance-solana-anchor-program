/// Example usage of the AMOCA Climate Insurance program with Switchboard Oracle
/// 
/// This file provides examples of how to interact with the upgraded program
/// that now uses Switchboard Oracle for climate data verification.

use pinocchio::pubkey::Pubkey;

/// Example instruction data for different operations
pub mod instruction_examples {
    /// Instruction 0: Crank - Update oracle with latest climate data
    pub const CRANK: u8 = 0;
    
    /// Instruction 1: Read climate data from oracle
    pub const READ_CLIMATE_DATA: u8 = 1;
    
    /// Instruction 2: Initialize state account
    pub const INIT_STATE: u8 = 2;
    
    /// Instruction 3: Initialize oracle quote account
    pub const INIT_ORACLE: u8 = 3;
    
    /// Instruction 4: Create climate policy
    pub const CREATE_CLIMATE_POLICY: u8 = 4;
    
    /// Instruction 5: Evaluate climate triggers
    pub const EVALUATE_CLIMATE_TRIGGER: u8 = 5;
    
    /// Instruction 6: Execute climate payout
    pub const EXECUTE_CLIMATE_PAYOUT: u8 = 6;
}

/// Example climate insurance policy types
pub enum ExamplePolicyType {
    DroughtProtection = 0,
    FloodInsurance = 1,
    HurricaneCoverage = 2,
    WildfireProtection = 3,
}

/// Example trigger thresholds for different climate events
pub struct ExampleTriggerThresholds {
    /// Drought: Less than 10mm rainfall per month
    pub drought_rainfall_mm_month: f64,
    /// Flood: More than 100mm rainfall per day  
    pub flood_rainfall_mm_day: f64,
    /// Hurricane: Wind speed above 25 m/s (90+ km/h)
    pub hurricane_wind_speed_ms: f64,
    /// Extreme heat: Temperature above 40°C
    pub extreme_heat_celsius: f64,
    /// Extreme cold: Temperature below -10°C
    pub extreme_cold_celsius: f64,
}

impl Default for ExampleTriggerThresholds {
    fn default() -> Self {
        Self {
            drought_rainfall_mm_month: 10.0,
            flood_rainfall_mm_day: 100.0,
            hurricane_wind_speed_ms: 25.0,
            extreme_heat_celsius: 40.0,
            extreme_cold_celsius: -10.0,
        }
    }
}

/// Example workflow for using the climate insurance program
/// 
/// 1. Initialize state account (one time setup)
/// 2. Initialize oracle quote account for climate feeds
/// 3. Create climate insurance policy
/// 4. Regularly crank oracle to update climate data
/// 5. Evaluate triggers based on oracle data
/// 6. Execute payouts when triggers are met
/// 
/// Required Accounts for each instruction:
/// 
/// Crank (Update Oracle):
/// - quote: Oracle quote account
/// - queue: Switchboard oracle queue
/// - state: Program state account
/// - payer: Transaction payer/signer
/// - instructions_sysvar: Instructions sysvar
/// - clock_sysvar: Clock sysvar
/// 
/// Read Climate Data:
/// - quote: Oracle quote account
/// - queue: Switchboard oracle queue  
/// - clock_sysvar: Clock sysvar
/// - slothashes_sysvar: SlotHashes sysvar
/// - instructions_sysvar: Instructions sysvar
/// 
/// Evaluate Climate Trigger:
/// - policy: Climate insurance policy account
/// - quote: Oracle quote account
/// - queue: Switchboard oracle queue
/// - clock_sysvar: Clock sysvar
/// - slothashes_sysvar: SlotHashes sysvar
/// - instructions_sysvar: Instructions sysvar
/// - state: Program state account
/// 
/// Execute Climate Payout:
/// - policy: Climate insurance policy account
/// - beneficiary: Payout recipient account
/// - quote: Oracle quote account
/// - queue: Switchboard oracle queue
/// - clock_sysvar: Clock sysvar
/// - slothashes_sysvar: SlotHashes sysvar
/// - instructions_sysvar: Instructions sysvar
/// - state: Program state account
pub struct ExampleWorkflow;

impl ExampleWorkflow {
    /// Example of creating instruction data for a payout
    /// Amount is encoded as little-endian u64
    pub fn create_payout_instruction_data(amount_lamports: u64) -> Vec<u8> {
        let mut data = vec![instruction_examples::EXECUTE_CLIMATE_PAYOUT];
        data.extend_from_slice(&amount_lamports.to_le_bytes());
        data
    }
    
    /// Example of creating instruction data for a policy
    /// This is simplified - in production you'd encode full policy parameters
    pub fn create_policy_instruction_data(
        policy_id: u64,
        policy_type: ExamplePolicyType,
        coverage_amount: u64,
        premium_amount: u64,
    ) -> Vec<u8> {
        let mut data = vec![instruction_examples::CREATE_CLIMATE_POLICY];
        data.extend_from_slice(&policy_id.to_le_bytes());
        data.push(policy_type as u8);
        data.extend_from_slice(&coverage_amount.to_le_bytes());
        data.extend_from_slice(&premium_amount.to_le_bytes());
        data
    }
}

/// Climate feed IDs that might be used with Switchboard Oracle
pub mod climate_feed_ids {
    /// Temperature feeds
    pub const TEMPERATURE_CELSIUS: &str = "temperature_celsius";
    pub const TEMP_DAILY_AVG: &str = "temp_daily_avg";
    pub const TEMP_DAILY_MAX: &str = "temp_daily_max";
    pub const TEMP_DAILY_MIN: &str = "temp_daily_min";
    
    /// Precipitation feeds
    pub const RAINFALL_MM: &str = "rainfall_mm";
    pub const PRECIPITATION_DAILY: &str = "precipitation_daily";
    pub const PRECIPITATION_MONTHLY: &str = "precipitation_monthly";
    
    /// Wind feeds
    pub const WIND_SPEED_MS: &str = "wind_speed_ms";
    pub const WIND_GUST_MS: &str = "wind_gust_ms";
    pub const WIND_DIRECTION_DEGREES: &str = "wind_direction_degrees";
    
    /// Humidity feeds
    pub const HUMIDITY_PERCENT: &str = "humidity_percent";
    pub const RELATIVE_HUMIDITY: &str = "relative_humidity";
    
    /// Pressure feeds
    pub const ATMOSPHERIC_PRESSURE: &str = "atmospheric_pressure_hpa";
    pub const BAROMETRIC_PRESSURE: &str = "barometric_pressure_mb";
}