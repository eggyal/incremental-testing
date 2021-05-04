// The definitions in this module currently duplicate `rustc_codegen_ssa::coverageinfo::ffi`,
// which aligns with LLVM: but there's no reason these could not become more specific to this
// interface in the future.

use abi_stable::StableAbi;

#[repr(u8)]
#[derive(Clone, Copy, StableAbi)]
pub enum CounterKind {
    Zero,
    CounterValueReference,
    Expression,
}

/// A reference to an instance of an abstract "Counter" that will yield a value in a coverage
/// report. Note that `id` has different interpretations, depending on the `kind`:
///   * For `CounterKind::Zero`, `id` is assumed to be `0`
///   * For `CounterKind::CounterValueReference`,  `id` matches the `counter_id` of the injected
///     instrumentation counter (the `index` argument to the `__instrprof_increment()` call)
///   * For `CounterKind::Expression`, `id` is the index into the coverage map's array of
///     counter expressions.
#[repr(C)]
#[derive(Clone, Copy, StableAbi)]
pub struct Counter {
    pub kind: CounterKind,
    pub id: u32,
}

#[repr(u8)]
#[derive(Clone, Copy, StableAbi)]
pub enum ExprKind {
    Subtract,
    Add,
}

#[repr(C)]
#[derive(Clone, Copy, StableAbi)]
pub struct CounterExpression {
    pub kind: ExprKind,
    pub lhs: Counter,
    pub rhs: Counter,
}
