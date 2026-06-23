use std::{
    collections::{HashMap, HashSet, VecDeque},
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

mod color {
    pub const RESET: &str = "\x1b[0m";

    pub const BOLD: &str = "\x1b[1m";

    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m";
}

static mut VERBOSE: bool = false;
fn set_verbose(v: bool) {
    unsafe { VERBOSE = v };
}
fn is_verbose() -> bool {
    unsafe { VERBOSE }
}



macro_rules! vprintln {
    ($($arg:tt)*) => {
        if is_verbose() {
            println!(
                "{}[calmake]{} {}",
                crate::color::YELLOW,
                crate::color::RESET,
                format!($($arg)*)
            );
        }
    };
}

macro_rules! erprintln {
    ($($arg:tt)*) => {
        if is_verbose() {
            println!(
                "{}[calmake]{} {}",
                crate::color::RED,
                crate::color::RESET,
                format!($($arg)*)
            );
        }
    };
}


fn cmd_graph() -> anyhow::Result<()> {
    let config_path = Path::new("build.cal");
    if !config_path.exists() {
        anyhow::bail!("build.cal not found in current directory");
    }

    let config_str = fs::read_to_string(config_path)?;
    let config = parse_config(&config_str)?;
    let graph = BuildGraph::from_config(&config)?;

    println!("digraph calmake {{");
    for (name, node) in &graph.targets {
        // Target → dependency edges
        for dep in &node.deps {
            println!("    \"{}\" -> \"{}\";", dep, name);
        }

        // Target → source edges
        for src in &node.sources {
            let s = src.to_string_lossy();
            println!("    \"{}\" -> \"{}\";", name, s);
        }
    }
    println!("}}");

    Ok(())
}


fn main() {
    if let Err(e) = entry() {
        eprintln!(
            "{}[calmake] error:{} {e}",
            color::BRIGHT_RED,
            color::RESET
        );
        std::process::exit(1);
    }
}

fn entry() -> anyhow::Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let verbose = args.iter().any(|a| a == "-v" || a == "--verbose");
    set_verbose(verbose);
    if verbose {
        args.retain(|a| a != "-v" && a != "--verbose");
        println!(
            "{}[calmake]{} {}-v or --verbose was passed! Verbose mode is now enabled!{}",
            color::CYAN,
            color::RESET,
            color::BRIGHT_YELLOW,
            color::RESET
        );
    }

    let mut it = args.into_iter();
    match it.next().as_deref() {
        Some("init") => {
            let name = it.next();
            cmd_init(name.as_deref())
        }
        Some("graph") => cmd_graph(),
        Some("build") => cmd_build(),
        Some("clean") => {
            let name = it.next();
            cmd_clean(name.as_deref())
        }
        Some(other) => anyhow::bail!("unknown command `{other}` (use `calmake`, `calmake init`, or `calmake graph` or `calmake build`)"),
        None => anyhow::bail!("no command given (use `calmake`, `calmake init`, or `calmake graph` or `calmake build`)"),
    }

}


fn cmd_clean(name: Option<&str>) -> anyhow::Result<()> {
    let project_name = name.unwrap_or("./");
    if project_name == "./" {
        println!(
            "{}[calmake]{} WARNING: cleaning current directory!{}{}",
            color::CYAN,
            color::RESET,
            color::BRIGHT_YELLOW,
            color::RESET
        );
    }
    let root = Path::new(project_name);
    if !root.exists() {
        anyhow::bail!("directory `{project_name}` does not exist");
    }

    println!(
        "{}[calmake]{} cleaning project `{}`",
        color::CYAN,
        color::RESET,
        project_name
    );
    fs::remove_dir_all(root.join("bin"))?;
    fs::remove_dir_all(root.join(".calmake"))?;

    println!(
        "{}[calmake]{} cleaned project `{}`",
        color::CYAN,
        color::RESET,
        project_name
    );

    Ok(())
}

fn cmd_init(name: Option<&str>) -> anyhow::Result<()> {
    let project_name = name.unwrap_or("calmake-hello");
    let root = Path::new(project_name);
    if root.exists() {
        anyhow::bail!("directory `{project_name}` already exists");
    }

    println!(
        "{}[calmake]{} creating project `{}`",
        color::CYAN,
        color::RESET,
        project_name
    );
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("libcallum"))?;
    fs::create_dir_all(root.join("include"))?;

    let build_cal = if is_windows() {
        r#"target libcallum {
    kind = sharedlib
    language = cpp
    sources = ["libcallum"]
    deps = []
    output = "bin/libcallum.dll"
    cppflags = []
    ldflags = []
}

