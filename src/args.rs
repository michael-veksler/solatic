use clap::{Args, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LiteralChoice {
    /// Always choose a False assignment
    #[default]
    False,

    /// Always choose a True assignment
    True,
}

#[derive(Args, Debug, Clone, Copy, Default)]
pub struct SolverArgs {
    /// Print the decisions, the backtracks, and the literal propagated after conflict.
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Choose how to assign variables when making a decision.
    #[arg(long, value_enum, default_value_t)]
    pub choose_literal: LiteralChoice,
}

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
            print!($($tokens)*);
        }
    };
}

/// like `dprint!` but adds a newline at the end.
#[macro_export]
macro_rules! dprintln {
    ($cond:expr, $($tokens: tt)*) => {
        if $cond {
            println!($($tokens)*);
        }
    };
}
