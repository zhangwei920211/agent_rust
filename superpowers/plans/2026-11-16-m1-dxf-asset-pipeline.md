# M1:DXF 读图与资产登记管线(acad-mcp 骨架)实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立自研 Rust MCP server `acad-mcp`,完成 DXF 文件级读图线:图层/块/实体统计、按规则分类清点机电设备、生成资产登记表 xlsx;并暴露为 MCP 工具与本地 CLI。

**Architecture:** 单 crate `acad-mcp`(lib + bin),核心模型 `drawing.rs`(DXF 解析→统计)与 `asset.rs`(规则分类→xlsx),经 `rmcp`(官方 Rust MCP SDK,stdio)暴露 5 个工具;同一批函数经 clap `cli` 子命令直接调用以便无 LLM 测试。DWG→DXF 转换由外部 LibreDWG `dwg2dxf` 脚本完成(不链接进二进制,规避 GPL 传染)。COM 线与规范网关属 M2/M4,本计划不含。

**Tech Stack:** Rust stable;`dxf` 0.6(DXF 解析)、`rmcp` 3.10(MCP server;features `transport-io` + `schemars`,server/macros 默认开启)、`rust_xlsxwriter` 0.102(xlsx 生成)、`sha2` 0.10(源文件哈希复核)、`clap` 4、`serde`(derive)/`serde_json`、`schemars` 1、`anyhow` 1、`regex` 1、`thiserror` 2、`tracing` 0.1、`tokio` 1(rt-multi-thread/macros/io-util);dev-deps:`calamine` 0.38(xlsx 读回断言)、`tempfile` 3、`tokio`(test rt)、`rmcp` 3.10(features `client` + `transport-async-rw`,进程内 in-memory 测试)。

**Spec:** `superpowers/specs/2026-11-16-cad-electrical-agent-design.md`(相对仓库根 `D:\AgentRust`;M1 对应其里程碑 M1;设计文档 8.1 六问按本方默认建议执行,M1 不受其阻塞)

## Global Constraints

