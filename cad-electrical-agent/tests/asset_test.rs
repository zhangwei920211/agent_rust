//! asset.rs 测试（M1 Task4，TDD）。
//! 单测用合成 InsertRecord 验证分类/排除/未分类/名称回退；
//! 集成测试经 env CAD_FIXTURE_DWG 门控：真实样本 -> 分类 -> xlsx -> calamine 读回断言。

use acad_mcp::asset::{build_register, default_config, write_xlsx};
use acad_mcp::drawing::{DrawingStats, InsertRecord};
use std::collections::BTreeMap;

fn rec(block: &str, layer: &str, a: &str) -> InsertRecord {
    let mut attrs = BTreeMap::new();
    if !a.is_empty() {
        attrs.insert("A".to_string(), a.to_string());
    }
    InsertRecord {
        block_name: block.to_string(),
        layer: layer.to_string(),
        attributes: attrs,
    }
}

#[test]
fn classification_rules() {
    let cfg = default_config();
    let mut stats = DrawingStats::default();
    stats.insert_records = vec![
        rec("$equip$1", "EQUIP-照明", "双管格栅灯"),
        rec("$equip$1", "EQUIP-照明", "双管格栅灯"),
        rec("$equip$2", "EQUIP-插座", "安全型五孔插座"),
        rec("$equip$3", "EQUIP-箱柜", "照明配电箱"),
        rec("$equip$4", "EQUIP-消防", "感烟火灾探测器"),
        rec("$TelecSys$1", "EQUIP-通讯", "数据插座"),
        rec("line", "LWIRE", ""),
        rec("fur", "家具", ""),
        rec("ws", "VC_设备编号", ""),
        rec("$equip$9", "某未知图层", "mystery"),
    ];
    let rows = build_register(&stats, &cfg);
    let find = |cat: &str, name: &str| rows.iter().find(|r| r.category == cat && r.name == name);
    assert_eq!(find("照明", "双管格栅灯").unwrap().quantity, 2);
    assert!(find("插座", "安全型五孔插座").is_some());
    assert!(find("配电箱柜", "照明配电箱").is_some());
    assert!(find("消防", "感烟火灾探测器").is_some());
    assert!(find("通讯智能化", "数据插座").is_some());
    assert!(find("未分类", "mystery").is_some());
    // 排除项不应出现
    assert!(!rows
        .iter()
        .any(|r| r.layers.contains("LWIRE") || r.layers.contains("家具") || r.layers.contains("VC_设备编号")));
}

#[test]
fn name_fallback_to_block() {
    let cfg = default_config();
    let mut stats = DrawingStats::default();
    stats.insert_records = vec![rec("$equip$777", "EQUIP-动力", "")];
    let rows = build_register(&stats, &cfg);
    assert!(rows.iter().any(|r| r.category == "动力" && r.name == "$equip$777"));
}

#[test]
fn real_sample_register_xlsx_roundtrip() {
    // 缺样本即失败（防静默跳过假绿）
    let p = std::env::var("CAD_FIXTURE_DWG").unwrap_or_else(|_| {
        panic!("缺少 CAD_FIXTURE_DWG：真实样图集成测试必须提供样本路径（AC1018 电气图）")
    });
    let stats = acad_mcp::drawing::summarize(std::path::Path::new(&p)).unwrap();
    let cfg = default_config();
    let rows = build_register(&stats, &cfg);
    assert!(!rows.is_empty());
    let total: u64 = rows.iter().map(|r| r.quantity).sum();
    eprintln!("资产行={} 设备总数={}", rows.len(), total);
    for cat in ["照明", "插座", "配电箱柜", "消防", "通讯智能化", "动力"] {
        assert!(rows.iter().any(|r| r.category == cat), "应含类别 {cat}");
    }
    let mut by_qty = rows.clone();
    by_qty.sort_by(|a, b| b.quantity.cmp(&a.quantity));
    eprintln!("=== 资产 Top-15 ===");
    for r in by_qty.iter().take(15) {
        eprintln!("  [{}] {} x{} ({})", r.category, r.name, r.quantity, r.layers);
    }

    // 写 xlsx + calamine 读回断言
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("assets.xlsx");
    write_xlsx(&rows, &out).unwrap();
    assert!(out.exists());
    use calamine::{open_workbook, DataType, Reader, Xlsx};
    let mut wb: Xlsx<_> = open_workbook(&out).unwrap();
    let range = wb.worksheet_range("资产登记表").unwrap();
    let h = range.get((0, 1)).and_then(|d| d.as_string()).unwrap_or_default();
    assert_eq!(h, "类别");
    let c1 = range.get((1, 1)).and_then(|d| d.as_string()).unwrap_or_default();
    assert!(!c1.is_empty());
    eprintln!(
        "xlsx 读回 OK：表头类别={} 首行类别={} 行数={}",
        h,
        c1,
        range.height()
    );
}
