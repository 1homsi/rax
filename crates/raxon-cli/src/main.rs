//! rax CLI — project scaffolding and development tool.
//!
//! Usage:
//!   rax new <project-name>                Create a new raxon app project
//!   rax doctor                            Print environment diagnostic info
//!   rax build [--target <ios-sim|ios|android|web|macos>]
//!                                         Print the cargo build command to run
//!   rax run [--target <ios-sim|ios>]      Print the cargo build + Xcode run steps
//!   rax test [-- <args>]                  Run cargo test, forwarding extra args
//!   rax lint                              Run cargo clippy --all-targets
//!   rax fmt [--check]                     Run cargo fmt (or check formatting)
//!   rax add <crate-name>                  Print the cargo add command for a crate
//!   rax generate [--target android|web|all] [--glue-only]
//!                                         Generate platform host bindings/shells
//!   rax --version                         Print the rax version
//!   rax --help                            Print help

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ANDROID_GRADLE_PLUGIN_VERSION: &str = "9.2.0";
const GRADLE_WRAPPER_VERSION: &str = "9.4.1";
const ANDROID_COMPILE_SDK: u32 = 36;
const ANDROID_MIN_SDK: u32 = 23;
const ANDROID_TARGET_SDK: u32 = 36;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("new") => {
            let name = match args.get(2) {
                Some(n) => n.clone(),
                None => {
                    eprintln!("Usage: rax new <project-name>");
                    process::exit(1);
                }
            };
            create_project(&name);
        }
        Some("doctor") => {
            run_doctor();
        }
        Some("build") => {
            let target = parse_target_flag(&args, "ios-sim");
            run_build(&target);
        }
        Some("run") => {
            let target = parse_target_flag(&args, "ios-sim");
            run_run(&target);
        }
        Some("test") => {
            // Collect everything after an optional "--" separator, or any
            // trailing args that don't look like rax flags.
            let extra: Vec<String> = {
                let mut after_sep = false;
                let mut out = Vec::new();
                for arg in args.iter().skip(2) {
                    if arg == "--" {
                        after_sep = true;
                        continue;
                    }
                    if after_sep || !arg.starts_with('-') {
                        out.push(arg.clone());
                    }
                }
                out
            };
            cmd_test(&extra);
        }
        Some("lint") => {
            cmd_lint();
        }
        Some("fmt") => {
            let check = args.iter().skip(2).any(|a| a == "--check");
            cmd_fmt(check);
        }
        Some("add") => {
            let crate_name = match args.get(2) {
                Some(n) => n.clone(),
                None => {
                    eprintln!("Usage: rax add <crate-name>");
                    process::exit(1);
                }
            };
            cmd_add(&crate_name);
        }
        Some("generate") => {
            if args
                .iter()
                .skip(2)
                .any(|arg| arg == "--help" || arg == "-h")
            {
                println!("{}", generate_usage());
                return;
            }
            let options = parse_generate_options(&args).unwrap_or_else(|error| {
                eprintln!("{error}");
                eprintln!(
                    "Usage: rax generate [--target android|web|all] [--out generated] [--app-fn app] [--glue-only]"
                );
                process::exit(1);
            });
            run_generate(&options);
        }
        Some("--version") | Some("-V") => {
            println!("rax {}", VERSION);
        }
        Some("--help") | Some("-h") | None => {
            print_help();
        }
        Some(cmd) => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("Run 'rax --help' for usage.");
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("rax {} — Rust-native mobile framework", VERSION);
    println!();
    println!("USAGE:");
    println!("    rax <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    new <name>                Create a new raxon app project");
    println!("    doctor                    Print environment diagnostic info");
    println!("    build [--target <TARGET>] Print the build command for a target");
    println!("    run   [--target <TARGET>] Print the run steps for a target");
    println!("    test  [-- <args>]         Run cargo test, forwarding extra args");
    println!("    lint                      Run cargo clippy --all-targets");
    println!("    fmt   [--check]           Run cargo fmt (or --check to only verify)");
    println!("    add   <crate-name>        Print the cargo add command for a crate");
    println!("    generate                  Generate Android/Web host bindings and shells");
    println!("    --version                 Print the rax version");
    println!("    --help                    Print this help message");
    println!();
    println!("TARGETS:");
    println!("    ios-sim   (default)  aarch64-apple-ios-sim");
    println!("    ios                  aarch64-apple-ios");
    println!("    android              aarch64-linux-android");
    println!("    web                  wasm32-unknown-unknown");
    println!("    macos                aarch64-apple-darwin");
    println!();
    println!("EXAMPLE:");
    println!("    rax new my-app");
    println!("    cd my-app");
    println!("    rax build --target ios-sim");
    println!("    rax generate --target all --out generated");
}

// ---------------------------------------------------------------------------
// doctor
// ---------------------------------------------------------------------------

fn run_doctor() {
    println!("rax doctor");
    println!();

    // rustc
    match Command::new("rustc").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("  ✓ rustc found: {}", ver);
        }
        _ => println!("  ✗ rustc not found — install Rust from https://rustup.rs"),
    }

    // cargo
    match Command::new("cargo").arg("--version").output() {
        Ok(out) if out.status.success() => {
            println!("  ✓ cargo found");
        }
        _ => println!("  ✗ cargo not found"),
    }

    // rustup installed targets
    let installed_targets: Vec<String> = match Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .collect(),
        _ => Vec::new(),
    };

    let check_target = |triple: &str| {
        if installed_targets.iter().any(|t| t == triple) {
            println!("  ✓ {} target installed", triple);
        } else {
            println!(
                "  ✗ {} target NOT installed — run: rustup target add {}",
                triple, triple
            );
        }
    };

    check_target("aarch64-apple-ios-sim");
    check_target("aarch64-apple-ios");
    check_target("aarch64-linux-android");
    check_target("wasm32-unknown-unknown");

    // Xcode Command Line Tools
    match Command::new("xcode-select").arg("--print-path").output() {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("  info: Xcode Command Line Tools: {}", path);
        }
        _ => {
            println!("  info: Xcode Command Line Tools: not found (run: xcode-select --install)");
        }
    }

    println!("  info: rax version: {}", VERSION);
}

// ---------------------------------------------------------------------------
// build
// ---------------------------------------------------------------------------

fn run_build(target: &str) {
    let cargo_triple = target_to_triple(target);
    if cargo_triple.is_empty() {
        eprintln!("Unknown target: {}", target);
        eprintln!("Valid targets: ios-sim, ios, android, web, macos");
        process::exit(1);
    }

    println!("rax build --target {}", target);
    println!();
    println!("→ cargo build --target {} --release", cargo_triple);
    println!();
    println!("Run this command in your project directory.");

    if target == "ios-sim" || target == "ios" {
        println!();
        println!("After the build succeeds, open your Xcode project and link the");
        println!(
            "generated `.a` static library from `target/{}/release/`.`",
            cargo_triple
        );
    } else if target == "android" {
        println!();
        println!("Android hosts should load the generated native library from");
        println!(
            "`target/{}/release/` or use cargo-ndk for APK packaging.",
            cargo_triple
        );
    } else if target == "web" {
        println!();
        println!("The web host should load the generated `.wasm` and drive");
        println!("`raxon::web::WebDriver` from requestAnimationFrame.");
    }
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

fn run_run(target: &str) {
    let cargo_triple = target_to_triple(target);
    if cargo_triple.is_empty() || (target != "ios-sim" && target != "ios") {
        if target == "android" || target == "web" || target == "macos" {
            eprintln!("'rax run' currently supports ios-sim and ios targets only.");
            eprintln!(
                "For {} use 'rax build --target {}' and deploy manually.",
                target, target
            );
            process::exit(1);
        }
        eprintln!("Unknown target: {}", target);
        eprintln!("Valid targets for run: ios-sim, ios");
        process::exit(1);
    }

    println!("rax run --target {}", target);
    println!();
    println!("Step 1 — build the library:");
    println!("  cargo build --target {} --release", cargo_triple);
    println!();

    if target == "ios-sim" {
        println!("Step 2 — open your Xcode project and choose an iOS Simulator destination,");
        println!("         then press ▶ Run (or use xcodebuild):");
        println!("  xcodebuild -scheme <YourScheme> -destination 'platform=iOS Simulator,name=iPhone 16' build");
    } else {
        println!("Step 2 — open your Xcode project, select a connected device, then press ▶ Run:");
        println!(
            "  xcodebuild -scheme <YourScheme> -destination 'platform=iOS,id=<DEVICE_UDID>' build"
        );
    }

    println!();
    println!("Run the cargo command first, then rebuild/run in Xcode to pick up the new library.");
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Parse `--target <value>` from args, returning `default_target` if absent.
fn parse_target_flag(args: &[String], default_target: &str) -> String {
    let mut iter = args.iter().skip(2).peekable();
    while let Some(arg) = iter.next() {
        if arg == "--target" || arg == "-t" {
            if let Some(val) = iter.next() {
                return val.clone();
            }
        }
    }
    default_target.to_string()
}

/// Map a friendly target name to a Rust target triple.
fn target_to_triple(target: &str) -> &'static str {
    match target {
        "ios-sim" => "aarch64-apple-ios-sim",
        "ios" => "aarch64-apple-ios",
        "android" => "aarch64-linux-android",
        "web" => "wasm32-unknown-unknown",
        "macos" => "aarch64-apple-darwin",
        _ => "",
    }
}

// ---------------------------------------------------------------------------
// generate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GenerateTarget {
    Android,
    Web,
    All,
}

impl GenerateTarget {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "android" => Ok(GenerateTarget::Android),
            "web" => Ok(GenerateTarget::Web),
            "all" => Ok(GenerateTarget::All),
            _ => Err(format!(
                "Unknown generate target '{value}'. Valid targets: android, web, all"
            )),
        }
    }

    fn includes_android(self) -> bool {
        matches!(self, GenerateTarget::Android | GenerateTarget::All)
    }

    fn includes_web(self) -> bool {
        matches!(self, GenerateTarget::Web | GenerateTarget::All)
    }

    fn as_str(self) -> &'static str {
        match self {
            GenerateTarget::Android => "android",
            GenerateTarget::Web => "web",
            GenerateTarget::All => "all",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GenerateOptions {
    target: GenerateTarget,
    out_dir: PathBuf,
    app_fn: String,
    android_package: String,
    android_class: String,
    android_activity: String,
    android_library: String,
    wasm_module: String,
    web_title: String,
    web_root_id: String,
    host_shells: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        GenerateOptions {
            target: GenerateTarget::All,
            out_dir: PathBuf::from("generated"),
            app_fn: "app".to_string(),
            android_package: "com.example.raxon".to_string(),
            android_class: "RaxonHost".to_string(),
            android_activity: "RaxonActivity".to_string(),
            android_library: "raxon_app".to_string(),
            wasm_module: "./app_wasm.js".to_string(),
            web_title: "Raxon App".to_string(),
            web_root_id: "app".to_string(),
            host_shells: true,
        }
    }
}