- 仅 Windows 目标;仓库根 `D:\AgentRust`(git init 于本计划 Task 1;`_recon/`、`dumps/`、`*.dxf` fixture 入 `.gitignore`)
- crate 目录 `D:\AgentRust\cad-electrical-agent\`;Cargo 包名 `acad-mcp` → lib crate 名 `acad_mcp`,bin 名 `acad-mcp`;workspace 单成员起步
- GPL 代码不得链接进 `acad-mcp`;LibreDWG 仅作外部命令行工具被脚本调用
- 大文件 fixture 不入库(`tests/fixtures/*.dxf` gitignore,脚本再生成);手写小 fixture 直接入库
- 所有面向用户的输出(错误信息、xlsx 表头、日志)使用中文;代码标识符英文
- 设计文档反目标:不在原图上动手——本计划全部只读 DXF,唯一写动作是生成 xlsx 到用户指定输出路径
- 样图基准:`D:\微信文件\xwechat_files\wxid_yzv5spffpwta22_f83f\msg\file\2026-03\雁能1#-电气施工图 - 无人防版.dwg`(实测 AC1018/2004、6.98MB;原写的 AC1027/约 10MB 带「2026.3.27电气修改」后缀路径本机不存在,2026-11-17 已纠正),路径含 `#`、空格与中文——所有文件处理必须任意 Windows 路径可用

## 与知识库对齐的硬约束(2026-11-17 取证自 `D:\pi`)

M1 不实现全部机制,但不得违反下列约束;范围外的条目标注为"前置契约"。

- **K1 规范网关三态(M2 前置契约)**:命中 / 未命中 / 不现行必须分开处置;未命中(尤其单源 openstd 零命中)不得直接判阻断,须转业主知识库或人工确认并留痕;过渡期标准双记录并存。依据:`0001-standards-anchors-governance-2026-09-26.md`、`gbt-16895-15-2023-low-voltage-wiring-systems.md`(后者 `verified: false`,2023 版在 openstd 检索为 0 条)。
- **K2 通道边界(M4 前置契约)**:批量层与 CUA 精细层严禁并发,单写者锁按图纸文档持有;`accoreconsole` 只用于纯 AutoCAD 命令,**天正依赖动作必须留在会话内(COM/LISP/SCRIPT)**。依据:`cad-channel-selection-batch-vs-cua-2026-11-02.md`(引 FDES:AcCoreConsole 跑不了垂直产品命令)。
- **K3 环境基线**:已验证组合为 **AutoCAD 2020 (R23.1) + 天正电气 V10**;AutoCAD 2024/2025 与 2014 均无库内实测证据,M1 不依赖任何 AutoCAD 版本。
- **K4 零自证**:凡"读取/计数正确"的结论必须异道核验(第二读取器或 AutoCAD 侧读数),`entity_total == 计数之和` 只能算内部自洽,不得单独作为验收依据。
- **K5 写路径护栏**:任何输出路径不得与输入相同或被其覆盖;写出前记录源 DXF 的 SHA-256,写出后复核。依据:`cad-readonly-self-audit-atomic-write-2026-11-10.md`(实测 `--out` 可覆盖源图)。
- **K6 选型留痕**:goose / rmcp / Rust / LibreDWG / anydoc 在库内无项目级评估记录,Task 7 必须落一条带 `unverified` 标记的决策记录,不得默认已对齐。

## Review Focus

1. **大体量 DXF**:雁能1# 转换后可能数十万实体——期望解析完成且计数正确。`dxf` 0.6 的 `Drawing::load_file` 会把整图载入内存(无流式 API),故验收口径为:单次遍历、`entity_total` 等于 `entity_counts` 之和、记录 DXF 文件大小与实测峰值内存;不再承诺"不加载全图"。
2. **中文编码**:LibreDWG 输出 DXF 中图层/块名为中文——期望 UTF-8 正确呈现。→ Task 3 断言统计结果中存在含 CJK 字符的块名;Task 2 手写 fixture 含中文块名。
3. **匿名/布局块混入设备清单**:`*U`、`*Model_Space`、`A$xxx`、`*D1` 不是设备。→ Task 2/4 断言以 `*` 或 `A$` 开头的块被排除。
4. **定义数与插入数混淆**:blocks(定义)与 insert_counts(引用)是两回事——期望清单用引用计数。→ Task 2 断言 `insert_counts["风机-1"]==3` 而 `blocks` 仅含定义名。
5. **任意 Windows 路径**:空格、中文、`#`、括号。→ Task 3 将 fixture 复制到含空格+中文+`#` 的临时路径再跑。
6. **零自证与写路径**:计数需第二读取器异道核验;输出 xlsx 不得与输入 DXF 同路径,且写后复核源 SHA-256。→ Task 3 原始组码复算、Task 5 `ops::register` 护栏。

---

### Task 1: 仓库与 crate 脚手架

**Files:**
- Create: `.gitignore`(仓库根)、`cad-electrical-agent/Cargo.toml`、`cad-electrical-agent/src/main.rs`、`cad-electrical-agent/src/lib.rs`、`cad-electrical-agent/src/{drawing,asset,server,ops}.rs`(空模块占位)
- Test: `cad-electrical-agent/tests/cli_test.rs`

**Interfaces:**
- Produces: lib crate `acad_mcp` 导出 `pub struct Cli`、`pub enum Cmd`、`pub enum CliOp`、`pub fn run(Cli) -> anyhow::Result<()>`,以及 `pub mod drawing; pub mod asset; pub mod server; pub mod ops;`(后四个模块在后续任务填充,本任务先建空模块占位并 `pub use`)
- bin `acad-mcp` 只做 `Cli::parse` + exit code 映射;**集成测试一律经 `acad_mcp::{Cli, run}` 访问**,不从 bin 访问

- [ ] **Step 1: git init 与 .gitignore**

仓库根 `D:\AgentRust` 执行 `git init`;`.gitignore` 内容:`_recon/`、`dumps/`、`cad-electrical-agent/tests/fixtures/*.dxf`、`target/`、`*.xlsx`(fixture 产物)。

- [ ] **Step 2: cargo 脚手架**

`cad-electrical-agent/Cargo.toml`——包名与目录名解耦,否则集成测试 `use acad_mcp::` 找不到 crate:

```toml
[package]
name = "acad-mcp"
version = "0.1.0"
edition = "2021"

[lib]
name = "acad_mcp"
path = "src/lib.rs"

[[bin]]
name = "acad-mcp"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "io-util"] }
rmcp = { version = "3.10", features = ["transport-io", "schemars"] }
acadrust = { version = "0.6", default-features = false }
rust_xlsxwriter = "0.102"
sha2 = "0.10"
regex = "1"
thiserror = "2"
tracing = "0.1"

[dev-dependencies]
tempfile = "3"
calamine = "0.38"
rmcp = { version = "3.10", features = ["client", "transport-async-rw"] }
```

`src/lib.rs` 定义 CLI 类型(测试与 bin 共用):

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "acad-mcp", about = "CAD 电气深化设计 MCP server")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// 以 MCP stdio server 运行
    Serve,
    /// 本地直调(无 LLM 测试用)
    Cli {
        #[command(subcommand)]
        op: CliOp,
    },
}

#[derive(Subcommand, Debug)]
pub enum CliOp {
    Summary { dxf_path: PathBuf },
    Layers { dxf_path: PathBuf },
    Blocks { dxf_path: PathBuf, filter: Option<String> },
    Register { dxf_path: PathBuf, out_xlsx: PathBuf },
    Read { path: PathBuf },
}

pub fn run(_cli: Cli) -> anyhow::Result<()> {
    anyhow::bail!("该功能将在后续任务实现")
}

pub mod asset;
pub mod drawing;
pub mod ops;
pub mod server;
```

`src/main.rs` 只做解析与退出码映射:

```rust
use clap::Parser;

fn main() -> std::process::ExitCode {
    let cli = acad_mcp::Cli::parse();
    match acad_mcp::run(cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            std::process::ExitCode::from(2)
        }
    }
}
```

本任务所有 `CliOp` 都经 `run` 返回 `"该功能将在后续任务实现"`(main 映射为 exit code 2)。

- [ ] **Step 3: 写失败测试**

```rust
// tests/cli_test.rs
use clap::Parser;

#[test]
fn cli_parse_summary() {
    let cli = acad_mcp::Cli::try_parse_from(["acad-mcp", "cli", "summary", "x.dxf"]);
    assert!(cli.is_ok());
}

#[test]
fn unimplemented_op_returns_error() {
    let cli = acad_mcp::Cli::parse_from(["acad-mcp", "cli", "summary", "x.dxf"]);
    let err = acad_mcp::run(cli).unwrap_err();
    assert!(err.to_string().contains("后续任务实现"));
}
```

- [ ] **Step 4:** `cargo test --test cli_test` → 全 PASS(实现已在 Step 2 给出)。
- [ ] **Step 5:** `cargo run -- cli summary x.dxf` 输出中文错误,`$LASTEXITCODE` = 2(验证 main 的退出码映射)。
- [ ] **Step 6: Commit** `chore: acad-mcp crate 脚手架与 CLI 骨架`

### Task 2: DXF 解析核心 `drawing.rs`

**Files:**
- Create: `src/drawing.rs`、`tests/common/mod.rs`
- Test: `tests/drawing_test.rs`(`mod common;` 引用共享 fixture)

**Interfaces:**
- Consumes: `acadrust = { version = "0.6", default-features = false }`(2026-11-17 spike 实测裁定,替代 `dxf`+LibreDWG,详见 `spike-dxf-engine/SPIKE_REPORT.md`)。已核对的真实 API:`acadrust::io::dwg::dwg_reader::{DwgReader, DwgReadOptions::failsafe()}`、`acadrust::io::dxf::DxfReader`、`doc.model_space_entities()`(模型空间迭代,M1 统计主路)、`doc.entities()`(全库迭代,作 db_entity_total 自洽参照)、`doc.layers()` / `doc.block_records.names()`;实体类型 `EntityType`,`Unknown` 变体保留原始 `dxf_name`(天正 TCH_* / 代理实体身份无损);`Insert` 字段 `block_name` / `attributes`(tag/value);`DrawingStats` 含 `entity_total`(模型空间)与 `db_entity_total`(全库)双口径并作直方图自洽断言。
- Produces:

```rust
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct LayerInfo { pub name: String, pub is_on: bool, pub plottable: bool, pub color: Option<u8> }

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DrawingStats {
    pub layers: Vec<LayerInfo>,
    pub blocks: Vec<String>,                       // 块定义名,排除匿名块
    pub entity_counts: std::collections::BTreeMap<String, u64>,   // DXF 类型名 -> 数量
    pub insert_counts: std::collections::BTreeMap<String, u64>,   // INSERT 引用按块名
    pub insert_attributes: std::collections::BTreeMap<String, Vec<std::collections::BTreeMap<String, String>>>, // 块名 -> 每个插入的属性表
    pub proxy_entities: std::collections::BTreeMap<String, u64>,  // 图层名 -> 代理实体数
    pub entity_total: u64,
}

pub fn entity_type_name(t: &dxf::entities::EntityType) -> &'static str
pub fn summarize(dxf_path: &std::path::Path) -> anyhow::Result<DrawingStats>
```

- **命名口径**:`entity_counts` 的键是 DXF 组码 0 的类型名,由 `entity_type_name` 映射:`LINE`/`LWPOLYLINE`/`POLYLINE`/`INSERT`/`ATTRIB`/`ATTDEF`/`TEXT`/`MTEXT`/`CIRCLE`/`ARC`/`ACAD_PROXY_ENTITY`,其余统一 `OTHER`。**禁止使用 `AcDb*` 字符串**——那是 AutoCAD DRX 类名,本 crate 不提供。
- **冻结状态**:dxf 0.6.2 的 `Layer` 未公开 frozen(私有字段、无访问器),`LayerInfo` 不采集该项,也不要为此自写解析器。
- **代理实体**:天正自定义对象在本 crate 中是 `EntityType::ProxyEntity`,只有二进制块(`entity_data_string`),无法还原语义 → 按图层计入 `proxy_entities`,不进设备清单;xlsx 与 `tests/fixtures/README.md` 必须记录"未解析代理实体 N 个"。
- **内存口径**:`Drawing::load_file` 会把整图解析进内存(crate 设计,无流式 API)。本任务不再承诺"不加载全图",改为:只做一次遍历、`entity_total` 必须等于 `entity_counts` 之和,并在 README 记录样图 DXF 大小/实体总数作为后续基线。

匿名块判定:块名以 `*` 或 `A$` 开头即排除(布局空间/匿名块)。

- [ ] **Step 1: 写失败测试**(fixture 精确值,放在 `tests/common/mod.rs`):

手写最小合法 DXF:TABLES 含图层 `0`(on/plot/白)、`EQUIP`(关闭/不可打印:`62 = -4` → `is_layer_on=false` 且 `color.index()=None`,`290 = 0`)、`暖通-风管`(on/plot,`62 = 7`);BLOCKS 含定义 `风机-1`(带 2 个 ATTDEF:`功率`=null,`编号`)与 `*D1`;ENTITIES 含 3 个 INSERT `风机-1`(各带 ATTRIB:`功率`=`1.5kW`/`2.2kW`/`1.1kW`,`编号`=`F-1`/`F-2`/`F-3`)、1 个 INSERT `*D1`、10 条 LINE、2 条 **LWPOLYLINE**、1 个 `ACAD_PROXY_ENTITY`(图层 `0`)。

```rust
// tests/common/mod.rs
pub const MINIMAL_DXF: &str = /* 上述手写 DXF 全文 */;
pub fn write_minimal_dxf() -> tempfile::TempPath { /* 写入 tempfile 并返回 */ }
pub fn minimal_stats() -> acad_mcp::drawing::DrawingStats { /* summarize(write_minimal_dxf()) */ }

