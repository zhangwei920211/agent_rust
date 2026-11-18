//! DXF/DWG 引擎选型 spike — acadrust 原生 DWG 读取探针 v2。
//! v1 已证：AC1032 原生读取成功、<700ms、failsafe 恢复 4 条跳过记录。
//! v2 增：实体类型直方图 + 代理实体计数（P1-3）+ 图层/块名 CJK 校验（P2）。
//! 仅读取，不写任何源图。

use acadrust::io::dwg::dwg_reader::{DwgReadOptions, DwgReader};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

const DEFAULT_SAMPLE: &str =
    "D:/微信文件/xwechat_files/wxid_yzv5spffpwta22_f83f/msg/file/2026-03/雁能1#-电气施工图 - 无人防版.dwg";

fn has_cjk(s: &str) -> bool {
    s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let samples: Vec<String> = if args.is_empty() {
        vec![DEFAULT_SAMPLE.to_string()]
    } else {
        args
    };
    for path in &samples {
        println!("\n=== {} ===", path);
        let t0 = Instant::now();
        let reader = DwgReader::from_file_with_options(Path::new(path), DwgReadOptions::failsafe());
        let mut reader = match reader {
            Ok(r) => r,
            Err(e) => { println!("OPEN_ERROR: {:?}", e); continue; }
        };
        let outcome = match reader.read_with_stats() {
            Ok(o) => o,
            Err(e) => { println!("READ_ERROR: {:?}", e); continue; }
        };
        let s = &outcome.stats;
        println!("elapsed_ms: {}", t0.elapsed().as_millis());
        println!("source_version: {} | stream_completed: {} | usable: {}",
            s.source_version, s.stream_completed, s.has_usable_drawing_data());
        println!("source_records: {} | decoded: {} | skipped: {} | recovered_errors: {}",
            s.source_records, s.decoded_source_records, s.skipped_source_records, s.recovered_errors);
        println!("output_entities: {} | output_objects: {} | output_table_records: {}",
            s.output_entities, s.output_objects, s.output_table_records);

        let doc = &outcome.document;

        // 实体类型直方图（entity_type() 返回对象名，代理实体以 ACAD_PROXY_ENTITY 等出现）
        let mut hist: BTreeMap<String, u64> = BTreeMap::new();
        let mut total: u64 = 0;
        for e in doc.entities() {
            let name = e.as_entity().entity_type().to_string();
            *hist.entry(name).or_insert(0) += 1;
            total += 1;
        }
        println!("entities_iterated: {}", total);
        let mut sorted: Vec<(&String, &u64)> = hist.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        println!("-- entity_type histogram (top 20) --");
        for (k, v) in sorted.iter().take(20) {
            println!("  {} = {}", k, v);
        }
        let proxy: u64 = hist.iter()
            .filter(|(k, _)| k.to_uppercase().contains("PROXY"))
            .map(|(_, v)| *v).sum();
        println!("PROXY_TOTAL: {}", proxy);
        println!("-- proxy-related keys --");
        for (k, v) in hist.iter().filter(|(k, _)| k.to_uppercase().contains("PROXY")) {
            println!("  {} = {}", k, v);
        }

        // 图层名 CJK 校验
        let layer_names: Vec<&str> = doc.layers.names().collect();
        let layer_cjk = layer_names.iter().filter(|n| has_cjk(n)).count();
        println!("layers: {} | cjk_layers: {}", layer_names.len(), layer_cjk);
        println!("-- layer sample (first 15) --");
        for n in layer_names.iter().take(15) { println!("  L: {}", n); }

        // 块名 CJK 校验
        let block_names: Vec<&str> = doc.block_records.names().collect();
        let block_cjk = block_names.iter().filter(|n| has_cjk(n)).count();
        println!("block_records: {} | cjk_blocks: {}", block_names.len(), block_cjk);
        println!("-- block sample (first 15) --");
        for n in block_names.iter().take(15) { println!("  B: {}", n); }
    }
}
