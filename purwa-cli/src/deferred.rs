//! Stub commands deferred past Sprint 8 (see TASK.md).

pub fn print_deferred(cmd: &str) {
    eprintln!("`empu {cmd}` is not implemented yet.");
    eprintln!(
        "Tracked under Purwa Sprint roadmap (TASK.md): seeders, policies, `db:seed`, full `inertia:setup`."
    );
    eprintln!("Repository: https://github.com/sangkan/purwa (file issues for prioritization).");
}
