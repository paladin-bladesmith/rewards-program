pub const RENT_DEBT_NUMERATOR: u64 = 11;
pub const RENT_DEBT_DENOMINATOR: u64 = 11;

pub const fn rent_debt(rent_paid: u64) -> u64 {
    rent_paid * RENT_DEBT_NUMERATOR / RENT_DEBT_DENOMINATOR
}