// tests/drawing_test.rs
mod common;

#[test] fn layers_parsed()      // EQUIP: is_on=false、plottable=false、color=None;暖通-风管: is_on=true、plottable=true、color=Some(7)
#[test] fn anonymous_excluded() // blocks 不含 "*D1";insert_counts 不含 "*D1"
#[test] fn counts_exact()       // insert_counts["风机-1"]==3;entity_counts["LINE"]==10;entity_counts["LWPOLYLINE"]==2;entity_counts["INSERT"]==4
#[test] fn attributes_captured()// insert_attributes["风机-1"] 长度 3,其中一条含 功率=2.2kW、编号=F-2
#[test] fn cjk_names_ok()       // blocks 含 "风机-1",layers 含 "暖通-风管"
#[test] fn proxy_counted()      // proxy_entities["0"]==1,且 "ACAD_PROXY_ENTITY" 不进 insert_counts
#[test] fn counts_are_total()   // entity_counts.values().sum::<u64>() == entity_total == 17
```

- [ ] **Step 2:** `cargo test --test drawing_test` → FAIL(函数未定义)。
- [ ] **Step 3:** 按下列实现填充 `src/drawing.rs`(一次遍历,不 `.collect()` 实体):

```rust
use anyhow::Result;
use dxf::entities::EntityType;
use std::collections::BTreeMap;
use std::path::Path;

