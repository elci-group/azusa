//! Behavioural facet derivation (layers L6–L7 inputs).
//!
//! Classifies the behavioural surface of a captured state from its
//! filesystem listing and retained analysis text: test files, benches,
//! executable entry points, build scripts, test declarations, and the
//! public API symbols per Rust source file. A pure function of the state —
//! no I/O, no parsing beyond declaration-header inspection — so enrichment
//! stays deterministic.

use azusa_model::{BehaviourState, PublicSymbol, RepositoryState};

/// Derive the behaviour facet of a captured state.
#[must_use]
pub fn behaviour_of(state: &RepositoryState) -> BehaviourState {
    let mut behaviour = BehaviourState::default();

    for path in state.filesystem.files.keys() {
        if let Some(role) = classify_path(path) {
            match role {
                Role::Test => behaviour.test_files.push(path.clone()),
                Role::Bench => behaviour.bench_files.push(path.clone()),
                Role::EntryPoint => behaviour.entry_points.push(path.clone()),
                Role::BuildScript => behaviour.build_scripts.push(path.clone()),
            }
        }
    }

    for (path, text) in &state.captured_text {
        if is_rust_source(path) {
            if is_test_text(text) && !behaviour.test_files.contains(path) {
                behaviour.test_files.push(path.clone());
            }
            behaviour.test_declarations += count_lines_with(text, "#[test]");
            let module_prefix = module_path_from_file(path);
            let symbols = extract_public_symbols_in_module(text, &module_prefix);
            if !symbols.is_empty() {
                behaviour
                    .public_items_per_file
                    .insert(path.clone(), symbols.len() as u64);
                behaviour.public_symbols.insert(path.clone(), symbols);
            }
        }
    }

    behaviour
}

enum Role {
    Test,
    Bench,
    EntryPoint,
    BuildScript,
}

/// Path-based behavioural role of a file, if any.
fn classify_path(path: &str) -> Option<Role> {
    let segments: Vec<&str> = path.split('/').collect();
    let is_rust = is_rust_source(path);
    match segments.as_slice() {
        [.., "src", "main.rs"] => Some(Role::EntryPoint),
        [.., "src", "bin", _] if is_rust => Some(Role::EntryPoint),
        [.., "tests", _] if is_rust => Some(Role::Test),
        [.., "benches", _] if is_rust => Some(Role::Bench),
        [.., "build.rs"] => Some(Role::BuildScript),
        _ => None,
    }
}

/// Whether retained Rust text carries a unit-test module.
fn is_test_text(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("#[cfg(test)]")
    })
}

/// Whether a captured path is Rust source text.
fn is_rust_source(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
}

/// Count lines containing a marker at declaration position.
fn count_lines_with(text: &str, marker: &str) -> u64 {
    text.lines().filter(|line| line.contains(marker)).count() as u64
}

/// The item keyword of an unrestricted `pub` declaration, if any.
/// Restricted visibility (`pub(crate)`, `pub(super)`, `pub(in path)`) is
/// deliberately excluded: those items are not part of the external API.
fn public_decl_kind(trimmed: &str) -> Option<&'static str> {
    let rest = trimmed.strip_prefix("pub")?;
    if rest.starts_with('(') {
        return None; // restricted visibility
    }
    if !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None; // e.g. `published` — not a `pub` declaration
    }
    let keyword = rest.trim_start();
    for kind in [
        "fn", "struct", "enum", "trait", "mod", "const", "static", "type", "use",
    ] {
        if keyword.starts_with(kind)
            && keyword[kind.len()..]
                .chars()
                .next()
                .is_none_or(|c| c.is_whitespace() || c == ':')
        {
            return Some(kind);
        }
    }
    None
}

/// Whether a declaration header is complete: the signature has ended —
/// a `{` appeared (the body opens; it may also close on the same line,
/// as in `pub fn one() {}`) or the header ends with `;`.
fn header_is_complete(header: &str) -> bool {
    header.contains('{') || header.trim_end().ends_with(';')
}

