//! drawing.rs 测试（M1 Task2，TDD）。
//! 纯逻辑单测无需样本；真实样本集成测试经 env CAD_FIXTURE_DWG 门控（缺失即失败，不再静默跳过——防“全绿”掩盖未执行的样图验收）。
//! 集成测试同时 dump entity_counts（含 Unknown 的 dxf_name），并行回答 UNKNOWN 身份问题。

use acad_mcp::drawing::{is_anonymous_block, summarize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[test]
fn anonymous_block_exclusion() {
    // 匿名/布局块排除（plan:122）
    assert!(is_anonymous_block("*D1"));
    assert!(is_anonymous_block("*Model_Space"));
    assert!(is_anonymous_block("*U10"));
    assert!(is_anonymous_block("A$C61E561FA"));
    // 正常块与天正设备块不排除
    assert!(!is_anonymous_block("风机-1"));
    assert!(!is_anonymous_block("$equip$00002943"));
    assert!(!is_anonymous_block("EQUIP-箱柜"));
}

#[test]
fn real_sample_stats() {
    // 缺样本即失败：DoD 要求样图集成测试真的跑过，静默跳过会造成假绿
    let p = std::env::var("CAD_FIXTURE_DWG").unwrap_or_else(|_| {
        panic!("缺少 CAD_FIXTURE_DWG：真实样图集成测试必须提供样本路径（AC1018 电气图）")
    });
    let stats = summarize(Path::new(&p)).expect("summarize 应成功");

    // 已知 spike 事实（雁能 AC1018 电气图）：电气图层存在
    assert!(
        stats.layers.iter().any(|l| l.name == "EQUIP-箱柜"),
        "应含 EQUIP-箱柜 图层"
    );
    assert!(stats.layers.iter().any(|l| l.name.contains("EQUIP-照明")));
    // 实体计数量级（2026-11-17 实测模型空间：LINE=27611 / INSERT=9871 / entity_total=82107；
    // 全文档参照：LINE 39193 / INSERT 12059 / 107087）
    let line_n = stats.entity_counts.get("LINE").copied().unwrap_or(0);
    let insert_n = stats.entity_counts.get("INSERT").copied().unwrap_or(0);
    eprintln!(
        "model-space: LINE={line_n} INSERT={insert_n} entity_total={} db_entity_total={}",
        stats.entity_total, stats.db_entity_total
    );
    assert!(
        line_n > 20000,
        "模型空间 LINE 应 >20000（实测 27611），actual={line_n}"
    );
    assert!(
        insert_n > 5000,
        "模型空间 INSERT 应 >5000（实测 9871，设备清点主路），actual={insert_n}"
    );
    // 天正设备块 INSERT 存在（设备清点主路）
    assert!(
        stats.insert_counts.keys().any(|k| k.starts_with("$equip$")),
        "应含 $equip$ 设备块 INSERT"
    );
    // 零丢失证据（spike：AC1018 skipped=0）
    assert_eq!(stats.skipped_records, 0, "AC1018 应零跳过");

    // 统计口径自洽断言（整改：模型空间切换后）
    let hist_sum: u64 = stats.entity_counts.values().sum();
    assert_eq!(
        hist_sum, stats.entity_total,
        "直方图总和应等于模型空间迭代数"
    );
    assert!(
        stats.entity_total <= stats.db_entity_total,
        "模型空间实体数 {} 不得超过全库 {}",
        stats.entity_total,
        stats.db_entity_total
    );

    // —— 并行 option 1：dump 实体类型直方图（Unknown 已按 dxf_name 命名）——
    let mut sorted: Vec<(&String, &u64)> = stats.entity_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("=== entity_counts 全量（含 Unknown 的 dxf_name 身份）===");
    for (k, v) in &sorted {
        eprintln!("  {} = {}", k, v);
    }
    let proxy = stats.entity_counts.get("ACAD_PROXY_ENTITY").copied().unwrap_or(0);
    eprintln!("=== ACAD_PROXY_ENTITY 计数 = {} ===", proxy);
    eprintln!(
        "layers={} blocks={} insert_blocks={} source_version={} skipped={} recovered={} entity_total(模型空间)={} db_entity_total(全库)={} 差额(块定义/布局等)={}",
        stats.layers.len(),
        stats.blocks.len(),
        stats.insert_counts.len(),
        stats.source_version,
        stats.skipped_records,
        stats.recovered_errors,
        stats.entity_total,
        stats.db_entity_total,
        stats.db_entity_total - stats.entity_total
    );
    // 设备块 INSERT Top-10（供资产分类核对）
    let mut ins: Vec<(&String, &u64)> = stats.insert_counts.iter().collect();
    ins.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("=== insert_counts Top-10 ===");
    for (k, v) in ins.iter().take(10) {
        eprintln!("  {} = {}", k, v);
    }
}

/// Task4 数据探针：dump INSERT 按图层分布 + 属性 tag 全集 + EQUIP 图层样本，
/// 为分类规则提供真实依据（实事求是：先调查再定规则）。
#[test]
fn asset_data_probe() {
    // 缺样本即失败（同 real_sample_stats：防静默跳过假绿）
    let p = std::env::var("CAD_FIXTURE_DWG").unwrap_or_else(|_| {
        panic!("缺少 CAD_FIXTURE_DWG：真实样图集成测试必须提供样本路径（AC1018 电气图）")
    });
    let stats = summarize(Path::new(&p)).expect("summarize 应成功");

    let mut by_layer: BTreeMap<String, u64> = BTreeMap::new();
    let mut tags: BTreeSet<String> = BTreeSet::new();
    for r in &stats.insert_records {
        *by_layer.entry(r.layer.clone()).or_insert(0) += 1;
        for t in r.attributes.keys() {
            tags.insert(t.clone());
        }
    }
    let mut lv: Vec<(&String, &u64)> = by_layer.iter().collect();
    lv.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("=== INSERT 按图层分布 (top 30) ===");
    for (k, v) in lv.iter().take(30) {
        eprintln!("  {} = {}", k, v);
    }
    eprintln!("=== 属性 tag 全集 ({} 个) ===", tags.len());
    for t in &tags {
        eprintln!("  TAG: {}", t);
    }
    eprintln!("=== EQUIP/WIRE/设备 图层 insert 样本（每层前3）===");
    let mut shown: BTreeMap<String, u64> = BTreeMap::new();
    for r in &stats.insert_records {
        if r.layer.starts_with("EQUIP") || r.layer.starts_with("WIRE") || r.layer.contains("设备") {
            let c = shown.entry(r.layer.clone()).or_insert(0);
            if *c < 3 {
                eprintln!("  [{}] block={} attrs={:?}", r.layer, r.block_name, r.attributes);
                *c += 1;
            }
        }
    }
}