target hello {
    kind = exe
    language = cpp
    sources = ["src"]
    deps = ["libcallum"]
    output = "bin/hello.exe"
    cppflags = ["-Iinclude", "-Ilibcallum"]
    ldflags = []
}
"#
    } else {
        r#"target libcallum {
    kind = sharedlib
    language = cpp
    sources = ["libcallum"]
    deps = []
    output = "bin/libcallum.so"
    cppflags = []
    ldflags = []
}

target hello {
    kind = exe
    language = cpp
    sources = ["src"]
    deps = ["libcallum"]
    output = "bin/hello"
    cppflags = ["-Iinclude", "-Ilibcallum"]
    ldflags = []
}
"#
    };

    let main_cpp = r#"#include "libcallum.hpp"
#include <iostream>

int main() {
    libcallum_hello();
    return 0;
}
"#;

    let lib_hpp = r#"#pragma once

#ifdef _WIN32
#define CALLUM_API __declspec(dllexport)
#else
#define CALLUM_API
#endif

CALLUM_API void libcallum_hello();
"#;

    let lib_cpp = r#"#include "libcallum.hpp"
#include <iostream>

void libcallum_hello() {
    std::cout << "Hello from libcallum!\n";
}
"#;

    fs::write(root.join("build.cal"), build_cal)?;
    fs::write(root.join("src").join("main.cpp"), main_cpp)?;
    fs::write(root.join("libcallum").join("libcallum.hpp"), lib_hpp)?;
    fs::write(root.join("libcallum").join("libcallum.cpp"), lib_cpp)?;

    println!(
        "{}[calmake]{} created:",
        color::CYAN,
        color::RESET
    );
    println!("  {project_name}/build.cal");
    println!("  {project_name}/src/main.cpp");
    println!("  {project_name}/libcallum/libcallum.hpp");
    println!("  {project_name}/libcallum/libcallum.cpp");
    println!();
    println!("Next steps:");
    println!("  cd {project_name}");
    println!("  calmake");

    Ok(())
}

fn cmd_build() -> anyhow::Result<()> {
    let config_path = Path::new("build.cal");
    if !config_path.exists() {
        anyhow::bail!("build.cal not found in current directory (run `calmake init <name>` to create one)");
    }

    fs::create_dir_all(".calmake/cache/obj")?;
    fs::create_dir_all(".calmake/cache/deps")?;
    fs::create_dir_all(".calmake/cache/lib")?;
    fs::create_dir_all(".calmake/state")?;
    fs::create_dir_all("bin")?;

    let config_str = fs::read_to_string(config_path)?;
    let config = parse_config(&config_str)?;

    let compiler = detect_compiler()?;
    println!(
        "{}[calmake]{} using compiler: {}{:?}{}",
        color::CYAN,
        color::RESET,
        color::BRIGHT_GREEN,
        compiler,
        color::RESET
    );

    let graph = BuildGraph::from_config(&config)?;
    let roots = graph.root_targets();
    if roots.is_empty() {
        anyhow::bail!("no root targets found!");
    }
    println!(
        "{}[calmake]{} roots: {:?}",
        color::CYAN,
        color::RESET,
        roots
    );

    let order = graph.topo_sort()?;
    vprintln!("topo order: {:?}", order);

    let cache = BuildCache::load(".calmake/state/buildcache.json")?;
    let cache_arc = Arc::new(Mutex::new(cache));
    let graph_arc = Arc::new(graph);
    let compiler_arc = Arc::new(compiler);
    let done = Arc::new(Mutex::new(HashSet::<String>::new()));

    let num_threads = num_cpus::get().max(1);
    println!(
        "{}[calmake]{} using {}{}{} worker threads",
        color::CYAN,
        color::RESET,
        color::BRIGHT_CYAN,
        num_threads,
        color::RESET
    );
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    loop {
        {
            let done_guard = done.lock().unwrap();
            if done_guard.len() == order.len() {
                break;
            }
        }

        let mut to_run = Vec::new();
        {
            let done_guard = done.lock().unwrap();
            for name in &order {
                if done_guard.contains(name) {
                    continue;
                }
                let node = graph_arc.targets.get(name).unwrap();
                if node.deps.iter().all(|d| done_guard.contains(d)) {
                    to_run.push(name.clone());
                }
            }
        }

        if to_run.is_empty() {
            anyhow::bail!("BUG!!!! deadlock or cycle detected even after cycle detector!!!!! (no runnable targets but not all done!!!)");
        }

        rayon::scope(|s| {
            for name in to_run {
                let graph = Arc::clone(&graph_arc);
                let cache = Arc::clone(&cache_arc);
                let done = Arc::clone(&done);
                let compiler = Arc::clone(&compiler_arc);

                s.spawn(move |_| {
                    if let Err(e) = build_target(&name, &graph, cache, &compiler) {
                        eprintln!(
                            "{}[calmake] error:{} target `{}` failed: {e}",
                            color::BRIGHT_RED,
                            color::RESET,
                            name
                        );
                        std::process::exit(1);
                    }
                    let mut done_guard = done.lock().unwrap();
                    done_guard.insert(name);
                });
            }
        });
    }

    let cache = Arc::try_unwrap(cache_arc).unwrap().into_inner().unwrap();
    cache.save(".calmake/state/buildcache.json")?;

    cleanup_bin(&graph_arc)?;

    println!(
        "{}[calmake]{} {}build complete{}",
        color::CYAN,
        color::RESET,
        color::BRIGHT_GREEN,
        color::RESET
    );
    Ok(())
}

