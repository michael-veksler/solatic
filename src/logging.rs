#[doc(hidden)]
#[macro_export]
macro_rules! __dprint_impl {
    ($printer:ident, $maybe_out:expr, $($tokens: tt)*) => {
        {
            // Why we use take() to overcome the borrow checker for dprint!(self.out, "{}", self.val):
            //   1. We pass $self.out as $maybe_out, which is a mutable borrow of self.
            //      accessing $self.val requires an immutable borrow of self, which is verboten.
            //   2. Why not format_args!: same issue, it needs both mutable and immutable borrows of self. needs an immutable reference to self.val, i.e., to immutable reference of self,
            //   3. Why not format!: we want to avoid the cost of heap allocation.
            let mut maybe_out = $maybe_out.take();
            if let Some(out) = maybe_out.as_mut() {
                use ::std::io::Write as _; // bring in macros that support owned write
                let _ = $printer!(out, $($tokens)*);
            }
            $maybe_out = maybe_out;
        }
    }
}
/// Debug print macro that prints only if the condition is true.
///
/// # Arguments
/// * `maybe_out` - A possible target for write!: Some(target) or None.
/// * `tokens`    - The tokens to print if the condition is true.
///
/// # Examples
///
/// ```rust
/// use solatic::dprint;
/// use std::io;
/// # fn work(condition: bool) {
/// let mut out: Option<&mut dyn io::Write> = Some(&mut io::stdout());
/// dprint!(out, "This will print if condition is true: {}\n", 42);
/// # }
/// ```
#[macro_export]
macro_rules! dprint {
    ($maybe_out:expr, $($tokens: tt)*) => {
        $crate::__dprint_impl!(write, $maybe_out, $($tokens)*)
    }
}

/// like `dprint!` but adds a newline at the end.
#[macro_export]
macro_rules! dprintln {
    ($maybe_out:expr, $($tokens: tt)*) => {
        $crate::__dprint_impl!(writeln, $maybe_out, $($tokens)*)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dprint_once_impl {
    ($printer:ident, $maybe_out:expr, $($tokens: tt)*) => {
        if let Some(out) = $maybe_out.as_mut() {
            static WAS_PRINTED: ::std::sync::atomic::AtomicBool = ::std::sync::atomic::AtomicBool::new(false);
            if !WAS_PRINTED.swap(true, ::std::sync::atomic::Ordering::Relaxed) {
                use ::std::io::Write as _; // bring in macros that support owned write
                let _ = $printer!(out, $($tokens)*);
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
/// use std::io;
/// # fn main() {
/// let mut out: Option<&mut dyn io::Write> = Some(&mut io::stdout());
/// dprint_once!(out, "This will print only once if condition is true: {}\n", 42);
/// # }
/// ```
#[macro_export]
macro_rules! dprint_once {
    ($maybe_out:expr, $($tokens: tt)*) => {
        $crate::__dprint_once_impl!(write, $maybe_out, $($tokens)*)
    }
}

#[macro_export]
macro_rules! dprintln_once {
    ($maybe_out:expr, $($tokens: tt)*) => {
        $crate::__dprint_once_impl!(writeln, $maybe_out, $($tokens)*)
    }
}
