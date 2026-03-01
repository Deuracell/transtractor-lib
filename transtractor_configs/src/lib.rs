//! Bundled bank statement parsing configurations for transtractor.
//!
//! This crate provides all default configurations as compile-time embedded strings.
//! Configurations are loaded via `include_str!` at compile time, making them available
//! as constants without any runtime file I/O.

/// All bundled configuration JSON strings, embedded at compile time.
///
/// Each configuration defines parsing rules for a specific bank/account type.
/// Supported configurations:
/// - au__cba__credit_card__1: Commonwealth Bank of Australia (Credit Card)
/// - au__cba__debit__1: Commonwealth Bank of Australia (Debit/Savings)
/// - au__cba__loan__1: Commonwealth Bank of Australia (Loan)
/// - au__nab__classic_banking__1: National Australia Bank (Classic Banking)
pub const CONFIGS: &[&str] = &[
    include_str!("../configs/au/cba__credit_card__1.json"),
    include_str!("../configs/au/cba__debit__1.json"),
    include_str!("../configs/au/cba__loan__1.json"),
    include_str!("../configs/au/nab__classic_banking__1.json"),
];
