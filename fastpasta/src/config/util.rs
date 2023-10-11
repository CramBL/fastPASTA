//! Contains the [UtilOpt] Trait for all small utility options set by a user, that are not specific to any other subfunctionality.

/// Trait for all small utility options that are not specific to any other trait
pub trait UtilOpt {
    /// Verbosity level of the logger: 0 = error, 1 = warn, 2 = info, 3 = debug, 4 = trace
    fn verbosity(&self) -> u8;
    /// Maximum number of errors to tolerate before exiting
    fn max_tolerate_errors(&self) -> u32;
    /// Set the exit code for if any errors are detected in the input data
    fn any_errors_exit_code(&self) -> Option<u8>;
    /// If set, error messages are not displayed
    fn mute_errors(&self) -> bool;
    /// Allows specifying any number of error codes to filter by
    fn error_code_filter(&self) -> Option<&[String]>;
}

impl<T> UtilOpt for &T
where
    T: UtilOpt,
{
    fn verbosity(&self) -> u8 {
        (*self).verbosity()
    }
    fn max_tolerate_errors(&self) -> u32 {
        (*self).max_tolerate_errors()
    }

    fn any_errors_exit_code(&self) -> Option<u8> {
        (*self).any_errors_exit_code()
    }

    fn mute_errors(&self) -> bool {
        (*self).mute_errors()
    }

    fn error_code_filter(&self) -> Option<&[String]> {
        (*self).error_code_filter()
    }
}

impl<T> UtilOpt for &mut T
where
    T: UtilOpt,
{
    fn verbosity(&self) -> u8 {
        (**self).verbosity()
    }
    fn max_tolerate_errors(&self) -> u32 {
        (**self).max_tolerate_errors()
    }
    fn any_errors_exit_code(&self) -> Option<u8> {
        (**self).any_errors_exit_code()
    }
    fn mute_errors(&self) -> bool {
        (**self).mute_errors()
    }
    fn error_code_filter(&self) -> Option<&[String]> {
        (**self).error_code_filter()
    }
}

impl<T> UtilOpt for Box<T>
where
    T: UtilOpt,
{
    fn verbosity(&self) -> u8 {
        (**self).verbosity()
    }

    fn max_tolerate_errors(&self) -> u32 {
        (**self).max_tolerate_errors()
    }

    fn any_errors_exit_code(&self) -> Option<u8> {
        (**self).any_errors_exit_code()
    }
    fn mute_errors(&self) -> bool {
        (**self).mute_errors()
    }
    fn error_code_filter(&self) -> Option<&[String]> {
        (**self).error_code_filter()
    }
}

impl<T> UtilOpt for std::sync::Arc<T>
where
    T: UtilOpt,
{
    fn verbosity(&self) -> u8 {
        (**self).verbosity()
    }

    fn max_tolerate_errors(&self) -> u32 {
        (**self).max_tolerate_errors()
    }

    fn any_errors_exit_code(&self) -> Option<u8> {
        (**self).any_errors_exit_code()
    }

    fn mute_errors(&self) -> bool {
        (**self).mute_errors()
    }
    fn error_code_filter(&self) -> Option<&[String]> {
        (**self).error_code_filter()
    }
}