fn is_anonymous(name: &str) -> bool { name.starts_with('*') || name.starts_with("A$") }

pub fn entity_type_name(t: &EntityType) -> &'static str {
    match t {
        EntityType::Line(_) => "LINE",
        EntityType::LwPolyline(_) => "LWPOLYLINE",
        EntityType::Polyline(_) => "POLYLINE",
        EntityType::Insert(_) => "INSERT",
        EntityType::Attribute(_) => "ATTRIB",
        EntityType::AttributeDefinition(_) => "ATTDEF",
        EntityType::Text(_) => "TEXT",
        EntityType::MText(_) => "MTEXT",
        EntityType::Circle(_) => "CIRCLE",
        EntityType::Arc(_) => "ARC",
        EntityType::ProxyEntity(_) => "ACAD_PROXY_ENTITY",
        _ => "OTHER",
    }
}

pub fn summarize(dxf_path: &Path) -> Result<DrawingStats> {
    let drawing = dxf::Drawing::load_file(dxf_path)?;
    let layers = drawing.layers().map(|l| LayerInfo {
        name: l.name.clone(),
        is_on: l.is_layer_on,
        plottable: l.is_layer_plotted,
        color: l.color.index(),
    }).collect();
    let blocks = drawing.blocks()
        .filter(|b| !is_anonymous(&b.name))
        .map(|b| b.name.clone())
        .collect();
    let mut stats = DrawingStats { layers, blocks, ..Default::default() };
    for e in drawing.entities() {
        *stats.entity_counts.entry(entity_type_name(&e.specific).to_string()).or_default() += 1;
        stats.entity_total += 1;
        match &e.specific {
            EntityType::Insert(ins) if !is_anonymous(&ins.name) => {
                *stats.insert_counts.entry(ins.name.clone()).or_default() += 1;
                let attrs: BTreeMap<String, String> = ins.attributes()
                    .map(|a| (a.attribute_tag.clone(), a.value.clone()))
                    .collect();
                stats.insert_attributes.entry(ins.name.clone()).or_default().push(attrs);
            }
            EntityType::ProxyEntity(_) => {
                *stats.proxy_entities.entry(e.common.layer.clone()).or_default() += 1;
            }
            _ => {}
        }
    }
    Ok(stats)
}
```

- [ ] **Step 4:** `cargo test --test drawing_test` → 全 PASS。
- [ ] **Step 5: Commit** `feat: DXF 图层/块/实体统计核心`

### Task 3: 真实样图 fixture 转换与集成验证

**Files:**
- Create: `scripts/convert_sample.ps1`、`tests/fixtures/README.md`
- Test: `tests/sample_integration_test.rs`(环境变量门控)

**Interfaces:**
- Consumes: Task 2 `summarize`
- Produces: 集成测试内可用的样图 `DrawingStats`(转换产物 `tests/fixtures/yaneng1.dxf`,gitignore)

- [ ] **Step 1: 转换脚本** `convert_sample.ps1`:从 [LibreDWG releases](https://github.com/LibreDWG/libredwg/releases) 下载对应 Windows 版 `dwg2dxf.exe` 至 `tools/libredwg/`(脚本含 URL 变量与手工放置说明;仓库不含该二进制),将 `DWG_SOURCE`(默认设计文档所列样图路径)转出 `tests/fixtures/yaneng1.dxf`;输出其文件大小。
- [ ] **Step 2: 写门控集成测试**:

```rust
#[test]
fn sample_stats() {
    let p = std::env::var("CAD_FIXTURE_DWG").expect("缺少 CAD_FIXTURE_DWG:真实样图集成测试硬失败,不静默跳过");
    let stats = acad_mcp::drawing::summarize(p.as_ref()).unwrap();
    assert!(stats.entity_total > 10000);
    assert_eq!(stats.entity_counts.values().sum::<u64>(), stats.entity_total); // 计数自洽
    assert!(stats.blocks.iter().any(|b| b.chars().any(|c| (c as u32) > 0x4E00))); // 存在 CJK 块名
    assert!(!stats.layers.is_empty());
    eprintln!("未解析代理实体: {:?}", stats.proxy_entities);
}
```

- [ ] **Step 3:** 在含空格+中文+`#` 的临时目录复制 `雁能1#.dwg`(如 `$env:TEMP\测试 副本#1\sample.dwg`)后设 `CAD_FIXTURE_DWG` 再跑,期望同 PASS(路径健壮性)。
- [ ] **Step 4: 异道核验(不共享 crate 代码)**,用 PowerShell 原始组码扫描独立复算再对比:

