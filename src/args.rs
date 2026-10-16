use clap::{Args, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LiteralChoice {
    /// Reuse the previous latest assignment to this variable
    #[default]
    Phase,

    /// Use the negation of the previous latest assignment to this variable
    AntiPhase,

    /// Always choose a False assignment
    False,

    /// Always choose a True assignment
    True,
}

#[derive(Args, Debug, Clone, Copy, Default, PartialEq)]
pub struct SolverArgs {
    /// Print the decisions, the backtracks, and the literal propagated after conflict.
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Choose how to assign variables when making a decision.
    #[arg(long, value_enum, default_value_t)]
    pub choose_literal: LiteralChoice,
}
