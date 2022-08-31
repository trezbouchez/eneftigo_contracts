pub(crate) enum MarketplaceErrorCode {
    // seller
    InvalidUrl,
    SupplyTooHigh,
    PriceTooLow,
    PriceNotMultipleOfStep,
    WrongDateFormat,
    DateIntoThePast,
    DurationTooShort,
    InsufficientDeposit,
    // buyer
    InvalidOfferingStatus,
    OfferingDoesNotExist,
    CannotBuyFromYourself,
    NoSupplyLeft,
    DepositWontCoverPrice,
    NftMintingFailed
}

impl MarketplaceErrorCode {
    pub(crate) fn to_u16(&self) -> u16 {
        match self {
            MarketplaceErrorCode::InvalidUrl => 0,
            MarketplaceErrorCode::SupplyTooHigh => 1,
            MarketplaceErrorCode::PriceTooLow => 2,
            MarketplaceErrorCode::PriceNotMultipleOfStep => 3,
            MarketplaceErrorCode::WrongDateFormat => 4,
            MarketplaceErrorCode::DateIntoThePast => 5,
            MarketplaceErrorCode::DurationTooShort => 6,
            MarketplaceErrorCode::InsufficientDeposit => 7,
            MarketplaceErrorCode::InvalidOfferingStatus => 8,
            MarketplaceErrorCode::OfferingDoesNotExist => 9,
            MarketplaceErrorCode::CannotBuyFromYourself => 10,
            MarketplaceErrorCode::NoSupplyLeft => 11,
            MarketplaceErrorCode::DepositWontCoverPrice => 12,
            MarketplaceErrorCode::NftMintingFailed => 13,
        }
    }
}

pub(crate) struct MarketplaceError {
    code: u16,
    message: String,
}

impl MarketplaceError {
    pub(crate) fn new(code: MarketplaceErrorCode, message: &str) -> MarketplaceError {
        MarketplaceError { 
            code: code.to_u16(), 
            message: message.to_string(),
         }
    }

    // consumes self
    // typical usage:
    // MarketplaceError::new(MarketplaceErrorCode::NoSupplyLeft, "All NFTs have been sold").into_err()
    pub(crate) fn into_err<S>(self) -> Result<S,MarketplaceError> {
        Err(self)
    }
}

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return,
        }
    }
}