fn parse_generate_options(args: &[String]) -> Result<GenerateOptions, String> {
    let mut options = GenerateOptions::default();
    let mut iter = args.iter().skip(2);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--target" | "-t" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "Missing value for --target".to_string())?;
                options.target = GenerateTarget::parse(value)?;
            }
            "--out" | "-o" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "Missing value for --out".to_string())?;
                options.out_dir = PathBuf::from(value);
            }
            "--app-fn" => {
                options.app_fn = iter
                    .next()
                    .ok_or_else(|| "Missing value for --app-fn".to_string())?
                    .clone();
            }
            "--android-package" => {
                options.android_package = iter
                    .next()
                    .ok_or_else(|| "Missing value for --android-package".to_string())?
                    .clone();
            }
            "--android-class" => {
                options.android_class = iter
                    .next()
                    .ok_or_else(|| "Missing value for --android-class".to_string())?
                    .clone();
            }
            "--android-activity" => {
                options.android_activity = iter
                    .next()
                    .ok_or_else(|| "Missing value for --android-activity".to_string())?
                    .clone();
            }
            "--android-library" => {
                options.android_library = iter
                    .next()
                    .ok_or_else(|| "Missing value for --android-library".to_string())?
                    .clone();
            }
            "--wasm-module" => {
                options.wasm_module = iter
                    .next()
                    .ok_or_else(|| "Missing value for --wasm-module".to_string())?
                    .clone();
            }
            "--web-title" => {
                options.web_title = iter
                    .next()
                    .ok_or_else(|| "Missing value for --web-title".to_string())?
                    .clone();
            }
            "--web-root-id" => {
                options.web_root_id = iter
                    .next()
                    .ok_or_else(|| "Missing value for --web-root-id".to_string())?
                    .clone();
            }
            "--host-shells" => {
                options.host_shells = true;
            }
            "--glue-only" | "--no-host-shells" => {
                options.host_shells = false;
            }
            "--help" | "-h" => {
                return Err(generate_usage());
            }
            other => return Err(format!("Unknown generate option '{other}'")),
        }
    }
    validate_rust_path(&options.app_fn, "--app-fn")?;
    validate_android_identifier(&options.android_class, "--android-class")?;
    validate_android_identifier(&options.android_activity, "--android-activity")?;
    validate_android_library_name(&options.android_library)?;
    validate_android_package(&options.android_package)?;
    validate_html_id(&options.web_root_id)?;
    Ok(options)
}

fn generate_usage() -> String {
    [
        "Usage: rax generate [options]",
        "",
        "Options:",
        "  --target android|web|all      Which platform bindings to generate",
        "  --out <dir>                   Output directory (default: generated)",
        "  --app-fn <path>               Rust app factory path (default: app)",
        "  --android-package <package>   Android package (default: com.example.raxon)",
        "  --android-class <name>        Android Kotlin host class (default: RaxonHost)",
        "  --android-activity <name>     Android Activity class (default: RaxonActivity)",
        "  --android-library <name>      Native library loaded by the Activity",
        "  --wasm-module <path>          JS import path for the wasm module",
        "  --web-title <title>           Browser shell document title",
        "  --web-root-id <id>            Browser shell mount element id",
        "  --host-shells                 Generate Android/Web project shells (default)",
        "  --glue-only                   Generate only glue files for brownfield hosts",
    ]
    .join("\n")
}

fn run_generate(options: &GenerateOptions) {
    match generate_bindings(options) {
        Ok(files) => {
            println!(
                "Generated {} host binding/shell file{} in {}",
                files.len(),
                if files.len() == 1 { "" } else { "s" },
                options.out_dir.display()
            );
            for file in files {
                println!("  {}", file.display());
            }
        }
        Err(error) => {
            eprintln!("Failed to generate bindings: {error}");
            process::exit(1);
        }
    }
}

fn generate_bindings(options: &GenerateOptions) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    fs::create_dir_all(&options.out_dir)?;

    if options.target.includes_android() {
        let android_dir = options.out_dir.join("android");
        fs::create_dir_all(&android_dir)?;
        let rust_path = android_dir.join("raxon_android_bridge.rs");
        fs::write(&rust_path, android_rust_bridge_template(options))?;
        files.push(rust_path);

        let kotlin_path = if options.host_shells {
            android_dir
                .join("app/src/main/java")
                .join(android_package_path(&options.android_package))
                .join(format!("{}.kt", options.android_class))
        } else {
            android_dir.join(format!("{}.kt", options.android_class))
        };
        if let Some(parent) = kotlin_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&kotlin_path, android_kotlin_host_template(options))?;
        files.push(kotlin_path);

        if options.host_shells {
            generate_android_host_shell(options, &android_dir, &mut files)?;
        }
    }

    if options.target.includes_web() {
        let web_dir = options.out_dir.join("web");
        fs::create_dir_all(&web_dir)?;
        let rust_path = web_dir.join("raxon_web_bridge.rs");
        fs::write(&rust_path, web_rust_bridge_template(options))?;
        files.push(rust_path);

        let js_path = web_dir.join("raxon-web-host.js");
        fs::write(&js_path, web_js_host_template(options))?;
        files.push(js_path);

        let dts_path = web_dir.join("raxon-web-host.d.ts");
        fs::write(&dts_path, web_types_template())?;
        files.push(dts_path);

        if options.host_shells {
            generate_web_host_shell(options, &web_dir, &mut files)?;
        }
    }

    let manifest_path = options.out_dir.join("raxon-bindings.json");
    fs::write(&manifest_path, binding_manifest_template(options, &files))?;
    files.push(manifest_path);

    Ok(files)
}

fn validate_rust_path(value: &str, flag: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{flag} cannot be empty"));
    }
    let valid = value
        .split("::")
        .all(|segment| is_rust_identifier(segment) || segment == "crate" || segment == "self");
    if valid {
        Ok(())
    } else {
        Err(format!("{flag} must be a Rust path like app or crate::app"))
    }
}

fn is_rust_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn validate_android_identifier(value: &str, flag: &str) -> Result<(), String> {
    if is_rust_identifier(value) {
        Ok(())
    } else {
        Err(format!("{flag} must be an identifier"))
    }
}

fn validate_android_package(value: &str) -> Result<(), String> {
    let valid = value.split('.').all(is_rust_identifier);
    if valid {
        Ok(())
    } else {
        Err("--android-package must be a dotted Java/Kotlin package".to_string())
    }
}

fn validate_android_library_name(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("--android-library cannot be empty".to_string());
    }
    let valid = value
        .chars()
        .all(|ch| ch == '_' || ch == '-' || ch == '.' || ch.is_ascii_alphanumeric());
    if valid {
        Ok(())
    } else {
        Err("--android-library must contain only letters, numbers, '_', '-', or '.'".to_string())
    }
}

fn validate_html_id(value: &str) -> Result<(), String> {
    let mut chars = value.chars();
    let valid = matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '-' || ch.is_ascii_alphanumeric());
    if valid {
        Ok(())
    } else {
        Err("--web-root-id must start with a letter or '_' and contain only letters, numbers, '_' or '-'".to_string())
    }
}

fn jni_function_prefix(package: &str, class: &str) -> String {
    let mut prefix = String::from("Java_");
    let package = package
        .split('.')
        .map(jni_escape_identifier)
        .collect::<Vec<_>>()
        .join("_");
    prefix.push_str(&package);
    prefix.push('_');
    prefix.push_str(&jni_escape_identifier(class));
    prefix
}

fn jni_escape_identifier(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '_' => "_1".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn app_fn_reference(app_fn: &str) -> String {
    if app_fn.starts_with("crate::") || app_fn.starts_with("self::") {
        app_fn.to_string()
    } else {
        format!("crate::{app_fn}")
    }
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn js_string_escape(value: &str) -> String {
    json_escape(value)
}

fn kotlin_string_escape(value: &str) -> String {
    json_escape(value)
}

fn package_name_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "raxon-web-host".to_string()
    } else {
        slug
    }
}

fn gradle_project_name(options: &GenerateOptions) -> String {
    let mut name = options
        .web_title
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ' ' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim()
        .trim_matches('-')
        .to_string();
    if name.is_empty() {
        name = "Raxon App".to_string();
    }
    name
}

fn html_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect::<Vec<_>>(),
            '>' => "&gt;".chars().collect::<Vec<_>>(),
            '"' => "&quot;".chars().collect::<Vec<_>>(),
            '\'' => "&#39;".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn xml_escape(value: &str) -> String {
    html_escape(value)
}

fn android_package_path(package: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for segment in package.split('.') {
        path.push(segment);
    }
    path
}

