/// Debug print macro that prints only if the condition is true.
///
/// # Arguments
/// * `cond` - The condition to check before printing.
/// * `tokens` - The tokens to print if the condition is true.
///
/// # Examples
///
/// ```rust
/// use solatic::dprint;
/// # fn main() {
/// let condition = true;
/// dprint!(condition, "This will print if condition is true: {}\n", 42);
/// # }
/// ```
#[macro_export]
macro_rules! dprint {
    ($cond:expr, $($tokens: tt)*) => {
        if $cond {
            eprint!($($tokens)*);
        }
    };
}

/// like `dprint!` but adds a newline at the end.
#[macro_export]
macro_rules! dprintln {
    ($cond:expr, $($tokens: tt)*) => {
        if $cond {
            eprintln!($($tokens)*);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dprint_once_impl {
    ($printer:ident, $cond:expr, $($tokens: tt)*) => {
        if $cond {
            use ::std::sync::atomic::{AtomicBool, Ordering};
            static WAS_PRINTED: AtomicBool = AtomicBool::new(false);
            if !WAS_PRINTED.swap(true, Ordering::Relaxed) {
                $printer!($($tokens)*);
            }
        }
    };
}

/// Debug print macro that prints only once and if the condition is true.
///
/// # Arguments
/// * `cond` - The condition to check before printing.
/// * `tokens` - The tokens to print if the condition is true.
///
/// # Examples
///
/// ```rust
/// use solatic::dprint_once;
/// # fn main() {
/// let condition = true;
/// dprint_once!(condition, "This will print only once if condition is true: {}\n", 42);
/// # }
/// ```
#[macro_export]
macro_rules! dprint_once {
    ($cond:expr, $($tokens: tt)*) => {
        $crate::__dprint_once_impl!(eprint, $cond, $($tokens)*);
    };
}

#[macro_export]
macro_rules! dprintln_once {
    ($cond:expr, $($tokens: tt)*) => {
        $crate::__dprint_once_impl!(eprintln, $cond, $($tokens)*);
    };
}
