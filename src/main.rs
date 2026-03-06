use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use terminal_size::{Width, terminal_size};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
struct Lang {
    name: &'static str,
    color: (u8, u8, u8),
    bytes: usize,
    lines: usize,
}

impl Lang {
    fn percent(&self, total: usize) -> f32 {
        if total == 0 {
            return 0.0;
        }
        (self.bytes as f32 / total as f32) * 100.0
    }
}

fn get_language_info(ext: &str) -> Option<(&'static str, (u8, u8, u8))> {
    match ext {
        "rs" => Some(("Rust", (222, 165, 132))),
        "py" => Some(("Python", (53, 114, 165))),
        "js" => Some(("JavaScript", (241, 224, 90))),
        "ts" => Some(("TypeScript", (49, 120, 198))),
        "tsx" => Some(("TypeScript", (49, 120, 198))),
        "jsx" => Some(("JavaScript", (241, 224, 90))),
        "html" => Some(("HTML", (227, 76, 38))),
        "css" => Some(("CSS", (86, 61, 124))),
        "go" => Some(("Go", (0, 173, 216))),
        "java" => Some(("Java", (176, 114, 25))),
        "cpp" | "cc" | "cxx" => Some(("C++", (243, 75, 125))),
        "c" => Some(("C", (85, 85, 85))),
        "h" | "hpp" => Some(("C/C++ Header", (85, 85, 85))),
        "rb" => Some(("Ruby", (204, 52, 45))),
        "php" => Some(("PHP", (79, 93, 149))),
        "swift" => Some(("Swift", (240, 81, 56))),
        "kt" | "kts" => Some(("Kotlin", (167, 139, 250))),
        "sh" | "bash" => Some(("Shell", (137, 224, 81))),
        "vue" => Some(("Vue", (65, 184, 131))),
        "svelte" => Some(("Svelte", (255, 62, 0))),
        // "sql" => Some(("SQL", (228, 77, 38))),
        // GitHub無視ファイル（toml, json, md, yaml, xmlなど）は除外
        _ => None,
    }
}

fn count_bytes(path: &Path) -> std::io::Result<usize> {
    let content = fs::read(path)?;
    Ok(content.len())
}

fn count_lines(path: &Path) -> std::io::Result<usize> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().count())
}

fn should_ignore(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let ignore_dirs = [
        "node_modules",
        "target",
        "dist",
        "build",
        ".git",
        "vendor",
        "__pycache__",
        ".venv",
        "venv",
    ];

    ignore_dirs.iter().any(|&dir| path_str.contains(dir))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let target_dir = if args.len() > 1 { &args[1] } else { "." };

    println!("\nAnalyzing project: {}\n", target_dir.bright_cyan());

    let mut lang_stats: HashMap<&'static str, Lang> = HashMap::new();

    for entry in WalkDir::new(target_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if should_ignore(path) {
            continue;
        }

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if let Some((lang_name, color)) = get_language_info(ext_str) {
                        if let Ok(bytes) = count_bytes(path) {
                            if let Ok(lines) = count_lines(path) {
                                let lang = lang_stats.entry(lang_name).or_insert(Lang {
                                    name: lang_name,
                                    color,
                                    bytes: 0,
                                    lines: 0,
                                });
                                lang.bytes += bytes;
                                lang.lines += lines;
                            }
                        }
                    }
                }
            }
        }
    }

    if lang_stats.is_empty() {
        println!("⚠️  No supported language files found in this directory.");
        return;
    }

    // ソート: バイト数の多い順
    let mut langs: Vec<Lang> = lang_stats.values().cloned().collect();
    langs.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    let total_bytes: usize = langs.iter().map(|l| l.bytes).sum();

    // ターミナルの横幅を取得（デフォルトは80）
    let bar_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    // 上段: プログレスバー
    let mut total_cols = 0;
    let mut bar_segments: Vec<(usize, (u8, u8, u8))> = Vec::new();

    // 各言語の幅を計算
    for (i, lang) in langs.iter().enumerate() {
        let percent = lang.percent(total_bytes);
        let cols = if i == langs.len() - 1 {
            // 最後の言語は残りの幅を使う（丸め誤差を吸収）
            bar_width.saturating_sub(total_cols)
        } else {
            ((percent / 100.0) * bar_width as f32).round() as usize
        };
        total_cols += cols;
        bar_segments.push((cols, lang.color));
    }

    // バーを描画
    for (cols, color) in bar_segments {
        for _ in 0..cols {
            print!("{}", "■".truecolor(color.0, color.1, color.2));
        }
    }
    println!();

    // 下段: 言語リスト
    for lang in &langs {
        let percent = lang.percent(total_bytes);
        print!(
            "{}",
            "●".truecolor(lang.color.0, lang.color.1, lang.color.2)
        );
        print!(" {}", lang.name.white());
        let pad = 15usize.saturating_sub(lang.name.len());
        print!("{}", " ".repeat(pad));
        print!("{}", format!("{:>6.1}%", percent).bright_black());
        println!(
            "  {}",
            format!("({} lines)", lang.lines).bright_black().dimmed()
        );
    }

    let total_lines: usize = langs.iter().map(|l| l.lines).sum();
    println!(
        "\n{}",
        format!("Total: {} lines", total_lines).bright_green()
    );
}