```powershell
# 只扫 ENTITIES 段:组码 0 后紧跟的类型名即实体类型
$lines = Get-Content -LiteralPath $env:CAD_FIXTURE_DWG -Encoding UTF8
$counts = @{}; $inEnt = $false
for ($i = 0; $i -lt $lines.Count - 1; $i++) {
  if ($lines[$i].Trim() -eq '2' -and $lines[$i+1].Trim() -eq 'ENTITIES') { $inEnt = $true; continue }
  if ($inEnt -and $lines[$i].Trim() -eq '0') { $t = $lines[$i+1].Trim(); $counts[$t] = 1 + [int]$counts[$t] }
}
$counts.GetEnumerator() | Sort-Object Value -Descending | Select-Object -First 12
```

与 `DrawingStats.entity_counts` 逐项比对:已知类型必须完全一致,`OTHER` 允许聚合差异;不一致即失败。比对结果写入 `tests/fixtures/README.md`,AutoCAD/天正侧读数列为 M4 补证项(M1 不作为门槛)。
- [ ] **Step 5:** 执行脚本得到真实数据,记录 `yaneng1.dxf` 的实体总数、Top-10 块名、`proxy_entities` 分层计数,以及转换前后 `Get-FileHash` 值(证明源图未被改动)到 `tests/fixtures/README.md`(人工核对:块名是否如 `AP`/`AL`/风机/水泵等电气/机电命名;代理实体数用于判断天正降级缺口)。
- [ ] **Step 6: Commit** `test: 样图转换脚本与集成测试(gitignore 大 fixture)`