// ---------- DSL ----------

#[derive(Debug, Clone)]
struct TargetConfig {
    kind: TargetKind,
    language: Language,
    sources: Vec<String>,
    deps: Vec<String>,
    output: String,
    cflags: Vec<String>,
    cppflags: Vec<String>,
    ldflags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetKind {
    Exe,
    Staticlib,
    Sharedlib,
}

#[derive(Debug, Clone, Copy)]
enum Language {
    C,
    Cpp,
}

#[derive(Debug)]
struct BuildConfig {
    targets: HashMap<String, TargetConfig>,
}

fn parse_config(src: &str) -> anyhow::Result<BuildConfig> {
    let mut targets = HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current: Option<TargetConfig> = None;

    for (lineno, raw_line) in src.lines().enumerate() {
        let line_no = lineno + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("target ") && line.ends_with('{') {
            if current.is_some() {
                anyhow::bail!("line {line_no}: nested target not allowed");
            }
            let inner = &line["target ".len()..line.len() - 1].trim();
            if inner.is_empty() {
                anyhow::bail!("line {line_no}: target name missing");
            }
            current_name = Some(inner.to_string());
            current = Some(TargetConfig {
                kind: TargetKind::Exe,
                language: Language::Cpp,
                sources: Vec::new(),
                deps: Vec::new(),
                output: String::new(),
                cflags: Vec::new(),
                cppflags: Vec::new(),
                ldflags: Vec::new(),
            });
            continue;
        }

        if line == "}" {
            if let (Some(name), Some(cfg)) = (current_name.take(), current.take()) {
                vprintln!("parsed target {name}: {:?}", cfg);
                if cfg.output.is_empty() {
                    anyhow::bail!("target `{name}` missing `output`");
                }
                if cfg.sources.is_empty() {
                    anyhow::bail!("target `{name}` has no `sources`");
                }
                targets.insert(name, cfg);
            } else {
                anyhow::bail!("line {line_no}: stray `}}`");
            }
            continue;
        }

        let (name, cfg) = match (&current_name, &mut current) {
            (Some(n), Some(c)) => (n.clone(), c),
            _ => anyhow::bail!("line {line_no}: key outside of target block"),
        };

        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            anyhow::bail!("line {line_no}: expected `key = value`");
        }
        let key = parts[0].trim();
        let value = parts[1].trim();

        match key {
            "kind" => {
                let val = trim_string(value);
                cfg.kind = match val.as_str() {
                    "exe" => TargetKind::Exe,
                    "staticlib" => TargetKind::Staticlib,
                    "sharedlib" => TargetKind::Sharedlib,
                    other => anyhow::bail!("line {line_no}: invalid kind `{other}`"),
                };
            }
            "language" => {
                let val = trim_string(value);
                cfg.language = match val.as_str() {
                    "c" => Language::C,
                    "cpp" => Language::Cpp,
                    other => anyhow::bail!("line {line_no}: invalid language `{other}`"),
                };
            }
            "sources" => {
                cfg.sources = parse_list_or_word(value);
            }
            "deps" => {
                cfg.deps = parse_list_or_word(value);
            }
            "output" => {
                cfg.output = trim_string(value);
            }
            "cflags" => {
                cfg.cflags = parse_list_or_word(value);
            }
            "cppflags" => {
                cfg.cppflags = parse_list_or_word(value);
            }
            "ldflags" => {
                cfg.ldflags = parse_list_or_word(value);
            }
            other => {
                anyhow::bail!("line {line_no}: unknown key `{other}` in target `{name}`");
            }
        }
    }

    if current.is_some() || current_name.is_some() {
        anyhow::bail!("unterminated target block");
    }
    if targets.is_empty() {
        anyhow::bail!("no targets defined in build.cal");
    }

    Ok(BuildConfig { targets })
}

fn trim_string(v: &str) -> String {
    let v = v.trim();
    if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}

fn parse_list_or_word(v: &str) -> Vec<String> {
    let v = v.trim();
    if v.is_empty() {
        return Vec::new();
    }
    if v.starts_with('[') && v.ends_with(']') {
        parse_list(v)
    } else {
        v.split_whitespace().map(|s| trim_string(s)).collect()
    }
}