fn android_rust_bridge_template(options: &GenerateOptions) -> String {
    let prefix = jni_function_prefix(&options.android_package, &options.android_class);
    let app_fn = app_fn_reference(&options.app_fn);
    r##"// Generated by `rax generate --target android`.
//
// Add this module to your app crate and add the Android target dependency:
//
// [target.'cfg(target_os = "android")'.dependencies]
// jni = "0.21"
//
// The generated Kotlin host calls these JNI symbols. The Rust app factory is
// expected to be available as __APP_FN__ and return `impl raxon::view::View`.

use std::cell::RefCell;
use std::ptr;

use jni::objects::{JClass, JString};
use jni::sys::{jfloat, jlong, jstring};
use jni::JNIEnv;

thread_local! {
    static RAXON_ANDROID_BRIDGE: RefCell<raxon::android::AndroidHostBridge> =
        RefCell::new(raxon::android::AndroidHostBridge::new());
}

fn reply_for(request_json: &str) -> String {
    RAXON_ANDROID_BRIDGE.with(|bridge| {
        bridge.borrow_mut().handle_request_json_reply(request_json)
    })
}

#[no_mangle]
pub extern "system" fn __JNI_PREFIX___nativeMount(
    _env: JNIEnv,
    _class: JClass,
    width: jfloat,
    height: jfloat,
) -> jlong {
    let handle = RAXON_ANDROID_BRIDGE.with(|bridge| {
        bridge
            .borrow_mut()
            .mount_android(raxon::core::Size::new(width as f32, height as f32), __APP_FN__)
    });
    handle.to_raw() as jlong
}

#[no_mangle]
pub extern "system" fn __JNI_PREFIX___nativeHandleRequest(
    mut env: JNIEnv,
    _class: JClass,
    request_json: JString,
) -> jstring {
    let request_json = match env.get_string(&request_json) {
        Ok(value) => value.to_string_lossy().into_owned(),
        Err(error) => {
            let message = error.to_string().replace('\\', "\\\\").replace('"', "\\\"");
            let reply = format!(
                r#"{{"protocolVersion":1,"status":"error","error":{{"code":"request_json","message":"failed to read JNI request string: {}"}}}}"#,
                message
            );
            return match env.new_string(reply) {
                Ok(value) => value.into_raw(),
                Err(_) => ptr::null_mut(),
            };
        }
    };
    let reply = reply_for(&request_json);
    match env.new_string(reply) {
        Ok(value) => value.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}
"##
    .replace("__JNI_PREFIX__", &prefix)
    .replace("__APP_FN__", &app_fn)
}

fn android_kotlin_host_template(options: &GenerateOptions) -> String {
    r#"package __ANDROID_PACKAGE__

import android.graphics.Color
import android.graphics.Typeface
import android.text.Editable
import android.text.TextWatcher
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.CompoundButton
import android.widget.DatePicker
import android.widget.EditText
import android.widget.FrameLayout
import android.widget.ImageView
import android.widget.NumberPicker
import android.widget.ProgressBar
import android.widget.ScrollView
import android.widget.SeekBar
import android.widget.Switch
import android.widget.TextView
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.roundToInt

/**
 * Generated raxon Android host.
 *
 * Owns the Kotlin side of the versioned JSON bridge:
 * - mounts the Rust app through nativeMount
 * - sends resize/event/tick requests through nativeHandleRequest
 * - applies command batches to real Android views
 *
 * Keep app-specific widgets in [viewFactory], [attributeApplier], and
 * [platformRequestHandler] so regenerating this file stays safe.
 */
class __ANDROID_CLASS__(private val root: ViewGroup) {
    var handle: Long = 0L
        private set

    val views: MutableMap<Long, View> = linkedMapOf()
    private val installedGestures = mutableSetOf<String>()
    private var suppressEvents = false

    var viewFactory: (String) -> View = { className ->
        when (className) {
            "android.widget.FrameLayout" -> FrameLayout(root.context)
            "android.widget.TextView" -> TextView(root.context)
            "android.widget.Button" -> Button(root.context)
            "android.widget.ImageView" -> ImageView(root.context)
            "android.widget.Switch" -> Switch(root.context)
            "android.widget.SeekBar" -> SeekBar(root.context)
            "android.widget.EditText" -> EditText(root.context)
            "android.widget.ProgressBar" -> ProgressBar(root.context)
            "android.widget.ScrollView" -> ScrollView(root.context)
            "android.widget.NumberPicker" -> NumberPicker(root.context)
            "android.widget.DatePicker" -> DatePicker(root.context)
            "android.view.TextureView" -> android.view.TextureView(root.context)
            "android.webkit.WebView" -> android.webkit.WebView(root.context)
            "android.view.View" -> View(root.context)
            else -> {
                val type = Class.forName(className)
                type.getConstructor(android.content.Context::class.java)
                    .newInstance(root.context) as View
            }
        }
    }

    var commandHandler: (JSONObject) -> Boolean = { false }
    var attributeApplier: (View, JSONObject) -> Boolean = { _, _ -> false }
    var gestureInstaller: (View, String, JSONObject) -> Boolean = { _, _, _ -> false }
    var platformRequestHandler: (JSONObject) -> Unit = {}
    var bridgeErrorHandler: (JSONObject) -> Unit = { error ->
        throw IllegalStateException(error.optString("message", error.toString()))
    }

    fun mount(width: Float = root.width.toFloat(), height: Float = root.height.toFloat()): Long {
        if (handle == 0L) {
            handle = nativeMount(width, height)
        }
        return handle
    }

    fun resize(width: Float, height: Float): JSONObject =
        request(
            JSONObject()
                .put("protocolVersion", 1)
                .put("type", "resize_tick_and_drain_command_batch")
                .put("handle", ensureMounted())
                .put("width", width)
                .put("height", height)
        )

    fun tick(): JSONObject =
        request(
            JSONObject()
                .put("protocolVersion", 1)
                .put("type", "tick_and_drain_command_batch")
                .put("handle", ensureMounted())
        )

    fun dispatchEvents(events: JSONArray): JSONObject =
        request(
            JSONObject()
                .put("protocolVersion", 1)
                .put("type", "dispatch_events_tick_and_drain_command_batch")
                .put("handle", ensureMounted())
                .put(
                    "batch",
                    JSONObject()
                        .put("protocolVersion", 1)
                        .put("events", events)
                )
        )

    fun destroy(): JSONObject {
        val current = ensureMounted()
        val reply = request(
            JSONObject()
                .put("protocolVersion", 1)
                .put("type", "destroy")
                .put("handle", current)
        )
        handle = 0L
        views.clear()
        installedGestures.clear()
        root.removeAllViews()
        return reply
    }

    fun request(request: JSONObject): JSONObject {
        val reply = JSONObject(nativeHandleRequest(request.toString()))
        if (reply.optString("status") == "error") {
            bridgeErrorHandler(reply.getJSONObject("error"))
            return reply
        }
        if (reply.optString("type") == "command_batch") {
            applyCommandBatch(reply.getJSONObject("batch"))
        }
        return reply
    }

    fun applyCommandBatch(batch: JSONObject) {
        val commands = batch.optJSONArray("commands") ?: JSONArray()
        for (index in 0 until commands.length()) {
            applyCommand(commands.getJSONObject(index))
        }
    }

    fun applyCommand(command: JSONObject) {
        if (commandHandler(command)) return
        when (command.getString("type")) {
            "create" -> {
                val id = command.getLong("id")
                val className = command.getString("class_name")
                val view = viewFactory(className)
                if (id in 1..Int.MAX_VALUE.toLong()) {
                    view.id = id.toInt()
                }
                views[id] = view
                installBuiltInListeners(id, view)
            }
            "set_root" -> {
                val view = views[command.getLong("id")] ?: return
                (view.parent as? ViewGroup)?.removeView(view)
                root.removeAllViews()
                root.addView(view)
            }
            "set_frame" -> {
                val view = views[command.getLong("id")] ?: return
                applyFrame(view, command)
            }
            "insert_child" -> {
                val parent = views[command.getLong("parent")] as? ViewGroup ?: return
                val child = views[command.getLong("child")] ?: return
                val index = command.getInt("index").coerceAtMost(parent.childCount)
                if (child.parent is ViewGroup) {
                    (child.parent as ViewGroup).removeView(child)
                }
                parent.addView(child, index)
            }
            "remove_child" -> {
                val parent = views[command.getLong("parent")] as? ViewGroup ?: return
                val child = views[command.getLong("child")] ?: return
                parent.removeView(child)
            }
            "destroy" -> {
                val id = command.getLong("id")
                installedGestures.removeAll { it.startsWith("$id:") }
                views.remove(id)?.let { view ->
                    (view.parent as? ViewGroup)?.removeView(view)
                }
            }
            "set_attribute" -> {
                val view = views.getValue(command.getLong("id"))
                applyAttribute(view, command.getJSONObject("attr"))
            }
            "set_backdrop" -> root.setBackgroundColor(command.getLong("argb").toInt())
            "request" -> platformRequestHandler(command.optJSONObject("request") ?: command)
            "add_gesture" -> {
                val view = views[command.getLong("id")] ?: return
                installGesture(command.getLong("id"), view, command)
            }
            "set_content_size" -> {
                val view = views[command.getLong("id")] ?: return
                applyContentSize(view, command)
            }
            "scroll_to" -> {
                val view = views[command.getLong("id")] ?: return
                scrollTo(view, command)
            }
            "scroll_to_top" -> {
                val view = views[command.getLong("id")] ?: return
                scrollTo(
                    view,
                    JSONObject()
                        .put("offset_x", 0.0)
                        .put("offset_y", 0.0)
                        .put("animated", command.optBoolean("animated", false))
                )
            }
            "haptic" -> Unit
        }
    }

    private fun applyAttribute(view: View, attr: JSONObject) {
        if (attributeApplier(view, attr)) return
        when (attr.getString("name")) {
            "text" -> withoutEventEcho { (view as? TextView)?.text = attr.optString("value") }
            "font_size" -> (view as? TextView)?.textSize = attr.optDouble("value").toFloat()
            "text_color" -> (view as? TextView)?.setTextColor(attr.optLong("value").toInt())
            "background_color" -> view.setBackgroundColor(attr.optLong("value").toInt())
            "bool_value" -> withoutEventEcho {
                (view as? CompoundButton)?.isChecked = attr.optBoolean("value")
                view.isSelected = attr.optBoolean("value")
            }
            "float_value" -> withoutEventEcho {
                when (view) {
                    is SeekBar -> view.progress = (attr.optDouble("value") * view.max).roundToInt()
                    is NumberPicker -> view.value = attr.optDouble("value").roundToInt()
                    else -> Unit
                }
            }
            "placeholder" -> (view as? TextView)?.hint = attr.optString("value")
            "font_weight" -> {
                val style = if (attr.optDouble("value", 400.0) >= 600.0) Typeface.BOLD else Typeface.NORMAL
                (view as? TextView)?.setTypeface(Typeface.DEFAULT, style)
            }
            "opacity" -> view.alpha = attr.optDouble("value", 1.0).toFloat()
            "image_source", "url" -> {
                if (view is android.webkit.WebView) view.loadUrl(attr.optString("value"))
            }
            "accessibility_label" -> view.contentDescription = attr.optString("value")
            "accessibility_hidden" -> view.importantForAccessibility =
                if (attr.optBoolean("value")) View.IMPORTANT_FOR_ACCESSIBILITY_NO
                else View.IMPORTANT_FOR_ACCESSIBILITY_AUTO
            "event_listener" -> installEventListener(view, attr.optJSONObject("value"))
            "unsupported" -> Unit
        }
    }

    private fun installBuiltInListeners(id: Long, view: View) {
        when (view) {
            is Button -> installTapListener(id, view)
            is CompoundButton -> view.setOnCheckedChangeListener { _, checked ->
                if (!suppressEvents) emitEvent(
                    JSONObject()
                        .put("type", "value_changed")
                        .put("target", id)
                        .put("value", if (checked) 1.0 else 0.0)
                )
            }
            is SeekBar -> view.setOnSeekBarChangeListener(object : SeekBar.OnSeekBarChangeListener {
                override fun onProgressChanged(seekBar: SeekBar, progress: Int, fromUser: Boolean) {
                    if (fromUser && !suppressEvents) emitEvent(
                        JSONObject()
                            .put("type", "value_changed")
                            .put("target", id)
                            .put("value", progress.toDouble() / seekBar.max.coerceAtLeast(1))
                    )
                }

                override fun onStartTrackingTouch(seekBar: SeekBar) = Unit
                override fun onStopTrackingTouch(seekBar: SeekBar) = Unit
            })
            is EditText -> view.addTextChangedListener(object : TextWatcher {
                override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) = Unit
                override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) = Unit
                override fun afterTextChanged(s: Editable?) {
                    if (!suppressEvents) emitEvent(
                        JSONObject()
                            .put("type", "text_changed")
                            .put("target", id)
                            .put("value", s?.toString().orEmpty())
                            .put("selection_start", view.selectionStart.coerceAtLeast(0))
                            .put("selection_end", view.selectionEnd.coerceAtLeast(0))
                    )
                }
            })
        }
    }

    private fun installGesture(id: Long, view: View, command: JSONObject) {
        val gesture = command.optString("gesture")
        val key = "$id:$gesture"
        if (!installedGestures.add(key)) return
        if (gestureInstaller(view, gesture, command)) return
        when (gesture) {
            "Tap" -> installTapListener(id, view)
            "LongPress" -> view.setOnLongClickListener {
                emitEvent(JSONObject().put("type", "long_press").put("target", id))
                true
            }
            else -> Unit
        }
    }

    private fun installEventListener(view: View, value: JSONObject?) {
        val id = views.entries.firstOrNull { it.value === view }?.key ?: return
        when (value?.optString("event")) {
            "press_in" -> view.setOnTouchListener { _, event ->
                if (event.action == android.view.MotionEvent.ACTION_DOWN) {
                    emitEvent(JSONObject().put("type", "pointer_down").put("target", id).put("x", event.x).put("y", event.y).put("pointer", 0))
                }
                false
            }
            "press_out" -> view.setOnTouchListener { _, event ->
                if (event.action == android.view.MotionEvent.ACTION_UP) {
                    emitEvent(JSONObject().put("type", "pointer_up").put("target", id).put("x", event.x).put("y", event.y).put("pointer", 0))
                }
                false
            }
        }
    }

    private fun installTapListener(id: Long, view: View) {
        view.isClickable = true
        view.setOnClickListener {
            emitEvent(JSONObject().put("type", "tap").put("target", id))
        }
    }

    private fun applyFrame(view: View, command: JSONObject) {
        val width = command.getDouble("width").roundToInt().coerceAtLeast(0)
        val height = command.getDouble("height").roundToInt().coerceAtLeast(0)
        val params = view.layoutParams ?: ViewGroup.LayoutParams(width, height)
        params.width = width
        params.height = height
        view.layoutParams = params
        view.x = command.getDouble("x").toFloat()
        view.y = command.getDouble("y").toFloat()
    }

    private fun applyContentSize(view: View, command: JSONObject) {
        val width = command.getDouble("width").roundToInt().coerceAtLeast(0)
        val height = command.getDouble("height").roundToInt().coerceAtLeast(0)
        val child = (view as? ViewGroup)?.getChildAt(0) ?: return
        val params = child.layoutParams ?: ViewGroup.LayoutParams(width, height)
        params.width = width
        params.height = height
        child.layoutParams = params
    }

    private fun scrollTo(view: View, command: JSONObject) {
        val x = command.optDouble("offset_x", 0.0).roundToInt()
        val y = command.optDouble("offset_y", 0.0).roundToInt()
        if (view is ScrollView && command.optBoolean("animated", false)) {
            view.smoothScrollTo(x, y)
        } else {
            view.scrollTo(x, y)
        }
    }

    private fun emitEvent(event: JSONObject) {
        dispatchEvents(JSONArray().put(event))
    }

    private inline fun withoutEventEcho(block: () -> Unit) {
        suppressEvents = true
        try {
            block()
        } finally {
            suppressEvents = false
        }
    }

    private fun ensureMounted(): Long {
        if (handle == 0L) mount()
        return handle
    }

    companion object {
        fun loadLibrary(name: String) = System.loadLibrary(name)

        @JvmStatic external fun nativeMount(width: Float, height: Float): Long
        @JvmStatic external fun nativeHandleRequest(requestJson: String): String
    }
}
"#
    .replace("__ANDROID_PACKAGE__", &options.android_package)
    .replace("__ANDROID_CLASS__", &options.android_class)
}

