use turbo_rcstr::RcStr;
use turbo_tasks::Vc;
use turbopack_core::diagnostics::{Diagnostic, PlainBuildFeatureUsage};

/// Telemetry diagnostic reporting usage of a Next.js feature.
///
/// Emissions are aggregated by `feature_name` in
/// `crates/next-napi-bindings/src/next_api/utils.rs::get_diagnostics` before
/// crossing the NAPI boundary, so each feature produces exactly one
/// telemetry record per build — matching webpack's `TelemetryPlugin` shape.
///
/// Counts:
/// - Boolean config flags: `1` if enabled, `0` if disabled.
/// - Module imports (e.g. `next/image`): one emission per resolve of the feature module; the
///   aggregation step sums them.
#[turbo_tasks::value(shared)]
pub struct FeatureUsageTelemetry {
    pub feature_name: RcStr,
    pub invocation_count: u32,
}

impl FeatureUsageTelemetry {
    pub fn new(feature_name: RcStr, invocation_count: u32) -> Self {
        FeatureUsageTelemetry {
            feature_name,
            invocation_count,
        }
    }

    pub fn from_bool(feature_name: RcStr, enabled: bool) -> Self {
        FeatureUsageTelemetry::new(feature_name, if enabled { 1 } else { 0 })
    }
}

#[turbo_tasks::value_impl]
impl Diagnostic for FeatureUsageTelemetry {
    #[turbo_tasks::function]
    async fn into_plain(self: Vc<Self>) -> anyhow::Result<Vc<PlainBuildFeatureUsage>> {
        let this = self.await?;
        Ok(PlainBuildFeatureUsage {
            feature_name: this.feature_name.clone(),
            invocation_count: this.invocation_count,
        }
        .cell())
    }
}
