use std::cmp::Ordering;

use anyhow::Result;
use async_trait::async_trait;
use auto_hash_map::AutoSet;
use turbo_rcstr::RcStr;
use turbo_tasks::{CollectiblesSource, ResolvedVc, Upcast, Vc, emit};

/// A diagnostic signal surfaced from Turbopack to the host (typically Next.js).
///
/// Today the only shape we carry is a build-feature-usage telemetry record.
/// Diagnostics are emitted via [`DiagnosticExt::emit`] and collected as
/// turbo-tasks collectibles. Consumers call [`DiagnosticContextExt::peek_diagnostics`]
/// on the operation source and read back a set of [`ResolvedVc<Box<dyn Diagnostic>>`].
/// Since collectibles must be trait objects, `Diagnostic` is a trait — but it
/// only has one method so implementors materialize their state in a single task
/// call.
///
/// If we ever grow a second diagnostic shape, promote [`PlainBuildFeatureUsage`]
/// into a tagged enum and update the NAPI layer accordingly.
#[turbo_tasks::value_trait]
pub trait Diagnostic {
    /// Convert the diagnostic into its plain, NAPI-boundary-ready form.
    #[turbo_tasks::function]
    fn into_plain(self: Vc<Self>) -> Vc<PlainBuildFeatureUsage>;
}

/// The plain, serializable form of a build-feature-usage diagnostic, matching
/// the `NEXT_BUILD_FEATURE_USAGE` telemetry event shape.
///
/// Counts follow webpack's `TelemetryPlugin` semantics:
/// - Boolean config flags: `1` if enabled, `0` if disabled.
/// - Module imports (e.g. `next/image`): aggregated sum of per-resolve emissions.
#[turbo_tasks::value(shared, serialization = "none")]
#[derive(Clone, Debug)]
pub struct PlainBuildFeatureUsage {
    pub feature_name: RcStr,
    pub invocation_count: u32,
}

impl Ord for PlainBuildFeatureUsage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.feature_name
            .cmp(&other.feature_name)
            .then_with(|| self.invocation_count.cmp(&other.invocation_count))
    }
}

impl PartialOrd for PlainBuildFeatureUsage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub trait DiagnosticExt {
    fn emit(self);
}

impl<T> DiagnosticExt for ResolvedVc<T>
where
    T: Upcast<Box<dyn Diagnostic>>,
{
    fn emit(self) {
        let diagnostic = ResolvedVc::upcast_non_strict::<Box<dyn Diagnostic>>(self);
        emit(diagnostic);
    }
}

#[async_trait]
pub trait DiagnosticContextExt
where
    Self: Sized,
{
    async fn peek_diagnostics(self) -> Result<CapturedDiagnostics>;
}

#[async_trait]
impl<T> DiagnosticContextExt for T
where
    T: CollectiblesSource + Copy + Send,
{
    async fn peek_diagnostics(self) -> Result<CapturedDiagnostics> {
        Ok(CapturedDiagnostics {
            diagnostics: self.peek_collectibles(),
        })
    }
}

/// A list of diagnostics captured with [`DiagnosticContextExt::peek_diagnostics`].
#[derive(Debug)]
#[turbo_tasks::value]
pub struct CapturedDiagnostics {
    pub diagnostics: AutoSet<ResolvedVc<Box<dyn Diagnostic>>>,
}
