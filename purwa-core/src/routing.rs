//! Route registration via [`inventory`] and Axum [`Router`].
//!
//! **WASM:** the `inventory` crate relies on linker sections; Purwa routing macros do not
//! support `wasm32` targets (see project README / AGENT.md).

use std::cmp::Reverse;
use std::collections::HashSet;

use axum::Router;
use axum::http::Method;

/// One row for `route:list` / [`format_route_table`]. Handlers register through
/// [`inventory::submit!`] (via Purwa proc-macros).
#[derive(Clone)]
pub struct RegisteredRoute {
    pub method: Method,
    pub path: &'static str,
    pub handler_label: &'static str,
    /// When several HTTP methods share one Axum route, only one entry carries `Some(install)`;
    /// the others are list-only (`None`).
    pub install: Option<fn(Router) -> Router>,
}

inventory::collect!(RegisteredRoute);

/// Merge all registered routes into a single [`Router`] (unit state).
///
/// Installers run in deterministic order: longer paths first so `/items/create` and
/// `/items/{id}/edit` win over `/items/{id}`.
pub fn router_from_inventory() -> Router {
    struct InstallEntry {
        path: &'static str,
        install: fn(Router) -> Router,
    }

    let mut entries: Vec<InstallEntry> = inventory::iter::<RegisteredRoute>
        .into_iter()
        .filter_map(|r| {
            r.install.map(|install| InstallEntry {
                path: r.path,
                install,
            })
        })
        .collect();

    let mut seen = HashSet::new();
    entries.retain(|e| seen.insert(e.install as usize));
    entries.sort_by_key(|e| Reverse(e.path.len()));

    let mut router = Router::new();
    for e in entries {
        router = (e.install)(router);
    }
    router
}

/// Iterator over registered route metadata (for tooling and tests).
pub fn route_descriptors() -> impl Iterator<Item = RouteDescriptor> + 'static {
    inventory::iter::<RegisteredRoute>
        .into_iter()
        .map(|r| RouteDescriptor {
            method: r.method.clone(),
            path: r.path,
            handler_label: r.handler_label,
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteDescriptor {
    pub method: Method,
    pub path: &'static str,
    pub handler_label: &'static str,
}

/// Pretty-print METHOD, path, and handler (PRD §11 table style).
pub fn format_route_table() -> String {
    let mut rows: Vec<_> = route_descriptors().collect();
    rows.sort_by(|a, b| {
        a.path
            .cmp(b.path)
            .then_with(|| a.method.as_str().cmp(b.method.as_str()))
    });

    let mut out = String::from("METHOD   PATH                              HANDLER\n");
    for d in rows {
        out.push_str(&format!(
            "{:<8} {:<33} {}\n",
            d.method.as_str(),
            d.path,
            d.handler_label
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inventory_builds() {
        let _ = router_from_inventory();
    }
}
