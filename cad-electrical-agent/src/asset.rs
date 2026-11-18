//! 资产登记（M1 Task4）：规则分类 -> rust_xlsxwriter 资产登记表。
//! 规则据雁能 AC1018 真实数据归纳（probe.log）：图层主分类 + 属性 A 取设备名 + 排除非设备。
//! 反目标：只读图纸，唯一写动作是生成 xlsx 到用户指定路径。
//! 命名/分类规则 provisional，待业主确认固化（spec:119）。

use crate::drawing::DrawingStats;
use anyhow::{Context, Result};
use regex::RegexBuilder;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// 分类规则（plan CategoryRule）：regex 片段，不分大小写。
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryRule {
    pub category: String,
    #[serde(default)]
    pub layer_patterns: Vec<String>,
    #[serde(default)]
    pub block_patterns: Vec<String>,
}

/// 登记配置（plan RegisterConfig）。
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterConfig {
    pub rules: Vec<CategoryRule>,
    #[serde(default)]
    pub exclude_layer_patterns: Vec<String>,
    /// 取设备名的属性键优先级（A=天正设备名，产品名称=通用图库）
    #[serde(default)]
    pub name_attribute_keys: Vec<String>,
}

/// 资产登记表一行（按 类别+设备名 聚合）。
#[derive(Debug, Clone, PartialEq)]
pub struct AssetRow {
    pub category: String,
    pub name: String,
    pub quantity: u64,
    pub layers: String,
    pub sample_block: String,
}

/// 内置默认规则（include_str!，plan Task4）。
pub fn default_config() -> RegisterConfig {
    serde_json::from_str(include_str!("../config/asset_rules.json"))
        .expect("内置 asset_rules.json 应合法")
}

fn compile(patterns: &[String]) -> Vec<regex::Regex> {
    patterns
        .iter()
        .filter_map(|p| RegexBuilder::new(p).case_insensitive(true).build().ok())
        .collect()
}

/// 按规则分类聚合 INSERT -> 资产行。排除非设备图层；未匹配归「未分类」待核。
pub fn build_register(stats: &DrawingStats, config: &RegisterConfig) -> Vec<AssetRow> {
    let exclude = compile(&config.exclude_layer_patterns);
    let rules: Vec<(String, Vec<regex::Regex>, Vec<regex::Regex>)> = config
        .rules
        .iter()
        .map(|r| {
            (
                r.category.clone(),
                compile(&r.layer_patterns),
                compile(&r.block_patterns),
            )
        })
        .collect();

    // (类别, 设备名) -> (数量, 图层集合, 样例块名)
    let mut agg: BTreeMap<(String, String), (u64, BTreeSet<String>, String)> = BTreeMap::new();

    for r in &stats.insert_records {
        if exclude.iter().any(|re| re.is_match(&r.layer)) {
            continue;
        }
        let category = rules
            .iter()
            .find(|(_, lp, bp)| {
                lp.iter().any(|re| re.is_match(&r.layer))
                    || bp.iter().any(|re| re.is_match(&r.block_name))
            })
            .map(|(c, _, _)| c.clone())
            .unwrap_or_else(|| "未分类".to_string());

        let name = config
            .name_attribute_keys
            .iter()
            .find_map(|k| r.attributes.get(k).filter(|v| !v.is_empty()).cloned())
            .unwrap_or_else(|| r.block_name.clone());

        let entry = agg
            .entry((category, name))
            .or_insert_with(|| (0, BTreeSet::new(), r.block_name.clone()));
        entry.0 += 1;
        entry.1.insert(r.layer.clone());
    }

    let mut rows: Vec<AssetRow> = agg
        .into_iter()
        .map(|((category, name), (quantity, layers, sample_block))| AssetRow {
            category,
            name,
            quantity,
            layers: layers.into_iter().collect::<Vec<_>>().join(";"),
            sample_block,
        })
        .collect();
    rows.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(b.quantity.cmp(&a.quantity))
            .then(a.name.cmp(&b.name))
    });
    rows
}

/// 生成资产登记表 xlsx（中文表头）。唯一写动作，写到用户指定路径。
pub fn write_xlsx(rows: &[AssetRow], path: &Path) -> Result<()> {
    use rust_xlsxwriter::{Format, FormatBorder, Workbook};

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("资产登记表").context("设置工作表名失败")?;

    let header_fmt = Format::new()
        .set_bold()
        .set_background_color("#D9E1F2")
        .set_border(FormatBorder::Thin);
    let headers = ["序号", "类别", "设备名称", "数量", "图层", "样例块名"];
    for (c, h) in headers.iter().enumerate() {
        ws.write_with_format(0, c as u16, *h, &header_fmt)
            .context("写表头失败")?;
    }
    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write_number(r, 0, (i + 1) as f64)?;
        ws.write_string(r, 1, &row.category)?;
        ws.write_string(r, 2, &row.name)?;
        ws.write_number(r, 3, row.quantity as f64)?;
        ws.write_string(r, 4, &row.layers)?;
        ws.write_string(r, 5, &row.sample_block)?;
    }
    ws.autofilter(0, 0, rows.len() as u32, 5)?;
    ws.set_freeze_panes(1, 0)?;
    for (c, w) in [6.0, 14.0, 30.0, 8.0, 24.0, 34.0].iter().enumerate() {
        ws.set_column_width(c as u16, *w)?;
    }
    wb.save(path)
        .with_context(|| format!("保存 xlsx 失败：{}", path.display()))?;
    Ok(())
}