fn parse_list(v: &str) -> Vec<String> {
    let inner = &v[1..v.len() - 1].trim();
    if inner.is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(|s| trim_string(s.trim()))
        .collect()
}

// ---------- Graph ----------

#[derive(Debug)]
struct TargetNode {
    name: String,
    kind: TargetKind,
    language: Language,
    sources: Vec<PathBuf>,
    deps: Vec<String>,
    output: PathBuf,
    cflags: Vec<String>,
    cppflags: Vec<String>,
    ldflags: Vec<String>,
}

#[derive(Debug)]
struct BuildGraph {
    targets: HashMap<String, TargetNode>,
    reverse_deps: HashMap<String, Vec<String>>,
}

impl BuildGraph {
    fn from_config(cfg: &BuildConfig) -> anyhow::Result<Self> {
        let mut targets = HashMap::new();
        for (name, t) in &cfg.targets {
            let mut sources = Vec::new();
            for s in &t.sources {
                let p = PathBuf::from(s);
                if p.is_dir() {
                    vprintln!("auto-discovering sources under {:?}", p);
                    collect_sources(&p, &mut sources)?;
                } else {
                    sources.push(p);
                }
            }
            if sources.is_empty() {
                anyhow::bail!("target `{name}` has no sources after discovery");
            }

            let output = PathBuf::from(&t.output);
            targets.insert(
                name.clone(),
                TargetNode {
                    name: name.clone(),
                    kind: t.kind,
                    language: t.language,
                    sources,
                    deps: t.deps.clone(),
                    output,
                    cflags: t.cflags.clone(),
                    cppflags: t.cppflags.clone(),
                    ldflags: t.ldflags.clone(),
                },
            );
        }

        for (name, node) in &targets {
            for d in &node.deps {
                if !targets.contains_key(d) {
                    anyhow::bail!("target `{name}` depends on nonexistent target `{d}`");
                }
            }
        }

        let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();
        for (name, node) in &targets {
            for d in &node.deps {
                reverse_deps.entry(d.clone()).or_default().push(name.clone());
            }
        }

        Ok(Self { targets, reverse_deps })
    }

    fn root_targets(&self) -> Vec<String> {
        self.targets
            .values()
            .filter(|n| n.deps.is_empty())
            .map(|n| n.name.clone())
            .collect()
    }

    fn topo_sort(&self) -> anyhow::Result<Vec<String>> {
        let mut indegree: HashMap<String, usize> =
            self.targets.keys().map(|k| (k.clone(), 0)).collect();

        for (name, node) in &self.targets {
            indegree.insert(name.clone(), node.deps.len());
        }

        let mut q = VecDeque::new();
        for (name, &deg) in &indegree {
            if deg == 0 {
                q.push_back(name.clone());
            }
        }

        let mut order = Vec::new();
        while let Some(n) = q.pop_front() {
            order.push(n.clone());
            if let Some(children) = self.reverse_deps.get(&n) {
                for c in children {
                    let e = indegree.get_mut(c).unwrap();
                    *e -= 1;
                    if *e == 0 {
                        q.push_back(c.clone());
                    }
                }
            }
        }

        if order.len() != self.targets.len() {
            anyhow::bail!("Cyclic dependency detected!");
        }

        Ok(order)
    }
}

fn collect_sources(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_sources(&path, out)?;
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_ascii_lowercase();
            if ext == "c" || ext == "cc" || ext == "cxx" || ext == "cpp" {
                vprintln!("discovered source {:?}", path);
                out.push(path);
            }
        }
    }
    Ok(())
}

// ---------- Compiler detection ----------
///<summary>
/// The CompilerKind enum represents the different types of compilers that can be detected.
/// </summary>
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum CompilerKind {
    ClangCpp,
    ClangC,
    Gpp,
    Cl,
}
///<summary>
/// The Compiler struct represents a detected compiler, including its kind and the executable name.
/// </summary>
#[derive(Debug, Clone)]
struct Compiler {
    kind: CompilerKind,
    exe: String,
}

///<summary>
/// Checks if the current operating system is Windows.
/// </summary>
fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

fn detect_compiler() -> anyhow::Result<Compiler> {
    let candidates: Vec<(&str, CompilerKind)> = if is_windows() {
        vec![
            ("clang++", CompilerKind::ClangCpp),
            ("clang", CompilerKind::ClangC),
            ("g++", CompilerKind::Gpp),
            ("cl", CompilerKind::Cl),
        ]
    } else {
        vec![
            ("clang++", CompilerKind::ClangCpp),
            ("clang", CompilerKind::ClangC),
            ("g++", CompilerKind::Gpp),
        ]
    };

    for (name, kind) in candidates {
        if which(name).is_some() {
            return Ok(Compiler {
                kind,
                exe: name.to_string(),
            });
        }
    }

    anyhow::bail!(
        "no supported compiler found (tried clang++, clang, g++, cl). \
         Install LLVM/Clang, MSVC Build Tools, or MinGW and ensure the compiler is in your PATH. \
         Verbose mode (-v) can be used to see which compilers were checked. \
         Also, ensure that the compiler is callable from the command line (e.g., `clang++ --version` should work).
         " 
    );
}

