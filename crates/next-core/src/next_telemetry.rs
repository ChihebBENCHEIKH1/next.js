use phf::phf_map;
use turbo_rcstr::RcStr;

/// Summary of Next.js feature usage for a project, reported as telemetry.
///
/// Produced by `next_api::project::Project::project_feature_usage`. Entries cover:
/// - Boolean build/config flags (`1` if enabled, `0` if disabled), mirroring webpack's
///   `TelemetryPlugin`.
/// - Module imports (e.g. `next/image`, `next/font/google`): one count per unique importing module,
///   computed by walking the whole-app module graph.
///
/// The vector is sorted by `feature_name` for determinism.
#[turbo_tasks::value(shared)]
pub struct ProjectFeatureUsageSummary {
    pub features: Vec<(RcStr, u32)>,
}

/// Public feature specifier -> path suffix that identifies the resolved feature module.
///
/// Matched via `module.ident().path.path.ends_with(suffix)`. Mirrors the webpack
/// `FEATURE_MODULE_MAP` in
/// `packages/next/src/build/webpack/plugins/telemetry-plugin/telemetry-plugin.ts`.
pub static FEATURE_MODULE_PATH_SUFFIXES: phf::Map<&'static str, &'static str> = phf_map! {
    "next/image"        => "next/dist/shared/lib/image-external.js",
    "next/future/image" => "next/dist/client/future/image.js",
    "next/legacy/image" => "next/dist/client/legacy/image.js",
    "next/script"       => "next/dist/client/script.js",
    "next/dynamic"      => "next/dist/shared/lib/dynamic.js",
};

/// Public feature specifier -> substring that identifies the synthetic `target.css` virtual
/// module produced by the Next.js font loader transform.
///
/// The SWC transform in `crates/next-custom-transforms/src/transforms/fonts` rewrites
/// `import { Inter } from 'next/font/google'` into
/// `import inter from 'next/font/google/target.css?{...}'`, so the original `next/font/google`
/// specifier never appears in the module graph — we match on the synthesized virtual module
/// instead.
///
/// Note: webpack's equivalent uses regex + overwrite-on-match so the count collapses to roughly
/// 1-per-file. Our graph-walk sums unique `(parent, node)` pairs across all matching virtual
/// modules for a given feature, producing a truer "uses of this feature" count; the number may
/// differ slightly from webpack's for projects with many font calls per file.
pub static FEATURE_MODULE_IDENT_SUBSTRINGS: phf::Map<&'static str, &'static str> = phf_map! {
    "next/font/google"  => "/next/font/google/target.css",
    "next/font/local"   => "/next/font/local/target.css",
    "@next/font/google" => "/@next/font/google/target.css",
    "@next/font/local"  => "/@next/font/local/target.css",
};
