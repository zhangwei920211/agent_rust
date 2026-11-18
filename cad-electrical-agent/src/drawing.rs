//! 读图统计核心（M1 Task2）。引擎：acadrust（spike 实测裁定，原生 DWG/DXF）。
//! 反目标：只读图纸，不写任何源图。
//! 关键设计：Unknown 实体用其 `dxf_name` 作类型名，保留天正 ACAD_PROXY_ENTITY 等原始身份。

use acadrust::entities::{Entity, EntityType};
use acadrust::io::dxf::DxfReader;
use acadrust::io::dwg::dwg_reader::{DwgReadOptions, DwgReader};
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// 图层信息（plan LayerInfo）。
#[derive(Debug, Clone, Serialize)]
pub struct LayerInfo {
    pub name: String,
    pub is_on: bool,
    pub is_frozen: bool,
    pub plottable: bool,
    pub color: i16,
}

/// 单个 INSERT 引用记录（块名+所在图层+属性）——设备分类按图层+块名+属性。
#[derive(Debug, Clone, Serialize)]
pub struct InsertRecord {
    pub block_name: String,
    pub layer: String,
    pub attributes: BTreeMap<String, String>,
}

/// 图纸统计（plan DrawingStats，增 source_version/skipped 作损失证据）。
#[derive(Debug, Clone, Serialize, Default)]
pub struct DrawingStats {
    pub layers: Vec<LayerInfo>,
    /// 块定义名，排除匿名块（* / A$）
    pub blocks: Vec<String>,
    /// 实体类型直方图（Unknown 用 dxf_name，故 ACAD_PROXY_ENTITY 正确呈现）
    pub entity_counts: BTreeMap<String, u64>,
    /// 模型空间实体总数（本轮统计口径，与 ezdxf 模型空间清点基线可比）
    pub entity_total: u64,
    /// 全库实体总数（含块定义/布局），用于口径差额证据
    pub db_entity_total: u64,
    /// INSERT 引用按块名计数（排除匿名块）——设备清点用引用数而非定义数
    pub insert_counts: BTreeMap<String, u64>,
    /// 块名 -> 每个插入的属性表（tag->value）
    pub insert_attributes: BTreeMap<String, Vec<BTreeMap<String, String>>>,
    /// 每个非匿名 INSERT 的块名+图层+属性（设备分类用）
    pub insert_records: Vec<InsertRecord>,
    pub source_version: String,
    pub skipped_records: usize,
    pub recovered_errors: usize,
}

/// 匿名/布局块判定：以 `*` 或 `A$` 开头即排除（plan:122）。
pub fn is_anonymous_block(name: &str) -> bool {
    name.starts_with('*') || name.starts_with("A$")
}

/// 实体类型名：Unknown 取原始 dxf_name（保留代理身份），其余取 entity_type()。
fn type_name(e: &EntityType) -> String {
    match e {
        EntityType::Unknown(u) => u.dxf_name.clone(),
        _ => e.as_entity().entity_type().to_string(),
    }
}

/// 读图统计入口。按扩展名分派 DWG/DXF，均 failsafe 读取并保留诊断计数。
pub fn summarize(path: &Path) -> Result<DrawingStats> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let outcome = if ext == "dwg" {
        let mut reader = DwgReader::from_file_with_options(path, DwgReadOptions::failsafe())
            .with_context(|| format!("打开 DWG 失败：{}", path.display()))?;
        reader
            .read_with_stats()
            .with_context(|| format!("读取 DWG 失败：{}", path.display()))?
    } else {
        let reader = DxfReader::from_file(path)
            .with_context(|| format!("打开 DXF 失败：{}", path.display()))?;
        reader
            .read_with_stats()
            .with_context(|| format!("读取 DXF 失败：{}", path.display()))?
    };

    let doc = &outcome.document;
    let mut stats = DrawingStats {
        source_version: outcome.stats.source_version.clone(),
        skipped_records: outcome.stats.skipped_source_records,
        recovered_errors: outcome.stats.recovered_errors,
        ..Default::default()
    };

    for l in doc.layers.iter() {
        stats.layers.push(LayerInfo {
            name: l.name.clone(),
            is_on: !l.flags.off,
            is_frozen: l.flags.frozen,
            plottable: l.is_plottable,
            color: l.color.index().unwrap_or(7) as i16,
        });
    }

    for n in doc.block_records.names() {
        if !is_anonymous_block(n) {
            stats.blocks.push(n.to_string());
        }
    }

    // 迭代器遍历，禁 collect 全实体（plan Review Focus 1：防 OOM/爆栈）。
    // 口径=模型空间（doc.model_space_entities）：ezdxf 清点基线即模型空间口径；
    // 全库 doc.entities() 会把块定义/布局实体计入，导致 33337 vs 6436 口径混用（spike 教训）。
    let mut entity_total: u64 = 0;
    for e in doc.model_space_entities() {
        entity_total += 1;
        let name = type_name(e);
        *stats.entity_counts.entry(name).or_insert(0) += 1;

        if let EntityType::Insert(ins) = e {
            if !is_anonymous_block(&ins.block_name) {
                *stats
                    .insert_counts
                    .entry(ins.block_name.clone())
                    .or_insert(0) += 1;
                let attrs: BTreeMap<String, String> = ins
                    .attributes
                    .iter()
                    .map(|a| (a.tag.clone(), a.value.clone()))
                    .collect();
                stats.insert_records.push(InsertRecord {
                    block_name: ins.block_name.clone(),
                    layer: ins.layer().to_string(),
                    attributes: attrs.clone(),
                });
                if !attrs.is_empty() {
                    stats
                        .insert_attributes
                        .entry(ins.block_name.clone())
                        .or_default()
                        .push(attrs);
                }
            }
        }
    }

    stats.entity_total = entity_total;
    // 全库实体数（含块定义/布局），仅 count 不遍历内容，作口径差额证据
    stats.db_entity_total = doc.entities().count() as u64;

    // 自洽断言：直方图总和必须等于模型空间迭代数；模型空间必须 ⊆ 全库
    let hist_sum: u64 = stats.entity_counts.values().sum();
    anyhow::ensure!(
        hist_sum == entity_total,
        "实体直方图总和 {hist_sum} 与模型空间迭代数 {entity_total} 不一致"
    );
    anyhow::ensure!(
        entity_total <= stats.db_entity_total,
        "模型空间实体数 {entity_total} 超过全库实体数 {}，统计口径异常",
        stats.db_entity_total
    );

    Ok(stats)
}