fn which(exe: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let candidates = if is_windows() {
        vec![format!("{exe}.exe"), exe.to_string()]
    } else {
        vec![exe.to_string()]
    };

    for dir in env::split_paths(&path_var) {
        for candidate_name in &candidates {
            let candidate = dir.join(candidate_name);
            if candidate.is_file() {
                vprintln!("Found {} as a candidate!", candidate.display());
                return Some(candidate);
            }
        }
    }
    None
}

// ---------- Cache & build ----------

#[derive(Debug, Serialize, Deserialize, Default)]
struct BuildCache {
    targets: HashMap<String, TargetCache>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
/// <summary>
/// The TargetCache is a struct
/// that represents the data for the caching information for a Target.
/// link_hash is the hash for the linked object
/// And target_hash is the hash for the Target
/// sources is a HashMap for all source files for the Target.
/// </summary>
struct TargetCache {
    target_hash: String,
    link_hash: String,
    sources: HashMap<String, String>,
}

impl BuildCache {
    fn load(path: &str) -> anyhow::Result<Self> {
        let p = Path::new(path);
        if !p.exists() {
            return Ok(Self::default());
        }
        let mut s = String::new();
        File::open(p)?.read_to_string(&mut s)?;
        let cache: BuildCache = serde_json::from_str(&s)?;
        Ok(cache)
    }

