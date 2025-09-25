# AMOCA Climate Insurance

A decentralized parametric climate insurance protocol built on Solana using the Anchor framework. AMOCA provides automated climate risk protection through verifiable environmental data from oracles, enabling instant payouts when predefined climate thresholds are met.

## 🌍 Overview

Climate change poses increasing risks to agriculture, property, and livelihoods worldwide. Traditional insurance is slow, expensive, and often inaccessible to those who need it most. AMOCA Climate Insurance solves this through:

- **Parametric Triggers**: Automatic payouts based on objective climate data (rainfall, temperature, wind speed, etc.)
- **Oracle Integration**: Real-time data from satellites, weather stations, and IoT sensors
- **Instant Settlements**: Smart contract-based payouts without lengthy claims processes
- **Global Accessibility**: Decentralized protocol accessible to anyone worldwide
- **Transparent Operations**: All policies, triggers, and payouts are verifiable on-chain

## 🏗️ Architecture

### Core Components

1. **Climate Policies**: Smart contracts defining coverage terms, geographic bounds, and trigger conditions
2. **Oracle Network**: Decentralized data providers submitting verified climate information
3. **Risk Pool**: Automated treasury managing premiums and payouts
4. **Trigger Engine**: Parametric evaluation system for automated claim processing

### Supported Climate Risks

- 🌧️ **Drought Protection**: Based on rainfall measurements
- 🌊 **Flood Insurance**: Water level and precipitation monitoring  
- 🌪️ **Hurricane Coverage**: Wind speed and pressure tracking
- 🌾 **Agricultural Climate**: Multi-parameter crop protection
- 🔥 **Wildfire Protection**: Fire proximity and weather conditions
- 🌊 **Sea Level Rise**: Coastal monitoring and protection
- 🌡️ **Extreme Temperature**: Heat/cold wave protection

## 📋 Features

### For Policyholders

- Create customized climate insurance policies
- Set geographic coverage areas and risk parameters
- Deposit premiums to activate coverage
- Receive automatic payouts when triggers are met
- Monitor policy status and climate data in real-time

### For Oracle Providers

- Submit verified climate data from multiple sources
- Build reputation through consistent, accurate reporting
- Earn rewards for providing high-quality data
- Participate in decentralized consensus mechanisms

### For Administrators

