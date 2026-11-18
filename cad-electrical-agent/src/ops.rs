//! 业务操作封装（M1 Task5）：供 CLI 与 MCP 工具层复用。
//! 返回 `anyhow::Result<String>`（面向工具的文本/JSON 结果）；
//! 业务错误由上层映射为工具级可读结果，协议异常才走 McpError。

use crate::asset::{build_register, default_config, write_xlsx};
use crate::drawing::summarize;
use anyhow::{Context, Result};
use regex::RegexBuilder;
use std::collections::BTreeMap;
use std::path::Path;

/// 汇总图纸统计信息（JSON）。
pub fn drawing_summary(dxf_path: &str) -> Result<String> {
    let stats = summarize(Path::new(dxf_path))
        .with_context(|| format!("读取图纸失败: {dxf_path}"))?;
    serde_json::to_string_pretty(&stats).context("序列化统计结果失败")
}

/// 列出图层信息（JSON）。
pub fn list_layers(dxf_path: &str) -> Result<String> {
    let stats = summarize(Path::new(dxf_path))
        .with_context(|| format!("读取图纸失败: {dxf_path}"))?;
    serde_json::to_string_pretty(&stats.layers).context("序列化图层信息失败")
}

/// 统计块 INSERT 引用计数（JSON，可选正则过滤块名）。
pub fn count_blocks(dxf_path: &str, filter: Option<String>) -> Result<String> {
    let stats = summarize(Path::new(dxf_path))
        .with_context(|| format!("读取图纸失败: {dxf_path}"))?;
    let re = match filter {
        Some(pat) if !pat.is_empty() => Some(
            RegexBuilder::new(&pat)
                .case_insensitive(true)
                .build()
                .with_context(|| format!("无效的过滤正则: {pat}"))?,
        ),
        _ => None,
    };
    let mut out: BTreeMap<String, u64> = BTreeMap::new();
    for (name, count) in &stats.insert_counts {
        if re.as_ref().map(|r| r.is_match(name)).unwrap_or(true) {
            out.insert(name.clone(), *count);
        }
    }
    serde_json::to_string_pretty(&out).context("序列化块计数失败")
}

/// 生成资产登记表 xlsx，返回结果摘要（JSON）。
pub fn asset_register(dxf_path: &str, out_xlsx: &str) -> Result<String> {
    let stats = summarize(Path::new(dxf_path))
        .with_context(|| format!("读取图纸失败: {dxf_path}"))?;
    let cfg = default_config();
    let rows = build_register(&stats, &cfg);
    let out_path = Path::new(out_xlsx);
    write_xlsx(&rows, out_path).with_context(|| format!("写入资产登记表失败: {out_xlsx}"))?;

    let mut by_category: BTreeMap<String, u64> = BTreeMap::new();
    let mut total: u64 = 0;
    for r in &rows {
        *by_category.entry(r.category.clone()).or_insert(0) += r.quantity;
        total += r.quantity;
    }
    let summary = serde_json::json!({
        "out_xlsx": out_xlsx,
        "row_count": rows.len(),
        "total_quantity": total,
        "by_category": by_category,
    });
    serde_json::to_string_pretty(&summary).context("序列化资产登记结果失败")
}
