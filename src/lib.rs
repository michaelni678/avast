use std::collections::HashSet;

#[cfg(feature = "derive")]
pub use avast_derive::Validate;

/// Validity, returned by validators.
pub enum Validity {
    Valid,
    Invalid(Blacklist),
}

impl From<bool> for Validity {
    fn from(value: bool) -> Self {
        if value {
            Self::valid()
        } else {
            Self::invalid()
        }
    }
}

impl Validity {
    /// Create new valid variant.
    pub fn valid() -> Self {
        Self::Valid
    }
    /// Create new invalid variant.
    pub fn invalid() -> Self {
        Self::Invalid(Blacklist::default())
    }
    /// Push to blacklist if invalid.
    pub fn push_blacklist(
        mut self, 
        field_name: FieldName, 
        validator_name: ValidatorName,
    ) -> Self {
        match &mut self {
            Validity::Valid => {},
            Validity::Invalid(blacklist) => {
                blacklist.insert((field_name, validator_name));
            },
        }
        self
    }
    /// Combine validities.
    pub fn combine(&mut self, other: Self) {
        if let Validity::Invalid(blacklist) = self {
            if let Validity::Invalid(other_blacklist) = other {
                blacklist.extend(other_blacklist);
            }
        } else {
            *self = other;
        }
    }
}

/// A field name.
pub type FieldName = &'static str;

/// The validator's name.
pub type ValidatorName = &'static str;

/// Fields that failed validation.
pub type Blacklist = HashSet<(FieldName, ValidatorName)>;

/// The trait used for validation.
pub trait Validate {
    /// The context type.
    type Context;
    /// Validate.
    fn validate(&self, ctx: &Self::Context) -> Validity;
}

#[cfg(feature = "built-in")]
pub mod built_in {
    /// Built-in string validators.
    pub mod str {
        use crate::Validity;

        /// Validates all characters in a string are ascii.
        pub fn ascii<I: AsRef<str>>(item: I) -> Validity {
            item.as_ref().is_ascii().into()
        }

        /// Validates all characters in a string are alphanumeric.
        pub fn alphanumeric<I: AsRef<str>>(item: I) -> Validity {
            item.as_ref().chars().all(char::is_alphanumeric).into()
        }

        /// Validates a string is within the min and max length.
        pub fn length<I: AsRef<str>>(item: I, min: usize, max: usize) -> Validity {
            let len = item.as_ref().len();
            (len >= min && len <= max).into()
        }
    }

    /// Built-in number validators.
    pub mod num {
        use crate::Validity;

        /// Validates a number is within the min and max value.
        pub fn range<I: Ord, C>(item: &I, min: I, max: I) -> Validity {
            (*item >= min && *item <= max).into()
        }

        /// Validates a number is less than another value.
        pub fn less_than<I: Ord, C>(item: &I, other: I) -> Validity {
            (*item < other).into()
        }

        /// Validates a number is greater than another value.
        pub fn greater_than<I: Ord, C>(item: &I, other: I) -> Validity {
            (*item > other).into()
        }
    }

    /// Built-in option validators.
    pub mod option {
        use crate::Validity;

        /// Validates an option is `Some`.
        pub fn some<I>(item: &Option<I>) -> Validity {
            item.is_some().into()
        }

        /// Validates an option is `None`.
        pub fn none<I>(item: &Option<I>) -> Validity {
            item.is_none().into()
        }
    }

    /// Built-in miscellaneous validators.
    pub mod misc {
        use crate::{Validate, Validity};

        /// Validates two items are equal.
        pub fn eq<I: Eq>(item: &I, other: &I) -> Validity {
            (item == other).into()
        }

        /// Validates with an expession.
        pub fn expr<I>(_: &I, expr: impl Into<Validity>) -> Validity {
            expr.into()
        }

        /// Nested validation.
        pub fn validate<I: Validate<Context = C>, C>(item: &I, ctx: &C) -> Validity {
            item.validate(ctx)
        }
    }
}