fn generate_android_host_shell(
    options: &GenerateOptions,
    android_dir: &Path,
    files: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    let java_dir = android_dir
        .join("app/src/main/java")
        .join(android_package_path(&options.android_package));
    fs::create_dir_all(&java_dir)?;

    let activity_path = java_dir.join(format!("{}.kt", options.android_activity));
    fs::write(&activity_path, android_activity_template(options))?;
    files.push(activity_path);

    let manifest_path = android_dir.join("app/src/main/AndroidManifest.xml");
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&manifest_path, android_manifest_template(options))?;
    files.push(manifest_path);

    let values_dir = android_dir.join("app/src/main/res/values");
    fs::create_dir_all(&values_dir)?;

    let strings_path = values_dir.join("strings.xml");
    fs::write(&strings_path, android_strings_template(options))?;
    files.push(strings_path);

    let styles_path = values_dir.join("styles.xml");
    fs::write(&styles_path, android_styles_template())?;
    files.push(styles_path);

    let settings_path = android_dir.join("settings.gradle.kts");
    fs::write(&settings_path, android_settings_gradle_template(options))?;
    files.push(settings_path);

    let root_build_path = android_dir.join("build.gradle.kts");
    fs::write(&root_build_path, android_root_build_gradle_template())?;
    files.push(root_build_path);

    let app_build_path = android_dir.join("app/build.gradle.kts");
    fs::write(&app_build_path, android_app_build_gradle_template(options))?;
    files.push(app_build_path);

    let wrapper_path = android_dir.join("gradle/wrapper/gradle-wrapper.properties");
    if let Some(parent) = wrapper_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&wrapper_path, android_gradle_wrapper_properties_template())?;
    files.push(wrapper_path);

    let readme_path = android_dir.join("README.md");
    fs::write(&readme_path, android_shell_readme_template(options))?;
    files.push(readme_path);

    Ok(())
}

fn android_activity_template(options: &GenerateOptions) -> String {
    r#"package __ANDROID_PACKAGE__

import android.app.Activity
import android.os.Bundle
import android.view.Choreographer
import android.widget.FrameLayout
import org.json.JSONArray
import org.json.JSONObject

/**
 * Generated raxon Android Activity shell.
 *
 * It owns the Android view root, loads the Rust cdylib, mounts the generated
 * host bridge after the first layout, drives ticks from Choreographer, forwards
 * size changes, and sends system back as a versioned raxon event.
 */
open class __ANDROID_ACTIVITY__ : Activity() {
    protected lateinit var root: FrameLayout
        private set
    protected lateinit var host: __ANDROID_CLASS__
        private set

    private var running = false
    private val frameCallback = object : Choreographer.FrameCallback {
        override fun doFrame(frameTimeNanos: Long) {
            if (!running) return
            if (::host.isInitialized && host.handle != 0L) {
                host.tick()
            }
            Choreographer.getInstance().postFrameCallback(this)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        __ANDROID_CLASS__.loadLibrary(NATIVE_LIBRARY)

        root = FrameLayout(this)
        root.clipToPadding = false
        setContentView(root)

        host = __ANDROID_CLASS__(root)
        installDefaultPlatformHandlers(host)
        root.addOnLayoutChangeListener { _, left, top, right, bottom, oldLeft, oldTop, oldRight, oldBottom ->
            val width = (right - left).toFloat()
            val height = (bottom - top).toFloat()
            val oldWidth = (oldRight - oldLeft).toFloat()
            val oldHeight = (oldBottom - oldTop).toFloat()
            if (width > 0f && height > 0f && (width != oldWidth || height != oldHeight)) {
                mountOrResize(width, height)
            }
        }
    }

    override fun onResume() {
        super.onResume()
        startFrameLoop()
    }

    override fun onPause() {
        stopFrameLoop()
        super.onPause()
    }

    override fun onDestroy() {
        if (::host.isInitialized && host.handle != 0L) {
            host.destroy()
        }
        super.onDestroy()
    }

    override fun onBackPressed() {
        if (::host.isInitialized && host.handle != 0L) {
            host.dispatchEvents(JSONArray().put(JSONObject().put("type", "back_pressed")))
        } else {
            super.onBackPressed()
        }
    }

    protected open fun installDefaultPlatformHandlers(host: __ANDROID_CLASS__) {
        host.platformRequestHandler = { request ->
            when (request.optString("type")) {
                "announce_accessibility" -> {
                    root.announceForAccessibility(request.optString("message"))
                }
                "request_focus" -> {
                    host.views[request.optLong("id")]?.requestFocus()
                }
            }
        }
    }

    protected fun mountOrResize(width: Float = root.width.toFloat(), height: Float = root.height.toFloat()) {
        if (width <= 0f || height <= 0f) return
        if (host.handle == 0L) {
            host.mount(width, height)
        } else {
            host.resize(width, height)
        }
    }

    private fun startFrameLoop() {
        if (running) return
        running = true
        mountOrResize()
        Choreographer.getInstance().postFrameCallback(frameCallback)
    }

    private fun stopFrameLoop() {
        if (!running) return
        running = false
        Choreographer.getInstance().removeFrameCallback(frameCallback)
    }

    companion object {
        const val NATIVE_LIBRARY: String = "__ANDROID_LIBRARY__"
    }
}
"#
    .replace("__ANDROID_PACKAGE__", &options.android_package)
    .replace("__ANDROID_ACTIVITY__", &options.android_activity)
    .replace("__ANDROID_CLASS__", &options.android_class)
    .replace("__ANDROID_LIBRARY__", &options.android_library)
}

fn android_manifest_template(options: &GenerateOptions) -> String {
    r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application
        android:allowBackup="true"
        android:label="@string/app_name"
        android:theme="@style/RaxonTheme">
        <activity
            android:name="__ANDROID_PACKAGE__.__ANDROID_ACTIVITY__"
            android:configChanges="keyboard|keyboardHidden|orientation|screenLayout|screenSize|smallestScreenSize|uiMode"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
"#
    .replace("__ANDROID_PACKAGE__", &options.android_package)
    .replace("__ANDROID_ACTIVITY__", &options.android_activity)
}

fn android_strings_template(options: &GenerateOptions) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">{}</string>
</resources>
"#,
        xml_escape(&options.web_title)
    )
}

fn android_styles_template() -> String {
    r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <style name="RaxonTheme" parent="@android:style/Theme.Material.Light.NoActionBar">
        <item name="android:windowNoTitle">true</item>
        <item name="android:windowActionBar">false</item>
        <item name="android:windowLightStatusBar">true</item>
        <item name="android:navigationBarColor">#000000</item>
        <item name="android:windowDisablePreview">true</item>
    </style>
</resources>
"#
    .to_string()
}