### Task 4: 资产登记 `asset.rs` 与 xlsx 生成

**Files:**
- Create: `src/asset.rs`、`config/asset_rules.json`
- Modify: `src/main.rs`(Register/Read op 接线在 Task 5/6;此处仅 lib 层)
- Test: `tests/asset_test.rs`(`mod common;` 复用 Task 2 的 fixture;`calamine` 已在 dev-deps)

**Interfaces:**
- Consumes: Task 2 `DrawingStats`
- Produces:

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CategoryRule { pub category: String, pub block_patterns: Vec<String>, pub layer_patterns: Vec<String> } // regex 片段,不分大小写
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RegisterConfig { pub rules: Vec<CategoryRule> }
pub fn default_config() -> RegisterConfig        // include_str!("../config/asset_rules.json") + serde_json
#[derive(Debug, Clone, serde::Serialize)]
pub struct AssetRow { pub category: String, pub block_name: String, pub count: u64, pub attributes: Vec<BTreeMap<String,String>> }
pub fn build_register(stats: &DrawingStats, cfg: &RegisterConfig) -> Vec<AssetRow>   // 命中任意 pattern 即归类;一图块可归多类时按 rules 数组顺序取首个命中;未命中归 "未分类"
pub fn write_xlsx(rows: &[AssetRow], out: &Path) -> anyhow::Result<()>     // 仅负责写出;同路径/覆盖守卫在 ops::register(K5);表头:序号/类别/图块名/数量/属性(功率=1.5kW;编号=F-1 合并串)
```

`config/asset_rules.json` 内置起步规则(块名正则,后续 skills 迭代):`配电箱`:`^(AP|AL|AW|EAL|EAP|PE|XF)`;`风机`:`(风机|SF|HTF|BSF|JYF|PYF)`;`水泵`:`(水泵|B|SB)`;`灯具`:`(灯|LAMP|E..[0-9])`;`桥架`:`(桥架|CT)`;`母线`:`(母线|BUSWAY)`。pattern 匹配对象为块名(含中文)与实体所在图层名,`(?i)` 不分大小写。

- [ ] **Step 1: 写失败测试**(`mod common;` 复用 Task 2 的 `minimal_stats()`):

```rust
mod common;

#[test] fn classify_and_count()   // 断言:rows 含 {category:"风机", block_name:"风机-1", count:3};*D1 不出现
#[test] fn xlsx_roundtrip()       // write_xlsx 到临时目录 → calamine 读回:首行表头含 "类别";存在某行 类别="风机" 且数量=3;属性合并串含 "功率=1.5kW"
#[test] fn uncategorized_bucket() // 往 common::minimal_stats() 的 blocks/insert_counts 里混入无规则块 → 归 "未分类"
#[test] fn rule_order_wins()      // 同一块名命中两类时,按 asset_rules.json 数组顺序取第一个
```

- [ ] **Step 2:** `cargo test --test asset_test` → FAIL。
- [ ] **Step 3:** 实现三函数与 `config/asset_rules.json`。
- [ ] **Step 4:** `cargo test --test asset_test` → 全 PASS。
- [ ] **Step 5:** Task 3 真实样图接一次全链:`summarize(yaneng1.dxf)` → `build_register` → `write_xlsx` 至 `tests/fixtures/yaneng1_assets.xlsx`,人工抽查 xlsx 分类是否合理,并把 `proxy_entities` 合计写入 README(此步为人工验证,不写自动断言)。
- [ ] **Step 6: Commit** `feat: 机电设备分类清点与资产登记表 xlsx`

### Task 5: MCP server 暴露(rmcp 3.10)

**Files:**
- Create: `src/server.rs`、`src/ops.rs`
- Modify: `src/lib.rs`(`pub mod server; pub mod ops;`)、`src/main.rs`(`Serve` 与 `CliOp` 全部接线)
- Test: `tests/server_test.rs`(`mod common;`)

**Interfaces:**
- Consumes: Task 2 `summarize`、Task 4 `default_config/build_register/write_xlsx`
- Produces:

```rust
// src/ops.rs —— CLI 与 MCP 共用同一实现,统一返回 serde_json::Value
pub fn summary(dxf_path: &str) -> anyhow::Result<serde_json::Value>;
pub fn layers(dxf_path: &str) -> anyhow::Result<serde_json::Value>;
pub fn blocks(dxf_path: &str, filter: Option<&str>) -> anyhow::Result<serde_json::Value>;
// K5:输入 canonicalize;输出取 canonicalize(parent)+文件名(可能尚不存在)后比对,同路径直接 bail;
// 读图前/写出后各算一次 SHA-256,返回 {rows, sha256_before, sha256_after, unchanged}
pub fn register(dxf_path: &str, out_xlsx: &str) -> anyhow::Result<serde_json::Value>;