- Pause/unpause program operations
- Monitor global insurance metrics
- Manage oracle provider registrations
- Oversee risk pool operations

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustlang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor Framework](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/) and [Yarn](https://yarnpkg.com/)

### Installation

1. **Clone the repository**

   ```bash
   git clone https://github.com/your-org/amoca-climate-insurance.git
   cd amoca-climate-insurance
   ```

2. **Install dependencies**

   ```bash
   yarn install
   ```

3. **Build the program**

   ```bash
   anchor build
   ```

4. **Deploy to localnet**

   ```bash
   anchor deploy
   ```

5. **Run tests**

   ```bash
   anchor test
   ```

### Configuration

Update `Anchor.toml` with your desired network settings:

```toml
[features]
seeds = false
skip-lint = false

[programs.localnet]
amoca_climate_insurance = "8a2BSK86azg8kL6Cbd2wvEswnn2eKyS3CSZSgXpfTzTc"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "Localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```

## 📚 Program Instructions

### Core Instructions

#### `initialize`

Initialize the global program state and risk pool.

**Accounts:**

- `authority` - Program authority (signer)
- `global_state` - Global state PDA
- `system_program` - Solana System Program

#### `create_climate_policy`

Create a new parametric climate insurance policy.

**Parameters:**

- `params: PolicyParams` - Policy configuration including:
  - `policy_type` - Type of climate risk (drought, flood, etc.)
  - `geographic_bounds` - Coverage area coordinates
  - `trigger_conditions` - Climate thresholds for payouts
  - `coverage_amount` - Maximum payout amount
  - `premium_amount` - Required premium payment
  - `end_timestamp` - Policy expiration time

**Accounts:**

- `owner` - Policy owner (signer)
- `policy` - Policy account PDA
- `global_state` - Global state account

#### `deposit_premium`

Deposit premium to activate a climate insurance policy.

**Parameters:**

- `amount: u64` - Premium amount to deposit

**Accounts:**

- `owner` - Policy owner (signer)
- `policy` - Policy account
- `user_token_account` - Owner's token account
- `risk_pool_token_account` - Risk pool token account
- `token_program` - SPL Token Program

#### `submit_climate_data`

Submit verified climate data from oracle sources.

**Parameters:**

- `data_points: Vec<ClimateDataPoint>` - Array of climate measurements

**Accounts:**

- `oracle_provider` - Oracle provider (signer)
- `oracle_data` - Oracle data account
- `global_state` - Global state account

#### `evaluate_climate_trigger`

Evaluate policy trigger conditions against current climate data.

**Accounts:**

- `evaluator` - Trigger evaluator (signer)
- `policy` - Policy account
- `oracle_data_accounts` - Oracle data accounts for evaluation
- `global_state` - Global state account

#### `execute_climate_payout`

Execute automatic payout when trigger conditions are met.

**Parameters:**

- `payout_amount: u64` - Amount to pay out

**Accounts:**

- `executor` - Payout executor (signer)
- `policy` - Policy account
- `policyholder_token_account` - Recipient token account
- `risk_pool_token_account` - Risk pool token account
- `risk_pool_pda` - Risk pool PDA signer
- `token_program` - SPL Token Program

### Admin Instructions

#### `pause_program` / `unpause_program`

Emergency controls for program operations.

**Accounts:**

- `authority` - Program authority (signer)
- `global_state` - Global state account

## 🧪 Testing

The test suite covers all major functionality:

```bash
# Run all tests
anchor test

# Run specific test file
anchor test --file tests/amoca-climate-insurance.ts

# Run with verbose output
anchor test --verbose
```

### Test Coverage

- ✅ Program initialization
- ✅ Policy creation and validation
- ✅ Premium deposits and token transfers
- ✅ Oracle data submission
- ✅ Trigger evaluation logic
- ✅ Payout execution
- ✅ Admin controls
- ✅ Security and edge cases

## 🏛️ Program Data Structures

### `ClimatePolicy`

```rust
pub struct ClimatePolicy {
    pub bump: u8,
    pub owner: Pubkey,
    pub status: PolicyStatus,
    pub policy_type: ClimateRiskType,
    pub geographic_bounds: GeoBounds,
    pub trigger_thresholds: TriggerConditions,
    pub oracle_sources: Vec<Pubkey>,
    pub coverage_amount: u64,
    pub premium_amount: u64,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    // ... additional fields
}
```

### `ClimateDataPoint`

```rust
pub struct ClimateDataPoint {
    pub data_type: ClimateDataType,
    pub location: GeographicCoordinate,
    pub value: f64,
    pub timestamp: i64,
    pub confidence_level: u8,
    pub source_id: Pubkey,
    pub verification_hash: Vec<u8>,
}
```

### `TriggerConditions`

```rust
pub struct TriggerConditions {
    pub rainfall_threshold: Option<f64>,
    pub temperature_threshold: Option<f64>,
    pub wind_speed_threshold: Option<f64>,
    pub water_level_threshold: Option<f64>,
    pub fire_proximity_threshold: Option<f64>,
    pub measurement_period: u32,
    pub minimum_duration: u32,
}
```

## 🔒 Security Features

### Access Controls

- Program authority for admin operations
- Policy owner restrictions for policy management
- Oracle provider authentication for data submission
- PDA-based account validation

### Validation Checks

- Geographic bounds validation (-90° to 90° latitude, -180° to 180° longitude)
- Timestamp validation (data recency requirements)
- Confidence level thresholds for oracle data
- Premium and coverage amount validation
- Math overflow protection

### Emergency Controls

- Program pause/unpause functionality
- Administrative override capabilities
- Oracle reputation management
- Risk pool protection mechanisms

## 🌐 Oracle Integration

### Supported Oracle Types

- **Chainlink Weather**: Decentralized weather data
- **Pyth Satellite**: High-frequency satellite data
- **NASA MODIS**: Earth observation data
- **Weather Stations**: Ground-based measurements
- **IoT Sensors**: Real-time environmental monitoring
- **Switchboard Network**: Cross-chain oracle data

### Data Quality Assurance

- Multi-oracle consensus requirements
- Confidence level scoring (0-100%)
- Reputation-based oracle weighting
- Cryptographic verification hashes
- Temporal validation (data recency checks)

## 📊 Economics

### Premium Calculation

Premiums are calculated based on:

- Historical climate data for the region
- Coverage amount and duration
- Risk assessment algorithms
- Oracle data quality and availability
- Market conditions and pool reserves

### Payout Formulas

- **Linear Scale**: Proportional payout based on deviation from threshold
- **Step Function**: Fixed payouts at specific trigger levels  
- **Exponential**: Accelerating payouts for extreme events
- **Composite**: Multi-parameter weighted calculations

### Risk Pool Management

- Automated premium collection
- Dynamic reserve requirements
- Diversification across risk types and geographies
- Surplus distribution mechanisms

## 🚨 Error Codes

Common error codes and their meanings:

| Code | Error | Description |
|------|-------|-------------|
| 6000 | `InvalidCoverageAmount` | Coverage amount must be greater than 0 |
| 6001 | `InvalidPolicyDuration` | End timestamp must be in the future |
| 6002 | `InvalidPremiumAmount` | Premium amount must be greater than 0 |
| 6003 | `InvalidGeographicBounds` | Latitude/longitude values out of range |
| 6004 | `PolicyAlreadyActive` | Cannot modify active policy |
| 6005 | `InsufficientPremium` | Premium payment below required amount |
| 6006 | `OracleNotAuthorized` | Oracle provider not authorized |
| 6007 | `InvalidOracleData` | Invalid or malformed oracle data |
| 6008 | `StaleOracleData` | Oracle data too old to be valid |
| 6009 | `LowConfidenceData` | Oracle confidence level below threshold |
| 6010 | `PolicyNotActive` | Policy must be active for operation |
| 6011 | `PolicyExpired` | Policy has passed expiration date |
| 6012 | `TriggerNotMet` | Trigger conditions not satisfied |
| 6013 | `InvalidPayoutAmount` | Payout amount invalid or excessive |
| 6014 | `MathOverflow` | Arithmetic operation overflow |
| 6015 | `Unauthorized` | Insufficient permissions |
| 6016 | `ProgramPaused` | Program operations are paused |

## 🛣️ Roadmap

### Phase 1: Core Infrastructure ✅

- [x] Basic policy creation and management
- [x] Premium deposits and risk pool
- [x] Oracle data submission
- [x] Simple trigger evaluation
- [x] Manual payout execution

### Phase 2: Advanced Features 🚧

- [ ] Multi-oracle consensus mechanisms
- [ ] Advanced payout calculations
- [ ] Automated keeper network
- [ ] Policy transferability
- [ ] Staking and governance

### Phase 3: Ecosystem Expansion 📋

- [ ] Mobile applications
- [ ] Web dashboard
- [ ] API integrations
- [ ] Partner oracle networks
- [ ] Cross-chain compatibility

### Phase 4: Enterprise Features 📋

- [ ] Institutional risk pools
- [ ] Reinsurance protocols
- [ ] Regulatory compliance tools
- [ ] Enterprise APIs
- [ ] White-label solutions

## 🤝 Contributing

We welcome contributions from developers, climate scientists, insurance experts, and community members!

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

### Areas for Contribution

- Oracle integrations and data sources
- Advanced trigger logic and calculations
- User interface and experience improvements
- Testing and security audits
- Documentation and tutorials

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Solana Labs for the robust blockchain platform
- Anchor Framework for excellent Solana development tools
- Climate data providers and oracle networks
- Insurance industry experts and advisors
- Open source community contributors

## 📞 Support

- **Documentation**: [docs.amoca.climate](https://docs.amoca.climate)
- **Discord**: [discord.gg/amoca](https://discord.gg/amoca)
- **Twitter**: [@AmocaClimate](https://twitter.com/AmocaClimate)
- **Email**: [support@amoca.climate](mailto:support@amoca.climate)

---

**Disclaimer**: This software is experimental and under development. Use at your own risk. Climate insurance involves financial instruments and regulatory considerations that vary by jurisdiction. Please consult with legal and financial advisors before deployment in production environments.
