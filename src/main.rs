use clap::Parser;
use include_dir::{Dir, include_dir};
use std::env::current_dir;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

// static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");
static PAGES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/pages");
static CARGO_FILE: &str = include_str!("../pages/Cargo.toml");

#[derive(Debug, Parser)]
#[command(name = "pages")]
#[command(version, about = "pages-rs 命令行构建工具")]
enum Cli {
    /// 创建一个新的 pages-rs 项目在 <path> 路径下
    New {
        /// 项目路径和默认项目名称
        #[arg(required = true)]
        path: String,
        /// 透传给 cargo new 的所有后续参数
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// 打包当前项目 (仅支持 Linux 环境与 Zig 交叉编译)
    Pack {
        /// 要打包的目标项目路径 (默认为当前目录 `.`)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() {
    // 🔥 核心修改：直接解析顶层命令
    let cli = Cli::parse();

    match cli {
        Cli::New { path, args } => {
            if let Err(e) = handle_new(&path, args) {
                eprintln!("❌ 创建项目失败: {}", e);
                std::process::exit(1);
            }
        }
        Cli::Pack { path } => {
            if let Err(e) = handle_pack(&path) {
                eprintln!("❌ 打包失败: {}", e);
                std::process::exit(1);
            }
        }
    }
}

// ==================== 1. NEW 子命令处理 ====================
fn handle_new(path: &str, args: Vec<String>) -> io::Result<()> {
    println!("🚀 正在调用原始构建工具创建项目...");

    // 1. 无缝衔接后续参数并执行（这里默认透传给 cargo new）
    let command = Command::new("cargo")
        .arg("new")
        .arg(path)
        .args(&args)
        .status()?;
    if !command.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "底层项目创建指令执行失败",
        ));
    }

    let path = Path::new(path);

    // 2. 切换到项目目录下运行 cargo add pages-rs
    println!("📦 正在自动添加 pages-rs 依赖项...");
    let command = Command::new("cargo")
        .arg("add")
        .arg("pages-rs")
        .current_dir(path)
        .status()?;
    if !command.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "添加 pages-rs 依赖失败",
        ));
    }

    // 3. 替换创建项目后的 lib.rs 和 main.rs 为自定义内容
    println!("📝 正在注入自定义 main.rs 和 lib.rs 模板...");
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir)?;

    // if let Some(file) = TEMPLATES_DIR.get_file("main.rs") {
    //     fs::write(src_dir.join("main.rs"), file.contents())?;
    // }
    // if let Some(file) = TEMPLATES_DIR.get_file("lib.rs") {
    //     fs::write(src_dir.join("lib.rs"), file.contents())?;
    // }

    println!("✨ 项目初始化成功！");
    Ok(())
}

