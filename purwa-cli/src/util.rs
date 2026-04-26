//! Filesystem helpers for codegen.

use std::io;
use std::path::Path;

#[derive(Clone, Copy)]
pub struct GlobalOpts {
    pub verbose: bool,
    pub dry_run: bool,
}

pub fn write_output(path: &Path, content: &str, opts: GlobalOpts) -> io::Result<()> {
    if opts.dry_run {
        println!("--- {} ---\n{}\n", path.display(), content);
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    if opts.verbose {
        eprintln!("Wrote {}", path.display());
    }
    Ok(())
}