fn android_settings_gradle_template(options: &GenerateOptions) -> String {
    format!(
        r#"pluginManagement {{
    repositories {{
        google()
        mavenCentral()
        gradlePluginPortal()
    }}
}}

dependencyResolutionManagement {{
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {{
        google()
        mavenCentral()
    }}
}}

rootProject.name = "{project_name}"
include(":app")
"#,
        project_name = kotlin_string_escape(&gradle_project_name(options)),
    )
}

fn android_root_build_gradle_template() -> String {
    format!(
        r#"plugins {{
    id("com.android.application") version "{agp_version}" apply false
}}
"#,
        agp_version = ANDROID_GRADLE_PLUGIN_VERSION,
    )
}

fn android_app_build_gradle_template(options: &GenerateOptions) -> String {
    format!(
        r#"plugins {{
    id("com.android.application")
}}

android {{
    namespace = "{namespace}"
    compileSdk = {compile_sdk}

    defaultConfig {{
        applicationId = "{application_id}"
        minSdk = {min_sdk}
        targetSdk = {target_sdk}
        versionCode = 1
        versionName = "0.1.0"
    }}

    sourceSets {{
        getByName("main") {{
            java.srcDir("src/main/java")
            jniLibs.srcDir("src/main/jniLibs")
        }}
    }}
}}
"#,
        namespace = kotlin_string_escape(&options.android_package),
        application_id = kotlin_string_escape(&options.android_package),
        compile_sdk = ANDROID_COMPILE_SDK,
        min_sdk = ANDROID_MIN_SDK,
        target_sdk = ANDROID_TARGET_SDK,
    )
}

fn android_gradle_wrapper_properties_template() -> String {
    format!(
        r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-{gradle_version}-bin.zip
networkTimeout=10000
validateDistributionUrl=true
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
"#,
        gradle_version = GRADLE_WRAPPER_VERSION,
    )
}

fn android_shell_readme_template(options: &GenerateOptions) -> String {
    format!(
        r#"# raxon Android Host Shell

Generated by `rax generate --target android`.

## Files

- `raxon_android_bridge.rs`: Rust JNI bridge module for your app crate.
- `app/src/main/java/{package_path}/{host_class}.kt`: generated Android view host.
- `app/src/main/java/{package_path}/{activity}.kt`: Activity shell that mounts the Rust app, drives `Choreographer`, handles resize, and forwards back events.
- `app/src/main/AndroidManifest.xml`: launcher Activity declaration.
- `app/src/main/res/values/*.xml`: minimal resources for the generated Activity.
- `settings.gradle.kts`, `build.gradle.kts`, and `app/build.gradle.kts`: Android application project wired to AGP {agp_version}.
- `gradle/wrapper/gradle-wrapper.properties`: Gradle {gradle_version} distribution metadata for reproducible wrapper generation.

## Rust side

Include `raxon_android_bridge.rs` from your app crate and build a `cdylib` named
`{library}` for `aarch64-linux-android` (for example with `cargo ndk`). The
generated Activity calls `System.loadLibrary("{library}")`.

Place the built native libraries under `app/src/main/jniLibs/<abi>/lib{library}.so`
or wire your CI to copy them there after the Rust build.

## Android side

This directory is a standalone Android project skeleton. If the Gradle wrapper
scripts are not already present, run:

```sh
gradle wrapper --gradle-version {gradle_version}
./gradlew :app:assembleDebug
```

For brownfield apps, copy the `app/src/main` tree into an Android application
module, or merge these files into an existing module. Override
`{activity}.installDefaultPlatformHandlers` or set hooks on `{host_class}` for
platform services and custom widgets.
"#,
        package_path = options.android_package.replace('.', "/"),
        host_class = options.android_class,
        activity = options.android_activity,
        library = options.android_library,
        agp_version = ANDROID_GRADLE_PLUGIN_VERSION,
        gradle_version = GRADLE_WRAPPER_VERSION,
    )
}

fn web_rust_bridge_template(options: &GenerateOptions) -> String {
    let app_fn = app_fn_reference(&options.app_fn);
    r#"// Generated by `rax generate --target web`.
//
// Include this module in your app crate when building for wasm32-unknown-unknown.
// It exposes a tiny string ABI consumed by raxon-web-host.js without requiring
// wasm-bindgen in the app's public API.

use std::cell::RefCell;
use std::slice;

thread_local! {
    static RAXON_WEB_BRIDGE: RefCell<raxon::web::WebHostBridge> =
        RefCell::new(raxon::web::WebHostBridge::new());
    static RAXON_WEB_REPLY: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

#[no_mangle]
pub extern "C" fn raxon_web_mount(width: f32, height: f32) -> u64 {
    RAXON_WEB_BRIDGE.with(|bridge| {
        bridge
            .borrow_mut()
            .mount_web(raxon::core::Size::new(width, height), __APP_FN__)
            .to_raw()
    })
}

#[no_mangle]
pub extern "C" fn raxon_web_alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len);
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer
}

#[no_mangle]
pub unsafe extern "C" fn raxon_web_dealloc(pointer: *mut u8, capacity: usize) {
    if !pointer.is_null() && capacity > 0 {
        drop(Vec::from_raw_parts(pointer, 0, capacity));
    }
}

#[no_mangle]
pub unsafe extern "C" fn raxon_web_handle_request(pointer: *const u8, len: usize) -> usize {
    let bytes = if pointer.is_null() || len == 0 {
        &[]
    } else {
        slice::from_raw_parts(pointer, len)
    };
    let request = String::from_utf8_lossy(bytes);
    let reply = RAXON_WEB_BRIDGE.with(|bridge| {
        bridge.borrow_mut().handle_request_json_reply(&request)
    });
    RAXON_WEB_REPLY.with(|last| {
        let mut last = last.borrow_mut();
        last.clear();
        last.extend_from_slice(reply.as_bytes());
        last.len()
    })
}