/// Collapse whitespace runs so signatures compare stably across formatting.
fn normalize_signature(header: &str) -> String {
    header.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Crate-relative module path for a source file:
/// `crates/x/src/auth/deep.rs` → `crate::auth::deep`; `src/lib.rs` →
/// `crate`.
#[must_use]
fn module_path_from_file(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').collect();
    let start = segments
        .iter()
        .position(|segment| *segment == "src")
        .map_or(0, |position| position + 1);
    let mut modules: Vec<String> = Vec::new();
    for segment in &segments[start..] {
        match *segment {
            "lib.rs" | "main.rs" | "mod.rs" => break,
            other => {
                if let Some(stem) = other.strip_suffix(".rs") {
                    modules.push(stem.to_owned());
                    break;
                }
                modules.push((*other).to_owned());
            }
        }
    }
    if modules.is_empty() {
        "crate".to_owned()
    } else {
        format!("crate::{}", modules.join("::"))
    }
}

/// The name of a nested `mod NAME {` opening, if the line opens one.
fn nested_mod_open(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("mod ")?;
    let name_end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
    let name = &rest[..name_end];
    rest[name_end..]
        .trim_start()
        .starts_with('{')
        .then(|| name.to_owned())
}

/// Extract public API symbols from Rust source text.
///
/// Deterministic line/brace inspection: unrestricted `pub` declarations
/// become [`PublicSymbol`]s with their crate-relative fully qualified name
/// (module path from the file plus nested `mod` blocks), item kind and a
/// normalized signature. Multi-line signatures are accumulated until the
/// header closes. No semantic parsing — identity is lexical but stable.
#[must_use]
pub fn extract_public_symbols(text: &str) -> Vec<PublicSymbol> {
    extract_public_symbols_in_module(text, "crate")
}

/// Like [`extract_public_symbols`], with an explicit module prefix.
#[must_use]
pub fn extract_public_symbols_in_module(text: &str, module_prefix: &str) -> Vec<PublicSymbol> {
    let mut symbols: Vec<PublicSymbol> = Vec::new();
    let mut depth: usize = 0;
    // (mod name, depth after its opening brace)
    let mut mod_stack: Vec<(String, usize)> = Vec::new();
    let mut pending: Option<(String, String)> = None; // (kind, header)

    for line in text.lines() {
        let trimmed = line.trim_start();

        if let Some((kind, header)) = &mut pending {
            header.push(' ');
            header.push_str(trimmed);
            if header_is_complete(header) {
                let normalized = normalize_signature(header);
                if let Some(name) = declaration_name(kind, &normalized) {
                    let fqn = join_fqn(module_prefix, &mod_stack, &name);
                    symbols.push(PublicSymbol {
                        fqn,
                        kind: kind.clone(),
                        signature: normalized,
                    });
                }
                pending = None;
            }
            depth = update_depth(depth, line);
            pop_closed_mods(&mut mod_stack, depth);
            continue;
        }

        if let Some(kind) = public_decl_kind(trimmed) {
            depth = update_depth(depth, line);
            if header_is_complete(trimmed) {
                let normalized = normalize_signature(trimmed);
                if let Some(name) = declaration_name(kind, &normalized) {
                    let fqn = join_fqn(module_prefix, &mod_stack, &name);
                    symbols.push(PublicSymbol {
                        fqn,
                        kind: kind.to_owned(),
                        signature: normalized,
                    });
                }
            } else {
                pending = Some((kind.to_owned(), trimmed.to_owned()));
            }
            if kind == "mod" {
                if let Some(name) = nested_mod_open(trimmed) {
                    mod_stack.push((name, depth));
                }
            }
            pop_closed_mods(&mut mod_stack, depth);
            continue;
        }

        depth = update_depth(depth, line);
        if let Some(name) = nested_mod_open(trimmed) {
            mod_stack.push((name, depth));
        }
        pop_closed_mods(&mut mod_stack, depth);
    }

    symbols.sort();
    symbols
}

/// Net brace depth after one line (string literals are not tracked; the
/// result stays deterministic for a given input).
fn update_depth(depth: usize, line: &str) -> usize {
    let opened = line.matches('{').count();
    let closed = line.matches('}').count();
    if opened >= closed {
        depth + (opened - closed)
    } else {
        depth.saturating_sub(closed - opened)
    }
}

fn pop_closed_mods(mod_stack: &mut Vec<(String, usize)>, depth: usize) {
    while mod_stack
        .last()
        .is_some_and(|(_, open_depth)| depth < *open_depth)
    {
        mod_stack.pop();
    }
}

fn join_fqn(module_prefix: &str, mod_stack: &[(String, usize)], name: &str) -> String {
    let mut fqn = String::from(module_prefix);
    for (module, _) in mod_stack {
        fqn.push_str("::");
        fqn.push_str(module);
    }
    fqn.push_str("::");
    fqn.push_str(name);
    fqn
}

/// The declared item name from a normalized header (the token after the
/// kind keyword; for `use`, the final path segment or glob).
fn declaration_name(kind: &str, header: &str) -> Option<String> {
    let after_kind = header.split_once(kind)?.1;
    let mut tokens = after_kind.split_whitespace();
    let name = match kind {
        "use" => {
            let path = tokens.next()?;
            let last = path.rsplit("::").next().unwrap_or(path);
            return Some(last.trim_end_matches(';').to_owned());
        }
        _ => tokens.next()?,
    };
    let name: String = name
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() { None } else { Some(name) }
}

#[cfg(test)]
mod tests {

    use super::*;
    use azusa_model::{FileEntry, FileKind, ProviderKind, content_hash};

    fn entry(content: &str) -> FileEntry {
        FileEntry {
            hash: content_hash(content.as_bytes()),
            size: content.len() as u64,
            executable: false,
            kind: FileKind::File,
            lines: Some(content.lines().count() as u64),
        }
    }

    fn state(files: &[(&str, &str)]) -> RepositoryState {
        let mut state = RepositoryState::empty(ProviderKind::Git, "fixture");
        for (path, text) in files {
            state
                .filesystem
                .files
                .insert((*path).to_owned(), entry(text));
            state
                .captured_text
                .insert((*path).to_owned(), (*text).to_owned());
        }
        state
    }

    #[test]
    fn classifies_tests_benches_entry_points_and_build_scripts() {
        let behaviour = behaviour_of(&state(&[
            ("src/main.rs", "fn main() {}\n"),
            ("crates/app/src/lib.rs", "pub fn a() {}\n"),
            ("crates/app/src/bin/tool.rs", "fn main() {}\n"),
            ("tests/cli.rs", "#[test]\nfn t() {}\n"),
            ("benches/wide.rs", "fn bench() {}\n"),
            ("crates/app/build.rs", "fn main() {}\n"),
        ]));
        assert!(behaviour.entry_points.contains(&"src/main.rs".to_owned()));
        assert!(
            behaviour
                .entry_points
                .contains(&"crates/app/src/bin/tool.rs".to_owned())
        );
        assert!(behaviour.test_files.contains(&"tests/cli.rs".to_owned()));
        assert!(
            behaviour
                .bench_files
                .contains(&"benches/wide.rs".to_owned())
        );
        assert!(
            behaviour
                .build_scripts
                .contains(&"crates/app/build.rs".to_owned())
        );
    }

    #[test]
    fn counts_test_declarations_and_extracts_symbols() {
        let lib = "pub fn one() {}\npub struct Two;\nfn hidden() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n    #[test]\n    fn u() {}\n}\n";
        let behaviour = behaviour_of(&state(&[("crates/app/src/lib.rs", lib)]));
        assert_eq!(behaviour.test_declarations, 2);
        assert_eq!(
            behaviour.public_items_per_file.get("crates/app/src/lib.rs"),
            Some(&2)
        );
        let symbols = &behaviour.public_symbols["crates/app/src/lib.rs"];
        // Sorted by fully qualified name (byte order: `Two` < `one`).
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].fqn, "crate::Two");
        assert_eq!(symbols[0].kind, "struct");
        assert_eq!(symbols[1].fqn, "crate::one");
        assert_eq!(symbols[1].kind, "fn");
        // The lib carries a #[cfg(test)] module, so it registers as test-bearing.
        assert!(
            behaviour
                .test_files
                .contains(&"crates/app/src/lib.rs".to_owned())
        );
    }

    #[test]
    fn signature_changes_are_visible_in_symbols() {
        let before = "pub fn foo(x: u32) {}\n";
        let after = "pub fn foo(x: String) {}\n";
        let a = extract_public_symbols(before);
        let b = extract_public_symbols(after);
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 1);
        assert_eq!(a[0].fqn, b[0].fqn);
        assert_ne!(a[0].signature, b[0].signature, "signature must differ");
    }

    #[test]
    fn restricted_visibility_is_excluded() {
        let text = "pub(crate) fn internal() {}\npub(super) struct Semi;\npub fn real() {}\n";
        let symbols = extract_public_symbols(text);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].fqn, "crate::real");
    }

    #[test]
    fn nested_modules_extend_the_fqn() {
        let text =
            "mod outer {\n    mod inner {\n        pub fn deep() {}\n    }\n}\npub mod api {}\n";
        let symbols = extract_public_symbols(text);
        let fqns: Vec<&str> = symbols.iter().map(|s| s.fqn.as_str()).collect();
        assert!(fqns.contains(&"crate::outer::inner::deep"));
        assert!(fqns.contains(&"crate::api"));
    }

    #[test]
    fn multiline_signatures_normalize() {
        let text = "pub fn wide(\n    a: u32,\n    b: String,\n) -> u32 {\n    a as u32\n}\n";
        let symbols = extract_public_symbols(text);
        assert_eq!(symbols.len(), 1);
        assert_eq!(
            symbols[0].signature,
            "pub fn wide( a: u32, b: String, ) -> u32 {"
        );
    }

    #[test]
    fn use_reexports_take_the_last_segment() {
        let symbols = extract_public_symbols("pub use crate::inner::Widget;\n");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].kind, "use");
        assert_eq!(symbols[0].fqn, "crate::Widget");
    }

    #[test]
    fn module_path_follows_the_file() {
        assert_eq!(module_path_from_file("src/lib.rs"), "crate");
        assert_eq!(
            module_path_from_file("crates/app/src/auth/mod.rs"),
            "crate::auth"
        );
        assert_eq!(
            module_path_from_file("crates/app/src/auth/deep.rs"),
            "crate::auth::deep"
        );
    }

    #[test]
    fn empty_state_yields_empty_facet() {
        let behaviour = behaviour_of(&state(&[("docs/readme.md", "text\n")]));
        assert!(behaviour.test_files.is_empty(), "no tests expected");
        assert!(
            behaviour.public_items_per_file.is_empty(),
            "no public items expected"
        );
        assert!(behaviour.public_symbols.is_empty());
        assert_eq!(behaviour.test_declarations, 0);
    }
}
