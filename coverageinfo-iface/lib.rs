//! Interface definitions through which users (e.g. embedded JIT runtimes) can query foreign
//! libraries (e.g. their JIT host) for code coverage information.

use abi_stable::{
    erased_types::interfaces::IteratorInterface,
    external_types::crossbeam_channel::RSender,
    library::RootModule,
    sabi_trait,
    sabi_types::version::VersionStrings,
    std_types::{RBox, ROption, RVec, Tuple2},
    DynTrait, StableAbi,
};

mod coverageinfo;
pub use coverageinfo::*;

/// Root module interface, that is to be provided by the library (e.g. the JIT host).
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "CoverageInfoProviderRef")))]
pub struct CoverageInfoProvider {
    /// Generate an array of CounterExpressions, and an iterator over all `CounterRegion`s for the
    /// given `function`.
    pub get_expressions_and_counter_regions: extern "C" fn(
        function: Function,
    ) -> Tuple2<
        RVec<CounterExpression>,
        RVec<CounterRegion>,
    >,

    /// Request information on all MIR basic blocks updated since the previous compilation session.
    /// The information is projected through the callbacks provided in `request`.  `ProjectionId`s
    /// will be emitted at most once per request, even if they are projected from multiple updated
    /// blocks.
    ///
    /// For example, an incremental test runner would request that the updated basic blocks be
    /// projected to the (non-duplicated) identifiers of all tests that cover those blocks.
    pub project_updated_blocks: extern "C" fn(
        // A callback through which the library (e.g. the JIT host) can obtain from the user (e.g.
        // the embedded JIT runtime) the `BlockLookup` for a given `Function`; the library can
        // then use such `BlockLookup` to obtain from the user the `ProjectionId`s for a given
        // `BasicBlock`.
        projector_lookup: ProjectorLookupBox,

        // The channel through which the library (e.g. the JIT host) should send discovered
        // projections to the user (e.g. the embedded JIT runtime).
        tx: RSender<RVec<ProjectionId>>,
    ),
}

impl RootModule for CoverageInfoProviderRef {
    abi_stable::declare_root_module_statics! {CoverageInfoProviderRef}

    const BASE_NAME: &'static str = "coverageinfo-iface";
    const NAME: &'static str = "code coverage interface";
    const VERSION_STRINGS: VersionStrings = abi_stable::package_version_strings!();
}

/// Opaque identifier of a monomorphized function that is stable across compilation sessions.
#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, StableAbi)]
pub struct Function(u64);

impl Function {
    pub fn new(function: u64) -> Self {
        Function(function)
    }
}

/// Opaque identifier of a specific MIR basic block within some particular `Function`, that is
/// stable across compilation sessions provided all antecedent blocks in the normalized CFG are
/// unchanged.
#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, StableAbi)]
pub struct BasicBlock(u64);

impl BasicBlock {
    pub fn new(basic_block: u64) -> Self {
        BasicBlock(basic_block)
    }
}

/// Opaque representation of a projection identifier that is stable across compilation sessions.
#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, StableAbi)]
pub struct ProjectionId(u64);

impl ProjectionId {
    pub fn new(projection_id: u64) -> Self {
        ProjectionId(projection_id)
    }
}

/// A `Counter`, together with the `BasicBlock`s and `Region`s to which it relates.
#[repr(C)]
#[derive(StableAbi)]
pub struct CounterRegion {
    pub counter: Counter,
    pub basic_blocks: RVec<BasicBlock>,
    pub source_region: *const u8, //TODO
}

/// A callback through which the library (e.g. the JIT host) can obtain from the user (e.g. the
/// embedded JIT runtime) the `BlockLookup` for a given `Function`.
#[sabi_trait]
pub trait ProjectorLookup {
    fn lookup(&self, function: Function) -> ROption<BlockLookupRef<'_>>;
}

/// A callback through which the library (e.g. the JIT host) can obtain from the user (e.g. the
/// embedded JIR runtime) an iterator over all projection identifiers for the given `block_path`.
#[sabi_trait]
pub trait BlockLookup {
    fn lookup(
        &self,
        block_path: BasicBlock,
    ) -> DynTrait<'_, RBox<()>, IteratorInterface<ProjectionId>>;
}

// Exported FFI-safe trait objects for the above traits
pub type ProjectorLookupBox = ProjectorLookup_TO<'static, RBox<()>>;
pub type BlockLookupRef<'a> = BlockLookup_TO<'a, &'a ()>;