// ==================== 2. PACK 子命令处理 ====================
fn handle_pack(path: &Path) -> io::Result<()> {
    // 💥 第一：判断当前系统是否为 Linux
    if std::env::consts::OS != "linux" {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "当前系统为「{}」，本命令仅支持在 Linux 系统下运行构建",
                std::env::consts::OS
            ),
        ));
    }

    let package = match std::fs::read_to_string(path.join("Cargo.toml")) {
        Ok(content) => match content.parse::<toml_edit::DocumentMut>() {
            Ok(document) => match document["package"]["name"].as_str() {
                Some(package) => package.to_string(),
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Cargo.toml 文件中未找到 package.name 字段",
                    ));
                }
            },
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cargo.toml 文件解析失败: {}", e),
                ));
            }
        },
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "未找到 Cargo.toml 文件，请确保在 {} 目录下运行",
                    path.display()
                ),
            ));
        }
    };

    // 💥 第二：判断当前系统是否有 zig 构建环境
    println!("🔍 检查 Zig 环境...");
    let zig_check = Command::new("zig").arg("version").output();
    if zig_check.is_err() || !zig_check.unwrap().status.success() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "未检测到 zig 构建环境。请先安装 Zig (https://ziglang.org/ download/)",
        ));
    }

    // 💥 第三：判断并安装 cargo-zigbuild
    println!("🔍 检查 cargo-zigbuild 工具...");
    let zigbuild_check = Command::new("cargo-zigbuild").arg("--version").output();
    if zigbuild_check.is_err() || !zigbuild_check.unwrap().status.success() {
        println!("🛠️ 未检测到 cargo-zigbuild，正在通过 cargo install 安装...");
        let install_status = Command::new("cargo")
            .arg("install")
            .arg("cargo-zigbuild")
            .status()?;
        if !install_status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "自动安装 cargo-zigbuild 失败，请手动安装",
            ));
        }
    }

    // 确定工作及输出路径
    let target_pages_dir = Path::new("./target/pages");
    let edgeone_dir = target_pages_dir.join("edgeone").join("nodejs-stream");
    // let pages_sub_dir = target_pages_dir.join("pages");

    // 清理并重新创建构建目录
    if target_pages_dir.exists() {
        fs::remove_dir_all(target_pages_dir)?;
    }
    fs::create_dir_all(&target_pages_dir)?;

    // 💥 第四：释放 include_dir 包含的 pages 文件夹所有内容
    println!("📂 正在释放内置核心 pages 模块到构建目录...");
    extract_dir(&PAGES_DIR, target_pages_dir)?;

    let mut workspace = CARGO_FILE.parse::<toml_edit::DocumentMut>().unwrap();

    let pages = &mut workspace["workspace"]["dependencies"]["pages"];
    pages["path"] = toml_edit::value(current_dir()?.join(path).display().to_string());
    pages["package"] = toml_edit::value(package);

    std::fs::write(target_pages_dir.join("Cargo.toml"), workspace.to_string())?;

    // 💥 第五：运行 cargo zigbuild 构建动态库
    println!("🏗️ 正在使用 zigbuild 进行 x86_64 跨平台 Linux 编译...");
    let build_status = Command::new("cargo")
        .arg("zigbuild")
        .arg("--release")
        .arg("--lib")
        .arg("--manifest-path")
        .arg("./target/pages/Cargo.toml")
        .arg("--target")
        .arg("x86_64-unknown-linux-gnu.2.17")
        .status()?;

    if !build_status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "cargo-zigbuild 编译流程出错",
        ));
    }

    // 💥 第六：复制编译好的动态库并改名
    println!("🚚 正在同步动态库 (.so) 扩展至目标分发目录...");
    let edgeone_so = target_pages_dir
        .join("target")
        .join("x86_64-unknown-linux-gnu")
        .join("release")
        .join("libnodejs_stream.so");

    let edgeone_node = edgeone_dir.join("pages.node");
    fs::copy(&edgeone_so, &edgeone_node)?;

    // 💥 第七：打包文件为 zip 包
    let edgeone_zip = target_pages_dir.join("edgeone.zip");
    println!("🗜️ 正在将生产制品打包为密闭 ZIP 归档: {:?}", edgeone_zip);
    zip_directory(&edgeone_dir, &edgeone_zip)?;

    println!("🎉 完美的自动化构建完成！制品已生成：{:?}", edgeone_zip);
    Ok(())
}

// ==================== 辅助工具函数 ====================

/// 递归释放 include_dir 资源到真实文件系统
fn extract_dir(dir: &Dir, base_path: &Path) -> io::Result<()> {
    for file in dir.files() {
        let out_path = base_path.join(file.path());
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(out_path, file.contents())?;
    }
    for child_dir in dir.dirs() {
        extract_dir(child_dir, base_path)?;
    }
    Ok(())
}

/// 将指定目录打包为 Zip 归档
fn zip_directory(src_dir: &Path, dst_zip: &Path) -> io::Result<()> {
    let file = File::create(dst_zip)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    for entry in it.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path
            .strip_prefix(src_dir)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
