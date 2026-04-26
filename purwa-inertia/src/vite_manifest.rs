//! Parse Vite/Rollup `manifest.json` and emit `<script>` / `<link>` tags for full-page HTML.

use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Error building tags from a Vite manifest.
#[derive(Debug)]
pub enum ViteManifestError {
    Json(serde_json::Error),
    MissingEntry(String),
    Cycle,
}

impl std::fmt::Display for ViteManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViteManifestError::Json(e) => write!(f, "manifest JSON: {e}"),
            ViteManifestError::MissingEntry(k) => write!(f, "no manifest entry for key {k:?}"),
            ViteManifestError::Cycle => write!(f, "circular import in manifest"),
        }
    }
}

impl std::error::Error for ViteManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ViteManifestError::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ViteManifestError {
    fn from(e: serde_json::Error) -> Self {
        ViteManifestError::Json(e)
    }
}

#[derive(Debug, Deserialize)]
struct Chunk {
    file: String,
    #[serde(default)]
    imports: Vec<String>,
    #[serde(default)]
    css: Vec<String>,
}

fn dfs_js_order(
    chunks: &HashMap<String, Chunk>,
    key: &str,
    stack: &mut HashSet<String>,
    done: &mut HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), ViteManifestError> {
    if done.contains(key) {
        return Ok(());
    }
    if !stack.insert(key.to_string()) {
        return Err(ViteManifestError::Cycle);
    }
    let chunk = chunks
        .get(key)
        .ok_or_else(|| ViteManifestError::MissingEntry(key.to_string()))?;
    for imp in &chunk.imports {
        dfs_js_order(chunks, imp, stack, done, out)?;
    }
    stack.remove(key);
    done.insert(key.to_string());
    out.push(key.to_string());
    Ok(())
}

/// Build HTML tags for the JS entry (dependencies first) and associated CSS.
///
/// `manifest_json` is the contents of `public/.vite/manifest.json`.
/// `entry_key` is the Rollup input key, e.g. `"src/app.js"`.
pub fn html_tags_from_manifest(
    manifest_json: &str,
    entry_key: &str,
) -> Result<String, ViteManifestError> {
    let root: HashMap<String, Value> = serde_json::from_str(manifest_json)?;
    let mut chunks: HashMap<String, Chunk> = HashMap::with_capacity(root.len());
    for (k, v) in root {
        let c: Chunk = serde_json::from_value(v)?;
        chunks.insert(k, c);
    }

    let mut js_order = Vec::new();
    dfs_js_order(
        &chunks,
        entry_key,
        &mut HashSet::new(),
        &mut HashSet::new(),
        &mut js_order,
    )?;

    // CSS: follow same chunk order, dedupe by href
    let mut css_hrefs: Vec<String> = Vec::new();
    let mut css_seen: HashSet<String> = HashSet::new();
    for key in &js_order {
        let chunk = chunks.get(key).unwrap();
        for c in &chunk.css {
            if css_seen.insert(c.clone()) {
                css_hrefs.push(c.clone());
            }
        }
    }

    let mut out = String::new();
    for href in css_hrefs {
        out.push_str(&format!(
            r#"<link rel="stylesheet" href="/{}">"#,
            escape_attr(&href)
        ));
    }
    for key in &js_order {
        let chunk = chunks.get(key).unwrap();
        out.push_str(&format!(
            r#"<script type="module" src="/{}"></script>"#,
            escape_attr(&chunk.file)
        ));
    }
    Ok(out)
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

/// Dev server: Vite client + entry (see `PURWA_VITE_DEV_ORIGIN` in generated README).
pub fn html_tags_vite_dev(dev_origin: &str, entry_path: &str) -> String {
    let o = escape_attr(dev_origin.trim_end_matches('/'));
    let e = escape_attr(entry_path.trim_start_matches('/'));
    format!(
        r#"<script type="module" src="{o}/@vite/client"></script><script type="module" src="{o}/{e}"></script>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_js_and_css() {
        let manifest = r#"{
  "src/app.js": {
    "file": "assets/app-abc.js",
    "name": "app",
    "src": "src/app.js",
    "isEntry": true,
    "imports": ["_vendor-xyz.js"],
    "css": ["assets/app-def.css"]
  },
  "_vendor-xyz.js": {
    "file": "assets/vendor-xyz.js",
    "imports": []
  }
}"#;
        let tags = html_tags_from_manifest(manifest, "src/app.js").unwrap();
        let vendor_pos = tags.find("vendor-xyz").unwrap();
        let app_pos = tags.find("app-abc").unwrap();
        assert!(vendor_pos < app_pos, "vendor chunk must load before entry");
        assert!(tags.contains(r#"href="/assets/app-def.css""#));
    }
}