    fn save(&self, path: &str) -> anyhow::Result<()> {
        vprintln!("Saving buildcache to '{}'!",
            path
        );
        let s = serde_json::to_string_pretty(self)?;
        let mut f = File::create(path)?;
        f.write_all(s.as_bytes())?;
        Ok(())
    }
}

fn build_target(
    name: &str,
    graph: &BuildGraph,
    cache_arc: Arc<Mutex<BuildCache>>,
    compiler: &Compiler,
) -> anyhow::Result<()> {
    let node = graph.targets.get(name).unwrap();

    let mut meta_hasher = blake3::Hasher::new();
    meta_hasher.update(format!("{:?}", node.kind).as_bytes());
    meta_hasher.update(format!("{:?}", node.language).as_bytes());

    for dep in &node.deps {
        let dep_node = graph.targets.get(dep).unwrap();
        if dep_node.output.exists() {
            let h = hash_file(&dep_node.output)?;
            meta_hasher.update(h.as_bytes());
        } else {
            meta_hasher.update(b"missing-dep-output");
        }
    }

    for f in &node.cflags {
        meta_hasher.update(f.as_bytes());
    }
    for f in &node.cppflags {
        meta_hasher.update(f.as_bytes());
    }
    for f in &node.ldflags {
        meta_hasher.update(f.as_bytes());
    }

    let meta_hash = meta_hasher.finalize().to_hex().to_string();

    let (old_target_cache, mut cache_guard) = {
        let cache = cache_arc.lock().unwrap();
        let old = cache.targets.get(name).cloned().unwrap_or_default();
        (old, cache)
    };

    let mut new_target_cache = TargetCache {
        target_hash: meta_hash.clone(),
        link_hash: String::new(),
        sources: HashMap::new(),
    };

    let mut to_compile = Vec::new();
    let mut any_source_changed = false;

    for src in &node.sources {
        let src_str = src.to_string_lossy().to_string();
        let obj = obj_path_for(src);
        let dep = depfile_path_for(src);

        let mut src_hasher = blake3::Hasher::new();

        let src_file_hash = hash_file(src)?;
        src_hasher.update(src_file_hash.as_bytes());

        let dep_path = depfile_path_for(src);
        if dep_path.exists() {
            let headers = parse_depfile(&dep_path)?;
            for hdr in headers {
                if hdr.exists() {
                    let hh = hash_file(&hdr)?;
                    src_hasher.update(hh.as_bytes());
                }
            }
        }

        src_hasher.update(format!("{:?}", node.language).as_bytes());
        src_hasher.update(format!("{:?}", compiler.kind).as_bytes());
        for f in &node.cflags {
            src_hasher.update(f.as_bytes());
        }
        for f in &node.cppflags {
            src_hasher.update(f.as_bytes());
        }

        let src_hash = src_hasher.finalize().to_hex().to_string();
        new_target_cache
            .sources
            .insert(src_str.clone(), src_hash.clone());

        let old_hash_opt = old_target_cache.sources.get(&src_str);

        let obj_missing = !obj.exists();
        let dep_missing = !dep.exists();
        let hash_changed = old_hash_opt.map(|h| h != &src_hash).unwrap_or(true);

        if obj_missing || dep_missing || hash_changed {
            any_source_changed = true;
            to_compile.push((src.clone(), obj.clone(), dep.clone()));
        }
    }

    let output_exists = node.output.exists();

    if !any_source_changed
        && old_target_cache.target_hash == meta_hash
        && output_exists
    {
        println!(
            "{}[calmake]{} {}: {}is up to date!{} ",
            color::CYAN,
            color::RESET,
            name,
            color::BRIGHT_GREEN,
            color::RESET
        );
        cache_guard.targets.insert(name.to_string(), new_target_cache);
        return Ok(());
    }
    println!("{}[calmake]{} {} needs to be rebuilt!",
        color::CYAN,
        color::RESET,
        name

    );

    if let Some(parent) = node.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    rayon::scope(|s| {
        for (src, obj, dep) in to_compile {
            let compiler = compiler.clone();
            let node = node_clone_shallow(node);
            s.spawn(move |_| {
                if let Err(e) = compile_one_source(&compiler, &node, &src, &obj, &dep) {
                    eprintln!(
                        "{}[calmake] error:{} compile failed for {:?}: {e}",
                        color::BRIGHT_RED,
                        color::RESET,
                        src
                    );
                    std::process::exit(1);
                }
            });
        }
    });

    let mut objects = Vec::new();
    for src in &node.sources {
        let obj = obj_path_for(src);
        objects.push(obj);
    }

    let mut link_hasher = blake3::Hasher::new();
    link_hasher.update(format!("{:?}", compiler.kind).as_bytes());
    link_hasher.update(format!("{:?}", node.kind).as_bytes());
    link_hasher.update(node.output.to_string_lossy().as_bytes());
    for obj in &objects {
        if obj.exists() {
            let h = hash_file(obj)?;
            link_hasher.update(h.as_bytes());
        } else {
            link_hasher.update(b"missing-obj");
        }
    }
    for dep in &node.deps {
        let dep_node = graph.targets.get(dep).unwrap();
        if dep_node.output.exists() {
            let h = hash_file(&dep_node.output)?;
            link_hasher.update(h.as_bytes());
        } else {
            link_hasher.update(b"missing-dep-output");
        }
    }
    for f in &node.ldflags {
        link_hasher.update(f.as_bytes());
    }
    let link_hash = link_hasher.finalize().to_hex().to_string();
    new_target_cache.link_hash = link_hash.clone();

    if old_target_cache.link_hash == link_hash && output_exists {
        println!(
            "{}[calmake]{} {}: {}link up to date{} (skipping link step)",
            color::CYAN,
            color::RESET,
            name,
            color::BRIGHT_GREEN,
            color::RESET
        );
        cache_guard.targets.insert(name.to_string(), new_target_cache);
        return Ok(());
    }

    link_target(compiler, node, &objects, &graph.targets)?;

    cache_guard.targets.insert(name.to_string(), new_target_cache);

    Ok(())
}

fn node_clone_shallow(node: &TargetNode) -> TargetNode {
    TargetNode {
        name: node.name.clone(),
        kind: node.kind,
        language: node.language,
        sources: node.sources.clone(),
        deps: node.deps.clone(),
        output: node.output.clone(),
        cflags: node.cflags.clone(),
        cppflags: node.cppflags.clone(),
        ldflags: node.ldflags.clone(),
    }
}

fn obj_path_for(src: &Path) -> PathBuf {
    let mut name = src.to_string_lossy().replace(['\\', '/'], "_");
    if name.is_empty() {
        name = "unknown".into();
    }
    PathBuf::from(".calmake/cache/obj").join(format!("{name}.o"))
}

fn depfile_path_for(src: &Path) -> PathBuf {
    let mut name = src.to_string_lossy().replace(['\\', '/'], "_");
    if name.is_empty() {
        name = "unknown".into();
    }
    PathBuf::from(".calmake/cache/deps").join(format!("{name}.d"))
}

fn cached_import_lib_for(node: &TargetNode) -> PathBuf {
    let mut base = node.name.clone();
    if !base.ends_with(".lib") {
        base.push_str(".lib");
    }
    PathBuf::from(".calmake/cache/lib").join(base)
}

fn compile_one_source(
    compiler: &Compiler,
    node: &TargetNode,
    src: &Path,
    obj: &Path,
    depfile: &Path,
) -> anyhow::Result<()> {
    match compiler.kind {
        CompilerKind::ClangCpp | CompilerKind::Gpp | CompilerKind::ClangC => {
            let mut cmd = Command::new(&compiler.exe);

            if matches!(compiler.kind, CompilerKind::ClangC)
                && matches!(node.language, Language::Cpp)
            {
                cmd.arg("-x").arg("c++");
            }

            match node.language {
                Language::C => {
                    for flag in &node.cflags {
                        cmd.arg(flag);
                    }
                }
                Language::Cpp => {
                    for flag in &node.cppflags {
                        cmd.arg(flag);
                    }
                }
            }

            cmd.arg("-MMD");
            cmd.arg("-MF").arg(depfile);

            cmd.arg("-c").arg(src).arg("-o").arg(obj);

            println!(
                "{}[calmake]{} {}compile:{} {:?}",
                color::CYAN,
                color::RESET,
                color::BRIGHT_BLUE,
                color::RESET,
                cmd
            );
            let status = cmd.status()?;
            if !status.success() {
                anyhow::bail!("compiler failed with status {status}");
            }
        }
        CompilerKind::Cl => {
            let mut cmd = Command::new("cl");
            cmd.arg("/nologo");

            match node.language {
                Language::C => {
                    for flag in &node.cflags {
                        cmd.arg(flag);
                    }
                }
                Language::Cpp => {
                    cmd.arg("/EHsc");
                    for flag in &node.cppflags {
                        cmd.arg(flag);
                    }
                }
            }

            let fo = format!("/Fo:{}", obj.display());
            cmd.arg("/c").arg(src).arg(fo);

            println!(
                "{}[calmake]{} {}compile:{} {:?}",
                color::CYAN,
                color::RESET,
                color::BRIGHT_BLUE,
                color::RESET,
                cmd
            );
            let status = cmd.status()?;
            if !status.success() {
                anyhow::bail!("cl failed with status {status}");
            }
        }
    }

    Ok(())
}

fn link_target(
    compiler: &Compiler,
    node: &TargetNode,
    objects: &[PathBuf],
    all_targets: &HashMap<String, TargetNode>,
) -> anyhow::Result<()> {
    match compiler.kind {
        CompilerKind::ClangCpp | CompilerKind::Gpp | CompilerKind::ClangC => {
            match node.kind {
                TargetKind::Staticlib => {
                    let mut cmd = Command::new("llvm-ar");
                    cmd.arg("rcs");
                    cmd.arg(&node.output);
                    for obj in objects {
                        cmd.arg(obj);
                    }

                    println!(
                        "{}[calmake]{} {}archive:{} {:?}",
                        color::CYAN,
                        color::RESET,
                        color::BRIGHT_MAGENTA,
                        color::RESET,
                        cmd
                    );
                    let status = cmd.status()?;
                    if !status.success() {
                        anyhow::bail!("llvm-ar failed with status {status}");
                    }
                }
                _ => {
                    let mut cmd = Command::new(&compiler.exe);

                    for obj in objects {
                        cmd.arg(obj);
                    }

                    for dep in &node.deps {
                        let dep_node = all_targets.get(dep).unwrap();
                        if dep_node.kind == TargetKind::Sharedlib && is_windows() {
                            let cached_lib = cached_import_lib_for(dep_node);
                            cmd.arg(&cached_lib);
                        } else {
                            cmd.arg(&dep_node.output);
                        }
                    }

                    for flag in &node.ldflags {
                        cmd.arg(flag);
                    }

                    match node.kind {
                        TargetKind::Exe => {
                            cmd.arg("-o").arg(&node.output);
                        }
                        TargetKind::Sharedlib => {
                            let dll_path = node.output.to_string_lossy().to_string();
                            cmd.arg("-shared").arg("-o").arg(&dll_path);
                        }
                        TargetKind::Staticlib => unreachable!(),
                    }

                    println!(
                        "{}[calmake]{} {}link:{} {:?}",
                        color::CYAN,
                        color::RESET,
                        color::BRIGHT_MAGENTA,
                        color::RESET,
                        cmd
                    );
                    let status = cmd.status()?;
                    if !status.success() {
                        anyhow::bail!("link failed with status {status}");
                    }

                    if node.kind == TargetKind::Sharedlib && is_windows() {
                        let dll_path = node.output.to_string_lossy().to_string();
                        let default_lib = dll_path.replace(".dll", ".lib");
                        let default_lib_path = PathBuf::from(&default_lib);
                        let cached_lib = cached_import_lib_for(node);

                        if default_lib_path.exists() {
                            fs::create_dir_all(cached_lib.parent().unwrap())?;
                            fs::copy(&default_lib_path, &cached_lib)?;
                            vprintln!(
                                "cached import lib {:?} -> {:?}",
                                default_lib_path,
                                cached_lib
                            );
                        }
                    }
                }
            }
        }
        CompilerKind::Cl => {
            match node.kind {
                TargetKind::Staticlib => {
                    let mut cmd = Command::new("lib");
                    cmd.arg("/nologo");
                    let out = format!("/OUT:{}", node.output.display());
                    cmd.arg(out);
                    for obj in objects {
                        cmd.arg(obj);
                    }

                    println!(
                        "{}[calmake]{} {}archive:{} {:?}",
                        color::CYAN,
                        color::RESET,
                        color::BRIGHT_MAGENTA,
                        color::RESET,
                        cmd
                    );
                    let status = cmd.status()?;
                    if !status.success() {
                        anyhow::bail!("lib.exe failed with status {status}");
                    }
                }
                _ => {
                    let mut cmd = Command::new("cl");
                    cmd.arg("/nologo");

                    for obj in objects {
                        cmd.arg(obj);
                    }

                    for dep in &node.deps {
                        let dep_node = all_targets.get(dep).unwrap();
                        if dep_node.kind == TargetKind::Sharedlib {
                            let cached_lib = cached_import_lib_for(dep_node);
                            cmd.arg(&cached_lib);
                        } else {
                            cmd.arg(&dep_node.output);
                        }
                    }

                    for flag in &node.ldflags {
                        cmd.arg(flag);
                    }

                    match node.kind {
                        TargetKind::Exe => {
                            let fe = format!("/Fe:{}", node.output.display());
                            cmd.arg(fe);
                        }
                        TargetKind::Sharedlib => {
                            let fe = format!("/Fe:{}", node.output.display());
                            cmd.arg("/LD").arg(fe);
                        }
                        TargetKind::Staticlib => unreachable!(),
                    }

                    println!(
                        "{}[calmake]{} {}link:{} {:?}",
                        color::CYAN,
                        color::RESET,
                        color::BRIGHT_MAGENTA,
                        color::RESET,
                        cmd
                    );
                    let status = cmd.status()?;
                    if !status.success() {
                        anyhow::bail!("cl link failed with status {status}");
                    }

                    if node.kind == TargetKind::Sharedlib {
                        let dll_path = node.output.to_string_lossy().to_string();
                        let default_lib = dll_path.replace(".dll", ".lib");
                        let default_lib_path = PathBuf::from(&default_lib);
                        let cached_lib = cached_import_lib_for(node);

                        if default_lib_path.exists() {
                            fs::create_dir_all(cached_lib.parent().unwrap())?;
                            fs::copy(&default_lib_path, &cached_lib)?;
                            vprintln!(
                                "cached import lib {:?} -> {:?}",
                                default_lib_path,
                                cached_lib
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// ---------- depfile & hashing ----------

fn parse_depfile(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let f = File::open(path)?;
    let reader = BufReader::new(f);

    let mut line = String::new();
    for l in reader.lines() {
        let l = l?;
        line.push_str(&l);
        if line.ends_with('\\') {
            line.pop();
            continue;
        } else {
            break;
        }
    }

    if line.is_empty() {
        return Ok(Vec::new());
    }

    let mut parts = line.split_whitespace();
    let _first = parts.next();

    let mut headers = Vec::new();
    for p in parts {
        let p = p.trim_end_matches('\\');
        if p == ":" {
            continue;
        }
        if p.ends_with(':') {
            continue;
        }
        if p.is_empty() {
            continue;
        }
        headers.push(PathBuf::from(p));
    }

    vprintln!("depfile {:?} -> {} headers", path, headers.len());
    Ok(headers)
}

fn hash_file(path: &Path) -> anyhow::Result<blake3::Hash> {
    let mut f = File::open(path)
        .map_err(|e| anyhow::anyhow!("failed to open {:?}: {e}", path))?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize())
}

// ---------- cleanup bin/ ----------

fn cleanup_bin(graph: &BuildGraph) -> anyhow::Result<()> {
    let bin_dir = Path::new("bin");
    if !bin_dir.exists() {
        return Ok(());
    }

    let mut expected: HashSet<PathBuf> = HashSet::new();

    for node in graph.targets.values() {
        expected.insert(normalize(bin_dir, &node.output));

        if node.kind == TargetKind::Sharedlib && is_windows() {
            let dll_path = node.output.to_string_lossy().to_string();
            let default_lib = dll_path.replace(".dll", ".lib");
            expected.insert(normalize(bin_dir, &PathBuf::from(default_lib)));
        }
    }

    for entry in fs::read_dir(bin_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if ext != "dll" && ext != "exe" && ext != "lib" && ext != "so" {
            continue;
        }

        let norm = normalize(bin_dir, &path);
        if !expected.contains(&norm) {
            println!(
                "{}[calmake]{} {}removing unused artifact:{} {}",
                color::CYAN,
                color::RESET,
                color::BRIGHT_YELLOW,
                color::RESET,
                path.display()
            );
            let _ = fs::remove_file(&path);
        }
    }

    Ok(())
}

fn normalize(root: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p.file_name().unwrap_or_default())
    }
}