// src/server.rs
#[derive(Clone)]
pub struct AcadMcpServer;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DrawingSummaryParams { pub dxf_path: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListLayersParams { pub dxf_path: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CountBlocksParams { pub dxf_path: String, pub filter: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AssetRegisterParams { pub dxf_path: String, pub out_xlsx: String }

#[tool_router(server_handler)]
impl AcadMcpServer {
    #[tool(description = "统计 DXF 图纸:图层数/块定义/实体类型直方图/块插入计数(含匿名块排除)")]
    async fn drawing_summary(&self, Parameters(p): Parameters<DrawingSummaryParams>) -> Result<CallToolResult, ErrorData>;

    #[tool(description = "列出图层:名称/开关/可打印/颜色")]
    async fn list_layers(&self, Parameters(p): Parameters<ListLayersParams>) -> Result<CallToolResult, ErrorData>;

    #[tool(description = "按正则过滤块插入计数")]
    async fn count_blocks(&self, Parameters(p): Parameters<CountBlocksParams>) -> Result<CallToolResult, ErrorData>;

    #[tool(description = "生成机电设备资产登记表 xlsx")]
    async fn asset_register(&self, Parameters(p): Parameters<AssetRegisterParams>) -> Result<CallToolResult, ErrorData>;
}

pub async fn serve_stdio() -> anyhow::Result<()>
```

**已核对的 rmcp 3.10 约束**(docs.rs,2026-11-17):
- `#[tool]` 的参数必须是 `Parameters<T>`,`T: serde::Deserialize + schemars::JsonSchema`;裸 `String` 不接受。
- `#[tool_router(server_handler)]` 会自动生成 `impl ServerHandler`(`get_info` 从 `Cargo.toml` 取包名/版本),无需手写 `#[tool_handler]`。
- `CallToolResult::success(Vec<ContentBlock>)`;`ContentBlock::text(impl Into<String>)`;错误类型为 `rmcp::ErrorData`(`McpError` 在 3.x 已不存在)。
- stdio 启动:`AcadMcpServer.serve(rmcp::transport::io::stdio()).await?.waiting().await?`(`use rmcp::ServiceExt;`)。

`count_blocks` filter 用 Task 4 同款正则语义;`asset_register` 固定读内置 `asset_rules.json`。工具体写法(其余三个同构):

```rust
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ErrorData};

async fn drawing_summary(&self, Parameters(p): Parameters<DrawingSummaryParams>)
    -> Result<CallToolResult, ErrorData>
{
    let v = crate::ops::summary(&p.dxf_path)
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(v.to_string())]))
}
```

- [ ] **Step 1: 写失败测试**(rmcp 进程内 in-memory transport:`tokio::io::duplex` 满足 `AsyncRead + AsyncWrite`,rmcp 的 `IntoTransport` 直接接受):

```rust
mod common;

use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::ServiceExt;

#[tokio::test]
async fn tool_drawing_summary() {
    let dxf = common::write_minimal_dxf();
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    let server = tokio::spawn(async move {
        acad_mcp::server::AcadMcpServer.serve(server_io).await.unwrap().waiting().await
    });
    let client = ().serve(client_io).await.unwrap();
    let res = client.peer().call_tool(CallToolRequestParams {
        meta: None,
        name: "drawing_summary".into(),
        arguments: Some(serde_json::json!({ "dxf_path": dxf.to_string_lossy() }).as_object().unwrap().clone()),
        task: None,
    }).await.unwrap();
    let text: String = res.content.iter().filter_map(|c| match c {
        ContentBlock::Text(t) => Some(t.text.clone()),
        _ => None,
    }).collect();
    assert!(text.contains("\"风机-1\":3"));
    drop(client);
    server.abort();
}

#[tokio::test]
async fn tool_asset_register_writes_file() {
    // 同上:call asset_register {dxf_path, out_xlsx=临时目录下 out.xlsx}
    // 断言文件存在且 >0 字节;返回文本含 "行数" 与 "sha256_before"
}

#[test]
fn register_rejects_same_path() {
    let dxf = common::write_minimal_dxf();
    let p = dxf.to_string_lossy().to_string();
    let err = acad_mcp::ops::register(&p, &p).unwrap_err();
    assert!(err.to_string().contains("同一路径")); // K5 护栏
}
```

- [ ] **Step 2:** `cargo test --test server_test` → FAIL。
- [ ] **Step 3:** 实现 `src/ops.rs`、`src/server.rs` 与 main 接线(`Serve` → `serve_stdio()`;`CliOp` → `ops::*` 后 `println!("{}", v)` 打印 JSON)。`register` 内按 K5 实现:输入 `fs::canonicalize`,输出用 `canonicalize(parent) + file_name`(输出可能尚不存在)比较,同路径 bail(消息含"同一路径");`sha2::Sha256` 读前/写后各算一次,`unchanged` 为 false 时返回错误。
- [ ] **Step 4:** `cargo test` 全部 PASS;手动 `cargo run -- cli summary <手写 fixture>` 输出 JSON(验证 CLI 与工具共用同一实现)。
- [ ] **Step 5: Commit** `feat: rmcp stdio server 与工具暴露,CLI 直调接线`

### Task 6: anydoc 读文档工具 + goose 接入文档

**Files:**
- Modify: `src/server.rs`(追加工具)、`src/main.rs`(`CliOp::Read` 接线)、`Cargo.toml`(加 `anydoc` crate,版本取 crates.io 最新,API 按其 README `anydoc::to_markdown(path) -> Result<String, ConvertError>`)
- Create: `docs/goose-setup.md`、`config/goose-config-example.yaml`
- Test: `tests/read_document_test.rs`

**Interfaces:**
- Consumes: Task 5 server 结构;anydoc crate
- Produces:

```rust
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadDocumentParams { pub path: String }

#[tool(description = "读取 docx/xlsx/pdf 等文档为 Markdown(设计说明/设备表)")]
async fn read_document(&self, Parameters(p): Parameters<ReadDocumentParams>) -> Result<CallToolResult, ErrorData>;

pub fn read_document_impl(path: &str) -> anyhow::Result<String>; // 内部调用 anydoc::to_markdown
```

- [ ] **Step 1: 写失败测试**:

```rust
#[test]
fn xlsx_to_markdown_roundtrip() {
    // rust_xlsxwriter 写含 "配电箱AL1" 与 "风机" 单元格的临时 xlsx
    let md = acad_mcp::server::read_document_impl(path).unwrap();
    assert!(md.contains("配电箱AL1") && md.contains("风机"));
}
```

- [ ] **Step 2:** FAIL → 实现 `read_document_impl` 与工具包装 → PASS。
- [ ] **Step 3:** `docs/goose-setup.md`:goose 安装(Windows)、把 `acad-mcp serve` 注册为 MCP server 的配置示例(`config/goose-config-example.yaml`)、需要 LLM provider key 的说明(键位留空,标注"由业主安装时填写")。
- [ ] **Step 4:** `cargo test` 全绿;`cargo run -- cli read <某xlsx>` 输出 Markdown。
- [ ] **Step 5: Commit** `feat: anydoc 文档读取工具与 goose 接入文档`

### Task 7: M1 收尾——README 与全量回归

**Files:**
- Create: `cad-electrical-agent/README.md`(架构一句话、构建/测试方法、fixture 再生成步骤、M2-M5 展望链接)、`cad-electrical-agent/docs/decisions/0001-stack-baseline.md`
- Modify: `superpowers/plans/` 本文件勾选完成项

- [ ] **Step 1:** `cargo test` 全量 + `cargo clippy -- -D warnings` + `cargo fmt --check` 全过。
- [ ] **Step 2:** 按 README 步骤从零走一遍(clone 级验证:转换样图→CLI register→xlsx 生成)。
- [ ] **Step 3: 写决策记录** `docs/decisions/0001-stack-baseline.md`(K6):钉住 K3 环境基线(AutoCAD 2020 R23.1 + 天正 V10 已验证,2024/2025/2014 未验证)、goose/rmcp/Rust/LibreDWG/anydoc 的 `status: unverified` 与"库内无项目级评估"事实、M1 异道核验结论;结构:背景 / 决策 / 证据 / 未验证项。
- [ ] **Step 4: Commit** `docs: M1 README 与回归记录`

---

## 交付验收(M1 Definition of Done)

1. `cargo test` 全绿,样图集成测试经 `CAD_FIXTURE_DWG` 门控(未设时硬失败,设置后全过;AC1018 电气样本)
2. `acad-mcp cli register <yaneng1.dxf> out.xlsx` 产出可人工核对的资产登记表,README 记录 Top-10 块名、分类结论、原始组码异道核验比对结果与源文件 SHA-256 前后一致
3. rmcp stdio server 可被任意 MCP client 列出 5 个工具并调用成功
4. goose 配置文档就绪(实际连 goose 冒烟推迟到 provider key 到位,记入 M3)
5. 写路径护栏生效:输出与输入同路径被拒;读前/写后源 DXF SHA-256 一致
6. `docs/decisions/0001-stack-baseline.md` 落盘,含 `unverified` 标记与库内无评估声明