#[no_mangle]
pub extern "C" fn raxon_web_reply_ptr() -> *const u8 {
    RAXON_WEB_REPLY.with(|last| last.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn raxon_web_reply_len() -> usize {
    RAXON_WEB_REPLY.with(|last| last.borrow().len())
}
"#
    .replace("__APP_FN__", &app_fn)
}

fn web_js_host_template(options: &GenerateOptions) -> String {
    r#"const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

export async function createRaxonWebHost(root, options = {}) {
  const module = options.wasm ?? await import("__WASM_MODULE__");
  if (typeof module.default === "function" && options.initialize !== false) {
    await module.default(options.wasmUrl);
  }
  const host = new RaxonWebHost({
    root,
    wasm: module,
    memory: options.memory,
    onBridgeError: options.onBridgeError,
    handleCommand: options.handleCommand,
    applyAttribute: options.applyAttribute,
    installGesture: options.installGesture,
    handlePlatformRequest: options.handlePlatformRequest,
  });
  if (options.mount !== false) host.mount();
  return host;
}

export class RaxonWebHost {
  constructor({ root, wasm, memory, onBridgeError, handleCommand, applyAttribute, installGesture, handlePlatformRequest }) {
    this.root = root;
    this.wasm = wasm;
    this.memory = memory ?? wasm.memory;
    if (!this.memory) {
      throw new Error("RaxonWebHost requires a WebAssembly.Memory export or options.memory");
    }
    this.handle = 0n;
    this.nodes = new Map();
    this.listenerDisposers = new Map();
    this.onBridgeError = onBridgeError ?? ((error) => { throw new Error(error.message); });
    this.handleCommand = handleCommand ?? (() => false);
    this.applyAttributeHook = applyAttribute ?? (() => false);
    this.installGestureHook = installGesture ?? (() => false);
    this.handlePlatformRequest = handlePlatformRequest ?? (() => {});
  }

  mount(width = this.root.clientWidth, height = this.root.clientHeight) {
    if (!this.handle) {
      this.handle = BigInt(this.wasm.raxon_web_mount(width, height));
    }
    return this.handle;
  }

  resize(width, height) {
    return this.request({
      protocolVersion: 1,
      type: "resize_tick_and_drain_command_batch",
      handle: Number(this.ensureMounted()),
      width,
      height,
    });
  }

  tick() {
    return this.request({
      protocolVersion: 1,
      type: "tick_and_drain_command_batch",
      handle: Number(this.ensureMounted()),
    });
  }

  dispatchEvents(events) {
    return this.request({
      protocolVersion: 1,
      type: "dispatch_events_tick_and_drain_command_batch",
      handle: Number(this.ensureMounted()),
      batch: { protocolVersion: 1, events },
    });
  }

  destroy() {
    const reply = this.request({
      protocolVersion: 1,
      type: "destroy",
      handle: Number(this.ensureMounted()),
    });
    this.handle = 0n;
    for (const id of this.nodes.keys()) this.removeNodeListeners(id);
    this.nodes.clear();
    this.root.replaceChildren();
    return reply;
  }

  request(request) {
    const reply = this.callBridge(JSON.stringify(request));
    if (reply.status === "error") {
      this.onBridgeError(reply.error);
      return reply;
    }
    if (reply.type === "command_batch") {
      this.applyCommandBatch(reply.batch);
    }
    return reply;
  }

  callBridge(json) {
    const bytes = textEncoder.encode(json);
    const pointer = this.wasm.raxon_web_alloc(bytes.length);
    new Uint8Array(this.memory.buffer, pointer, bytes.length).set(bytes);
    let replyLen = 0;
    try {
      replyLen = this.wasm.raxon_web_handle_request(pointer, bytes.length);
    } finally {
      this.wasm.raxon_web_dealloc(pointer, bytes.length);
    }
    const replyPtr = this.wasm.raxon_web_reply_ptr();
    const replyBytes = new Uint8Array(this.memory.buffer, replyPtr, replyLen);
    return JSON.parse(textDecoder.decode(replyBytes));
  }

  applyCommandBatch(batch) {
    for (const command of batch.commands ?? []) {
      this.applyCommand(command);
    }
  }

  applyCommand(command) {
    if (this.handleCommand(command, this)) return;
    switch (command.type) {
      case "create": {
        const element = document.createElement(command.tag_name);
        if (command.input_type) element.type = command.input_type;
        element.dataset.raxonId = String(command.id);
        element.style.position = "absolute";
        this.nodes.set(command.id, element);
        this.installBuiltInListeners(command.id, element);
        break;
      }
      case "set_root": {
        const node = this.nodes.get(command.id);
        if (node) this.root.replaceChildren(node);
        break;
      }
      case "set_frame": {
        const node = this.nodes.get(command.id);
        if (!node) break;
        Object.assign(node.style, {
          left: `${command.x}px`,
          top: `${command.y}px`,
          width: `${command.width}px`,
          height: `${command.height}px`,
        });
        break;
      }
      case "insert_child": {
        const parent = this.nodes.get(command.parent);
        const child = this.nodes.get(command.child);
        if (parent && child) parent.insertBefore(child, parent.children[command.index] ?? null);
        break;
      }
      case "remove_child": {
        const child = this.nodes.get(command.child);
        if (child?.parentElement?.dataset.raxonId === String(command.parent) || child?.parentElement === this.nodes.get(command.parent)) {
          child.remove();
        }
        break;
      }
      case "destroy": {
        this.removeNodeListeners(command.id);
        this.nodes.get(command.id)?.remove();
        this.nodes.delete(command.id);
        break;
      }
      case "set_attribute": {
        const node = this.nodes.get(command.id);
        if (node) this.applyAttribute(node, command.attr);
        break;
      }
      case "set_backdrop":
        this.root.style.background = command.css_color;
        break;
      case "add_gesture":
        this.installGesture(command);
        break;
      case "set_content_size":
        this.applyContentSize(command);
        break;
      case "scroll_to":
        this.nodes.get(command.id)?.scrollTo({
          left: command.offset_x,
          top: command.offset_y,
          behavior: command.animated ? "smooth" : "auto",
        });
        break;
      case "scroll_to_top":
        this.nodes.get(command.id)?.scrollTo({
          left: 0,
          top: 0,
          behavior: command.animated ? "smooth" : "auto",
        });
        break;
      case "haptic":
        if (typeof navigator !== "undefined" && navigator.vibrate) {
          navigator.vibrate(command.style === "Heavy" ? 35 : 15);
        }
        break;
      case "request":
        this.handlePlatformRequest(command.request ?? command);
        break;
    }
  }

  applyAttribute(node, attr) {
    if (this.applyAttributeHook(node, attr)) return;
    switch (attr.name) {
      case "text":
        node.textContent = attr.value;
        break;
      case "font_size":
        node.style.fontSize = `${attr.value}px`;
        break;
      case "text_color":
        node.style.color = attr.value;
        break;
      case "background_color":
        node.style.backgroundColor = attr.value;
        break;
      case "border_color":
        node.style.borderColor = attr.value;
        break;
      case "border_width":
        node.style.borderWidth = `${attr.value}px`;
        node.style.borderStyle = node.style.borderStyle || "solid";
        break;
      case "corner_radius":
        node.style.borderRadius = `${attr.value}px`;
        break;
      case "image_source":
        node.src = attr.value;
        break;
      case "url":
        node.src = attr.value;
        break;
      case "placeholder":
        node.placeholder = attr.value;
        break;
      case "bool_value":
        node.checked = Boolean(attr.value);
        break;
      case "float_value":
        node.value = String(attr.value);
        if (node.tagName === "PROGRESS") node.value = attr.value;
        break;
      case "accessibility_label":
        node.setAttribute("aria-label", attr.value);
        break;
      case "accessibility_hidden":
        node.setAttribute("aria-hidden", attr.value ? "true" : "false");
        break;
      case "opacity":
        node.style.opacity = String(attr.value);
        break;
      case "font_weight":
        node.style.fontWeight = String(attr.value);
        break;
      case "italic":
        node.style.fontStyle = attr.value ? "italic" : "";
        break;
      case "text_align":
        node.style.textAlign = String(attr.value).toLowerCase();
        break;
      case "event_listener":
        this.installEventListener(node, attr.value);
        break;
      case "unsupported":
        break;
    }
  }

  installBuiltInListeners(id, node) {
    if (node.tagName === "BUTTON") {
      this.addNodeListener(id, node, "click", () => {
        this.dispatchEvents([{ type: "tap", target: id }]);
      });
    }
    if (node.tagName === "INPUT" || node.tagName === "TEXTAREA") {
      this.addNodeListener(id, node, "input", () => {
        if (node.type === "checkbox") {
          this.dispatchEvents([{ type: "value_changed", target: id, value: node.checked ? 1 : 0 }]);
        } else if (node.type === "range" || node.type === "number") {
          this.dispatchEvents([{ type: "value_changed", target: id, value: Number(node.value) || 0 }]);
        } else {
          this.dispatchEvents([{
            type: "text_changed",
            target: id,
            value: node.value ?? "",
            selection_start: node.selectionStart ?? 0,
            selection_end: node.selectionEnd ?? 0,
          }]);
        }
      });
      this.addNodeListener(id, node, "keydown", (event) => {
        if (event.key === "Enter") this.dispatchEvents([{ type: "submit", target: id }]);
      });
    }
  }

  installGesture(command) {
    const node = this.nodes.get(command.id);
    if (!node || this.installGestureHook(node, command, this)) return;
    switch (command.gesture) {
      case "Tap":
        this.addNodeListener(command.id, node, "click", () => {
          this.dispatchEvents([{ type: "tap", target: command.id }]);
        }, `gesture:${command.gesture}`);
        break;
      case "DoubleTap":
        this.addNodeListener(command.id, node, "dblclick", () => {
          this.dispatchEvents([{ type: "double_tap", target: command.id }]);
        }, `gesture:${command.gesture}`);
        break;
      case "LongPress":
        this.installLongPress(command.id, node);
        break;
    }
  }

  installLongPress(id, node) {
    let timeout = 0;
    const clear = () => {
      if (timeout) clearTimeout(timeout);
      timeout = 0;
    };
    this.addNodeListener(id, node, "pointerdown", () => {
      clear();
      timeout = setTimeout(() => {
        timeout = 0;
        this.dispatchEvents([{ type: "long_press", target: id }]);
      }, 500);
    }, "gesture:LongPress:start");
    this.addNodeListener(id, node, "pointerup", clear, "gesture:LongPress:end");
    this.addNodeListener(id, node, "pointercancel", clear, "gesture:LongPress:cancel");
    this.addNodeListener(id, node, "pointerleave", clear, "gesture:LongPress:leave");
  }

  installEventListener(node, value) {
    const id = Number(node.dataset.raxonId);
    switch (value?.event) {
      case "scroll_change":
        this.addNodeListener(id, node, "scroll", () => {
          this.dispatchEvents([{
            type: "scroll_changed",
            target: id,
            offset_x: node.scrollLeft,
            offset_y: node.scrollTop,
          }]);
        }, "event:scroll_change");
        break;
      case "image_load":
        this.addNodeListener(id, node, "load", () => {
          this.handlePlatformRequest({ type: "image_load", target: id });
        }, "event:image_load");
        break;
      case "image_error":
        this.addNodeListener(id, node, "error", () => {
          this.handlePlatformRequest({ type: "image_error", target: id });
        }, "event:image_error");
        break;
    }
  }

  applyContentSize(command) {
    const node = this.nodes.get(command.id);
    if (!node) return;
    Object.assign(node.style, {
      minWidth: `${command.width}px`,
      minHeight: `${command.height}px`,
    });
  }

  addNodeListener(id, node, event, listener, suffix = event) {
    const key = `${id}:${suffix}`;
    if (this.listenerDisposers.has(key)) return;
    node.addEventListener(event, listener);
    this.listenerDisposers.set(key, () => node.removeEventListener(event, listener));
  }

  removeNodeListeners(id) {
    const prefix = `${id}:`;
    for (const [key, dispose] of this.listenerDisposers) {
      if (key.startsWith(prefix)) {
        dispose();
        this.listenerDisposers.delete(key);
      }
    }
  }

  ensureMounted() {
    if (!this.handle) this.mount();
    return this.handle;
  }
}
"#
    .replace("__WASM_MODULE__", &js_string_escape(&options.wasm_module))
}

fn web_types_template() -> String {
    r#"export type RaxonBridgeReply =
  | ({ protocolVersion: number; status: "ok" } & Record<string, unknown>)
  | { protocolVersion: number; status: "error"; error: RaxonBridgeError };

export interface RaxonBridgeError {
  code: string;
  message: string;
  handle?: number;
  expectedVersion?: number;
  foundVersion?: number;
}

export interface RaxonWebHostOptions {
  wasm?: Record<string, any>;
  memory?: WebAssembly.Memory;
  wasmUrl?: string;
  initialize?: boolean;
  mount?: boolean;
  onBridgeError?: (error: RaxonBridgeError) => void;
  handleCommand?: (command: Record<string, any>, host: RaxonWebHost) => boolean;
  applyAttribute?: (node: HTMLElement, attr: Record<string, any>) => boolean;
  installGesture?: (
    node: HTMLElement,
    command: Record<string, any>,
    host: RaxonWebHost,
  ) => boolean;
  handlePlatformRequest?: (request: Record<string, any>) => void;
}

export function createRaxonWebHost(
  root: HTMLElement,
  options?: RaxonWebHostOptions,
): Promise<RaxonWebHost>;

export class RaxonWebHost {
  constructor(options: { root: HTMLElement; wasm: Record<string, any> } & RaxonWebHostOptions);
  readonly root: HTMLElement;
  readonly memory: WebAssembly.Memory;
  handle: bigint;
  mount(width?: number, height?: number): bigint;
  resize(width: number, height: number): RaxonBridgeReply;
  tick(): RaxonBridgeReply;
  dispatchEvents(events: unknown[]): RaxonBridgeReply;
  destroy(): RaxonBridgeReply;
  request(request: Record<string, any>): RaxonBridgeReply;
  applyCommandBatch(batch: { commands?: unknown[] }): void;
  applyCommand(command: Record<string, any>): void;
  applyAttribute(node: HTMLElement, attr: Record<string, any>): void;
}
"#
    .to_string()
}

fn generate_web_host_shell(
    options: &GenerateOptions,
    web_dir: &Path,
    files: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    let index_path = web_dir.join("index.html");
    fs::write(&index_path, web_index_template(options))?;
    files.push(index_path);

    let main_path = web_dir.join("main.js");
    fs::write(&main_path, web_main_template(options))?;
    files.push(main_path);

    let package_path = web_dir.join("package.json");
    fs::write(&package_path, web_package_json_template(options))?;
    files.push(package_path);

    let dev_server_path = web_dir.join("dev-server.mjs");
    fs::write(&dev_server_path, web_dev_server_template())?;
    files.push(dev_server_path);

    let readme_path = web_dir.join("README.md");
    fs::write(&readme_path, web_shell_readme_template(options))?;
    files.push(readme_path);

    Ok(())
}

fn web_index_template(options: &GenerateOptions) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{title}</title>
    <style>
      html,
      body,
      #{root_id} {{
        width: 100%;
        height: 100%;
        margin: 0;
      }}

      body {{
        overflow: hidden;
        background: #ffffff;
        color: #111111;
        font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }}

      #{root_id} {{
        position: relative;
        overflow: hidden;
      }}
    </style>
  </head>
  <body>
    <main id="{root_id}"></main>
    <script type="module" src="./main.js"></script>
  </body>
