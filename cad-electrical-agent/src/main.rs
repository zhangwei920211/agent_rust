//! acad-mcp CLI 入口（M1 Task1）。Serve 与部分 CliOp 在后续任务实现。
//! 面向用户输出中文，代码标识符英文（plan Global Constraints）。

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "acad-mcp", about = "CAD 电气深化设计 MCP server")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 以 MCP stdio server 运行（M1 Task5 实现）
    Serve,
    /// 本地直调（无 LLM 测试用）
    Cli {
        #[command(subcommand)]
        op: CliOp,
    },
}

#[derive(Subcommand)]
enum CliOp {
    /// 统计图纸：图层/块/实体类型/INSERT 引用计数
    Summary { path: PathBuf },
    /// 列出图层：名称/开关/冻结/可打印/颜色
    Layers { path: PathBuf },
    /// 按正则过滤块插入计数（Task4/5 实现）
    Blocks { path: PathBuf, filter: Option<String> },
    /// 生成机电设备资产登记表 xlsx（Task4 实现）
    Register { path: PathBuf, out_xlsx: PathBuf },
    /// 读取 docx/xlsx/pdf 为 Markdown（Task6 实现）
    Read { path: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Cmd::Serve => {
            eprintln!("MCP stdio server 将在 M1 Task5 实现");
            2
        }
        Cmd::Cli { op } => match op {
            CliOp::Summary { path } => match acad_mcp::drawing::summarize(&path) {
                Ok(stats) => {
                    println!("{}", serde_json::to_string_pretty(&stats).unwrap());
                    0
                }
                Err(e) => {
                    eprintln!("错误：{e:#}");
                    1
                }
            },
            CliOp::Layers { path } => match acad_mcp::drawing::summarize(&path) {
                Ok(stats) => {
                    println!("{}", serde_json::to_string_pretty(&stats.layers).unwrap());
                    0
                }
                Err(e) => {
                    eprintln!("错误：{e:#}");
                    1
                }
            },
            CliOp::Register { path, out_xlsx } => match acad_mcp::drawing::summarize(&path) {
                Ok(stats) => {
                    let cfg = acad_mcp::asset::default_config();
                    let rows = acad_mcp::asset::build_register(&stats, &cfg);
                    match acad_mcp::asset::write_xlsx(&rows, &out_xlsx) {
                        Ok(()) => {
                            eprintln!("已生成资产登记表：{}（{} 行）", out_xlsx.display(), rows.len());
                            0
                        }
                        Err(e) => {
                            eprintln!("错误：{e:#}");
                            1
                        }
                    }
                }
                Err(e) => {
                    eprintln!("错误：{e:#}");
                    1
                }
            },
            CliOp::Blocks { .. } | CliOp::Read { .. } => {
                eprintln!("该功能将在后续任务实现");
                2
            }
        },
    };
    std::process::exit(code);
}