</html>
"#,
        title = html_escape(&options.web_title),
        root_id = html_escape(&options.web_root_id),
    )
}

fn web_main_template(options: &GenerateOptions) -> String {
    format!(
        r#"import {{ createRaxonWebHost }} from "./raxon-web-host.js";

const root = document.getElementById("{root_id}");
if (!root) {{
  throw new Error("Missing raxon mount element #{root_id}");
}}

function readViewport() {{
  const rect = root.getBoundingClientRect();
  return {{
    width: Math.max(1, rect.width || window.innerWidth || 1),
    height: Math.max(1, rect.height || window.innerHeight || 1),
  }};
}}

const host = await createRaxonWebHost(root, {{
  mount: false,
  wasmUrl: undefined,
  onBridgeError(error) {{
    console.error("[raxon] bridge error", error);
  }},
  handlePlatformRequest(request) {{
    console.warn("[raxon] unhandled platform request", request);
  }},
}});

const initial = readViewport();
host.mount(initial.width, initial.height);

let lastWidth = initial.width;
let lastHeight = initial.height;
function resizeIfNeeded() {{
  const next = readViewport();
  if (next.width !== lastWidth || next.height !== lastHeight) {{
    lastWidth = next.width;
    lastHeight = next.height;
    host.resize(next.width, next.height);
  }}
}}

if ("ResizeObserver" in window) {{
  const observer = new ResizeObserver(resizeIfNeeded);
  observer.observe(root);
}} else {{
  window.addEventListener("resize", resizeIfNeeded);
}}

let running = true;
function frame() {{
  if (!running) return;
  resizeIfNeeded();
  host.tick();
  window.requestAnimationFrame(frame);
}}
window.requestAnimationFrame(frame);

window.addEventListener("beforeunload", () => {{
  running = false;
  if (host.handle) host.destroy();
}});
"#,
        root_id = js_string_escape(&options.web_root_id),
    )
}

fn web_package_json_template(options: &GenerateOptions) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "node ./dev-server.mjs",
    "preview": "node ./dev-server.mjs"
  }}
}}
"#,
        name = json_escape(&package_name_slug(&options.web_title)),
    )
}

fn web_dev_server_template() -> String {
    r#"import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, join, normalize, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = resolve(fileURLToPath(new URL(".", import.meta.url)));
const host = process.env.HOST ?? "127.0.0.1";
const port = Number.parseInt(process.env.PORT ?? "5173", 10);

const contentTypes = new Map([
  [".css", "text/css; charset=utf-8"],
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
  [".map", "application/json; charset=utf-8"],
  [".mjs", "text/javascript; charset=utf-8"],
  [".wasm", "application/wasm"],
]);

function resolveRequestPath(rawUrl) {
  const requestUrl = new URL(rawUrl ?? "/", `http://${host}:${port}`);
  const pathname = decodeURIComponent(requestUrl.pathname);
  const relativePath = pathname === "/" ? "index.html" : pathname.replace(/^\/+/, "");
  const filePath = resolve(rootDir, normalize(relativePath));
  if (filePath !== rootDir && !filePath.startsWith(`${rootDir}${sep}`)) {
    return null;
  }
  return filePath;
}

async function sendFile(req, res) {
  if (req.method !== "GET" && req.method !== "HEAD") {
    res.writeHead(405, { Allow: "GET, HEAD" });
    res.end("Method Not Allowed");
    return;
  }

  let filePath;
  try {
    filePath = resolveRequestPath(req.url);
  } catch {
    res.writeHead(400, { "Content-Type": "text/plain; charset=utf-8" });
    res.end("Bad Request");
    return;
  }

  if (!filePath) {
    res.writeHead(403, { "Content-Type": "text/plain; charset=utf-8" });
    res.end("Forbidden");
    return;
  }

  try {
    let info = await stat(filePath);
    if (info.isDirectory()) {
      filePath = join(filePath, "index.html");
      info = await stat(filePath);
    }
    if (!info.isFile()) {
      res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
      res.end("Not Found");
      return;
    }

    res.writeHead(200, {
      "Content-Length": info.size,
      "Content-Type": contentTypes.get(extname(filePath)) ?? "application/octet-stream",
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    });

    if (req.method === "HEAD") {
      res.end();
      return;
    }

    const stream = createReadStream(filePath);
    stream.on("error", (error) => {
      if (!res.headersSent) {
        res.writeHead(500, { "Content-Type": "text/plain; charset=utf-8" });
      }
      res.end(error.message);
    });
    stream.pipe(res);
  } catch (error) {
    if (error && error.code === "ENOENT") {
      res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
      res.end("Not Found");
      return;
    }
    res.writeHead(500, { "Content-Type": "text/plain; charset=utf-8" });
    res.end(error?.message ?? "Internal Server Error");
  }
}

createServer(sendFile).listen(port, host, () => {
  console.log(`raxon web host: http://${host}:${port}/`);
});
"#
    .to_string()
}

fn web_shell_readme_template(options: &GenerateOptions) -> String {
    format!(
        r#"# raxon Web Host Shell

Generated by `rax generate --target web`.

## Files

- `raxon_web_bridge.rs`: Rust wasm bridge module for your app crate.
- `raxon-web-host.js`: browser host runtime that applies DOM command batches and emits raxon wire events.
- `raxon-web-host.d.ts`: TypeScript declarations for custom host integration.
- `index.html` and `main.js`: static browser shell that mounts `{wasm_module}`, resizes with `ResizeObserver`, and ticks with `requestAnimationFrame`.
- `package.json` and `dev-server.mjs`: no-dependency Node dev server with wasm MIME and cross-origin isolation headers.

## Rust side

Include `raxon_web_bridge.rs` from your app crate and expose the generated wasm
module at `{wasm_module}`. The default shell expects a wasm-bindgen-style module
whose default export initializes the `.wasm` and whose named exports include the
`raxon_web_*` bridge functions.

## Browser side

Run `npm run dev` from this directory and open the printed local URL. Customize
`main.js` to handle platform requests such as notifications, media picker,
clipboard, and accessibility announcements for your app. Set `HOST` or `PORT`
to override the dev-server bind address.
"#,
        wasm_module = options.wasm_module,
    )
}

fn binding_manifest_template(options: &GenerateOptions, files: &[PathBuf]) -> String {
    let file_list = files
        .iter()
        .map(|path| {
            let rel = path.strip_prefix(&options.out_dir).unwrap_or(path);
            format!("    \"{}\"", json_escape(&rel.display().to_string()))
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        r#"{{
  "tool": "raxon-cli",
  "target": "{}",
  "hostShells": {},
  "bridgeProtocolVersion": 1,
  "appFn": "{}",
  "android": {{
    "package": "{}",
    "class": "{}",
    "activity": "{}",
    "library": "{}"
  }},
  "web": {{
    "wasmModule": "{}",
    "title": "{}",
    "rootId": "{}"
  }},
  "files": [
{}
  ]
}}
"#,
        options.target.as_str(),
        options.host_shells,
        json_escape(&options.app_fn),
        json_escape(&options.android_package),
        json_escape(&options.android_class),
        json_escape(&options.android_activity),
        json_escape(&options.android_library),
        json_escape(&options.wasm_module),
        json_escape(&options.web_title),
        json_escape(&options.web_root_id),
        file_list
    )
}

// ---------------------------------------------------------------------------
// create_project
// ---------------------------------------------------------------------------

fn create_project(name: &str) {
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("Error: directory '{}' already exists", name);
        process::exit(1);
    }

    println!("Creating rax project '{}'...", name);

    // Create directory structure
    fs::create_dir_all(dir.join("src")).expect("Failed to create src/");

    // Write Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
name = "{lib_name}"
crate-type = ["staticlib", "cdylib"]

[dependencies]
raxon = "0.0.9"
"#,
        name = name,
        lib_name = name.replace('-', "_"),
    );
    fs::write(dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // Write src/lib.rs
    let lib_rs = r#"use raxon::prelude::*;

pub fn app() -> impl View {
    let count = create_signal(0);

    column((
        text("Hello from raxon!")
            .font_size(24.0)
            .color(Color::rgb(26, 26, 26)),
        text("Build native apps in Rust.")
            .font_size(16.0)
            .color(Color::rgba(0, 0, 0, 153)),
        button("Tap me", move || count.update(|n| *n += 1)),
        dynamic(move || {
            text(format!("Tapped {} times", count.get()))
                .font_size(14.0)
                .color(Color::rgb(51, 128, 255))
        }),
    ))
    .padding(32.0)
    .gap(16.0)
    .align(AlignItems::Center)
}

#[no_mangle]
pub extern "C" fn rax_main() {
    raxon::run(app);
}
"#;
    fs::write(dir.join("src").join("lib.rs"), lib_rs).expect("Failed to write src/lib.rs");

    // Write .gitignore
    fs::write(dir.join(".gitignore"), "/target\n").expect("Failed to write .gitignore");

    println!("Created '{}'", name);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  rax doctor");
    println!("  rax build --target ios-sim");
    println!("  rax generate --target all");
    println!("  rax build --target android");
    println!("  rax build --target web");
    println!();
    println!("To run, use the native host project for the platform you are targeting.");
}

// ---------------------------------------------------------------------------
// test
// ---------------------------------------------------------------------------

fn cmd_test(extra_args: &[String]) {
    println!("Running: cargo test {}", extra_args.join(" "));
    println!();
    println!("For iOS integration tests, run on a simulator:");
    println!("  RUSTC=<path> cargo test --target aarch64-apple-ios-sim");
    println!();
    println!("rax includes a built-in test harness via rax-test:");
    println!("  • Unit tests: use #[test] as normal");
    println!("  • Widget tests: use raxon_test::render() + finders");
    println!();

    let status = std::process::Command::new("cargo")
        .arg("test")
        .args(extra_args)
        .status()
        .expect("failed to run cargo test");
    std::process::exit(status.code().unwrap_or(1));
}

// ---------------------------------------------------------------------------
// lint
// ---------------------------------------------------------------------------

fn cmd_lint() {
    println!("Running: cargo clippy --all-targets");
    let status = std::process::Command::new("cargo")
        .args(["clippy", "--all-targets"])
        .status()
        .expect("failed to run cargo clippy");
    std::process::exit(status.code().unwrap_or(1));
}

// ---------------------------------------------------------------------------
// fmt
// ---------------------------------------------------------------------------

fn cmd_fmt(check: bool) {
    let args = if check {
        vec!["fmt", "--check"]
    } else {
        vec!["fmt"]
    };
    println!("Running: cargo {}", args.join(" "));
    let status = std::process::Command::new("cargo")
        .args(&args)
        .status()
        .expect("failed to run cargo fmt");
    std::process::exit(status.code().unwrap_or(1));
}

// ---------------------------------------------------------------------------
// add
// ---------------------------------------------------------------------------

fn cmd_add(crate_name: &str) {
    println!("To add a dependency:");
    println!("  cargo add {crate_name}");
    println!();
    println!("For rax plugins, check: https://github.com/1homsi/rax");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_output_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is valid")
            .as_nanos();
        std::env::temp_dir().join(format!("raxon-cli-{name}-{stamp}"))
    }

    #[test]
    fn parses_generate_options() {
        let args = vec![
            "rax".to_string(),
            "generate".to_string(),
            "--target".to_string(),
            "web".to_string(),
            "--out".to_string(),
            "bindings".to_string(),
            "--app-fn".to_string(),
            "crate::ui::app".to_string(),
            "--android-package".to_string(),
            "dev.raxon.demo".to_string(),
            "--android-class".to_string(),
            "DemoHost".to_string(),
            "--android-activity".to_string(),
            "DemoActivity".to_string(),
            "--android-library".to_string(),
            "demo_lib".to_string(),
            "--wasm-module".to_string(),
            "./pkg/demo.js".to_string(),
            "--web-title".to_string(),
            "Demo App".to_string(),
            "--web-root-id".to_string(),
            "demo_root".to_string(),
            "--glue-only".to_string(),
        ];

        let options = parse_generate_options(&args).expect("options parse");

        assert_eq!(options.target, GenerateTarget::Web);
        assert_eq!(options.out_dir, PathBuf::from("bindings"));
        assert_eq!(options.app_fn, "crate::ui::app");
        assert_eq!(options.android_package, "dev.raxon.demo");
        assert_eq!(options.android_class, "DemoHost");
        assert_eq!(options.android_activity, "DemoActivity");
        assert_eq!(options.android_library, "demo_lib");
        assert_eq!(options.wasm_module, "./pkg/demo.js");
        assert_eq!(options.web_title, "Demo App");
        assert_eq!(options.web_root_id, "demo_root");
        assert!(!options.host_shells);
    }

    #[test]
    fn rejects_invalid_generate_target() {
        let args = vec![
            "rax".to_string(),
            "generate".to_string(),
            "--target".to_string(),
            "desktop".to_string(),
        ];

        assert!(parse_generate_options(&args).is_err());
    }

    #[test]
    fn jni_function_prefix_escapes_underscores() {
        assert_eq!(
            jni_function_prefix("dev.raxon_demo", "Demo_Host"),
            "Java_dev_raxon_1demo_Demo_1Host"
        );
    }

    #[test]
    fn generate_all_writes_android_web_and_manifest_files() {
        let out_dir = temp_output_dir("all");
        let options = GenerateOptions {
            out_dir: out_dir.clone(),
            app_fn: "crate::ui::app".to_string(),
            android_package: "dev.raxon.demo".to_string(),
            android_class: "DemoHost".to_string(),
            android_activity: "DemoActivity".to_string(),
            android_library: "demo_lib".to_string(),
            wasm_module: "./pkg/demo.js".to_string(),
            web_title: "Demo App".to_string(),
            web_root_id: "demo_root".to_string(),
            ..GenerateOptions::default()
        };

        let files = generate_bindings(&options).expect("bindings generate");

        assert_eq!(files.len(), 20);
        let android_rust =
            fs::read_to_string(out_dir.join("android/raxon_android_bridge.rs")).unwrap();
        assert!(android_rust.contains("Java_dev_raxon_demo_DemoHost_nativeMount"));
        assert!(android_rust.contains("mount_android(raxon::core::Size::new"));
        assert!(android_rust.contains("crate::ui::app"));

        let kotlin = fs::read_to_string(
            out_dir.join("android/app/src/main/java/dev/raxon/demo/DemoHost.kt"),
        )
        .unwrap();
        assert!(kotlin.contains("package dev.raxon.demo"));
        assert!(kotlin.contains("class DemoHost"));
        assert!(kotlin.contains("nativeHandleRequest"));
        assert!(kotlin.contains("applyCommandBatch"));
        assert!(kotlin.contains("command.getString(\"class_name\")"));
        assert!(kotlin.contains("installBuiltInListeners"));
        assert!(kotlin.contains("installGesture"));
        assert!(kotlin.contains("type\", \"text_changed\""));
        assert!(kotlin.contains("commandHandler(command)"));

        let activity = fs::read_to_string(
            out_dir.join("android/app/src/main/java/dev/raxon/demo/DemoActivity.kt"),
        )
        .unwrap();
        assert!(activity.contains("open class DemoActivity : Activity()"));
        assert!(activity.contains("Choreographer.getInstance().postFrameCallback"));
        assert!(activity.contains("DemoHost.loadLibrary(NATIVE_LIBRARY)"));
        assert!(activity.contains("const val NATIVE_LIBRARY: String = \"demo_lib\""));

        let manifest =
            fs::read_to_string(out_dir.join("android/app/src/main/AndroidManifest.xml")).unwrap();
        assert!(manifest.contains("android:name=\"dev.raxon.demo.DemoActivity\""));

        let settings = fs::read_to_string(out_dir.join("android/settings.gradle.kts")).unwrap();
        assert!(settings.contains("rootProject.name = \"Demo App\""));
        assert!(settings.contains("include(\":app\")"));

        let root_build = fs::read_to_string(out_dir.join("android/build.gradle.kts")).unwrap();
        assert!(root_build.contains("com.android.application"));
        assert!(root_build.contains(ANDROID_GRADLE_PLUGIN_VERSION));

        let app_build = fs::read_to_string(out_dir.join("android/app/build.gradle.kts")).unwrap();
        assert!(app_build.contains("namespace = \"dev.raxon.demo\""));
        assert!(app_build.contains(&format!("compileSdk = {ANDROID_COMPILE_SDK}")));
        assert!(app_build.contains(&format!("minSdk = {ANDROID_MIN_SDK}")));
        assert!(app_build.contains(&format!("targetSdk = {ANDROID_TARGET_SDK}")));
        assert!(app_build.contains("jniLibs.srcDir(\"src/main/jniLibs\")"));

        let wrapper =
            fs::read_to_string(out_dir.join("android/gradle/wrapper/gradle-wrapper.properties"))
                .unwrap();
        assert!(wrapper.contains(&format!("gradle-{GRADLE_WRAPPER_VERSION}-bin.zip")));

        let android_readme = fs::read_to_string(out_dir.join("android/README.md")).unwrap();
        assert!(android_readme.contains("./gradlew :app:assembleDebug"));
        assert!(android_readme.contains("app/src/main/jniLibs/<abi>/libdemo_lib.so"));

        let web_rust = fs::read_to_string(out_dir.join("web/raxon_web_bridge.rs")).unwrap();
        assert!(web_rust.contains("raxon_web_handle_request"));
        assert!(web_rust.contains("mount_web(raxon::core::Size::new"));

        let web_js = fs::read_to_string(out_dir.join("web/raxon-web-host.js")).unwrap();
        assert!(web_js.contains("await import(\"./pkg/demo.js\")"));
        assert!(web_js.contains("dispatchEvents(events)"));
        assert!(web_js.contains("applyCommand(command)"));
        assert!(web_js.contains("command.tag_name"));
        assert!(web_js.contains("command.css_color"));
        assert!(web_js.contains("installBuiltInListeners"));
        assert!(web_js.contains("type: \"text_changed\""));
        assert!(web_js.contains("node.style.color = attr.value"));
        assert!(web_js.contains("handlePlatformRequest(command.request ?? command)"));

        let web_index = fs::read_to_string(out_dir.join("web/index.html")).unwrap();
        assert!(web_index.contains("<title>Demo App</title>"));
        assert!(web_index.contains("id=\"demo_root\""));

        let web_main = fs::read_to_string(out_dir.join("web/main.js")).unwrap();
        assert!(web_main.contains("createRaxonWebHost(root"));
        assert!(web_main.contains("ResizeObserver"));
        assert!(web_main.contains("window.requestAnimationFrame(frame)"));

        let package_json = fs::read_to_string(out_dir.join("web/package.json")).unwrap();
        assert!(package_json.contains("\"name\": \"demo-app\""));
        assert!(package_json.contains("\"dev\": \"node ./dev-server.mjs\""));

        let dev_server = fs::read_to_string(out_dir.join("web/dev-server.mjs")).unwrap();
        assert!(dev_server.contains("createServer(sendFile)"));
        assert!(dev_server.contains("\"application/wasm\""));
        assert!(dev_server.contains("\"Cross-Origin-Embedder-Policy\""));

        let manifest = fs::read_to_string(out_dir.join("raxon-bindings.json")).unwrap();
        assert!(manifest.contains("\"target\": \"all\""));
        assert!(manifest.contains("\"hostShells\": true"));
        assert!(manifest.contains("\"bridgeProtocolVersion\": 1"));
        assert!(manifest.contains("\"android/raxon_android_bridge.rs\""));
        assert!(manifest.contains("\"android/app/src/main/java/dev/raxon/demo/DemoActivity.kt\""));
        assert!(manifest.contains("\"android/app/build.gradle.kts\""));
        assert!(manifest.contains("\"web/index.html\""));
        assert!(manifest.contains("\"web/package.json\""));

        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn generate_web_only_writes_browser_shell_and_skips_android_files() {
        let out_dir = temp_output_dir("web");
        let options = GenerateOptions {
            target: GenerateTarget::Web,
            out_dir: out_dir.clone(),
            ..GenerateOptions::default()
        };

        let files = generate_bindings(&options).expect("bindings generate");

        assert_eq!(files.len(), 9);
        assert!(out_dir.join("web/raxon-web-host.js").exists());
        assert!(out_dir.join("web/index.html").exists());
        assert!(out_dir.join("web/main.js").exists());
        assert!(out_dir.join("web/package.json").exists());
        assert!(out_dir.join("web/dev-server.mjs").exists());
        assert!(!out_dir.join("android").exists());

        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn generate_glue_only_skips_host_shell_files() {
        let out_dir = temp_output_dir("glue");
        let options = GenerateOptions {
            target: GenerateTarget::Web,
            out_dir: out_dir.clone(),
            host_shells: false,
            ..GenerateOptions::default()
        };

        let files = generate_bindings(&options).expect("bindings generate");

        assert_eq!(files.len(), 4);
        assert!(out_dir.join("web/raxon-web-host.js").exists());
        assert!(!out_dir.join("web/index.html").exists());
        let manifest = fs::read_to_string(out_dir.join("raxon-bindings.json")).unwrap();
        assert!(manifest.contains("\"hostShells\": false"));

        let _ = fs::remove_dir_all(out_dir);
    }
}
