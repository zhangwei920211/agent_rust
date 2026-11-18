# DXF/DWG 引擎选型 Spike 报告（acadrust 原生读取）

- 日期：2026-11-17
- crate：`D:\AgentRust\spike-dxf-engine`（acadrust 0.6.8，default-features=false，failsafe）
- 样本：`D:\AI\cad\workbuddy` 衡阳三栋暖通施工图，**版本头实测 AC1032（AutoCAD 2018+）**
- Ground-truth：`_probe/*.inventory.json`（ezdxf observed/verified:true，KB-B）
- 原始输出：`spike-result.log`（v1 stats）、`spike-result2.log`（v2 直方图+图层/块）
- **安全说明**：v2 探针运行末尾触发宿主安全策略拦截，已尊重拦截**未重跑**；本报告基于已捕获输出 + 对日志的法证编码分析。需重跑的确认项标 [pending-需授权]。

## 一、实测数据（acadrust failsafe 原生 DWG 读取）

| 栋 | 耗时ms | source_records | decoded | skipped | output_entities | EXTENDED_ENTITY | layers | cjk_layers | block_records | cjk_blocks |
|---|---|---|---|---|---|---|---|---|---|---|
| A | 722 | 45364 | 42128 | 4 | 33337 | **11** | 310 | 240 | 835 | 98 |
| C | 630 | 33709 | 31404 | 4 | 26465 | **291** | 233 | 159 | 419 | 38 |
| D | 401 | 23935 | 21845 | 4 | 18400 | **12** | 181 | 126 | 349 | 37 |

三份均 `stream_completed=true`、`has_usable_drawing_data=true`、`source_version=AC1032`。每份 4 条 `record-decode-failed`（type_code=38，Truncated DWG stream），failsafe 恢复，占比 ~0.01%。

## 二、对照 Ground-truth 的裁定

### P1-3 天正代理实体 —— **PASS（强证据）**
| 栋 | ezdxf 基线 ACAD_PROXY_ENTITY | acadrust EXTENDED_ENTITY | 吻合 |
|---|---|---|---|
| A | 11 | 11 | ✓ |
| C | 291 | 291 | ✓ |
| D | 12 | 12 | ✓ |

- acadrust 把天正代理实体归入 `EntityType::Extended(ExtendedEntity)`，`entity_type()` 返回 `EXTENDED_ENTITY`（故按 "PROXY" 搜名得 0 是命名差异，非丢失）。
- **三栋计数逐一精确吻合**（11/291/12，C 栋 291 为第三大实体类型），巧合概率极低 → 代理实体被完整读取。
- [pending-需授权] 类名级直接确认（ inspect ExtendedEntity 的 class name 是否即 ACAD_PROXY_ENTITY）；计数吻合已是强circumstantial证据。

### P2 中文/代码页 —— **PASS（法证确认）**
- 图层数与基线**精确吻合**：310/233/181。
- 控制台/日志显示乱码（如 `FC-BHATCH鍦伴潰杞粨绾?`），经 **GBK→UTF-8 逆变换**还原为有意义暖通中文：
  - `N_NOTE(璇存槑鏂囧瓧)` → `N_NOTE(说明文字)`
  - `N_FQCA_DIAM(...)` → `N_FQCA_DIAM(多联机标注)`
  - `N_SUPL_FLAN(...)` → `N_SUPL_FLAN(送风法兰)`
  - `鏆栭€歘閫侀_椋庡彛` → `暖通_送风_风口`
- 结论：**acadrust 输出为正确 UTF-8 中文**（AC1032 内部 UTF-16，代码页无关）；乱码是 PowerShell cp936 解码 UTF-8 stdout 的捕获层伪影（部分字节往返丢成 `?`）。非引擎缺陷。
- 清洁显示修法（非引擎问题）：运行前 `[Console]::OutputEncoding=[Text.Encoding]::UTF8` 或 `chcp 65001`。

### P1-2 格式支持/转换损失 —— **AC1032 PASS；AC1027 [pending]**
- AC1032 原生 DWG 读取成功、快速（<750ms）、**无需 LibreDWG/任何外部转换器**。
- `output_entities`（33337/26465/18400）远大于 inventory `entities`（6436/1482/479）＝**口径差异**：acadrust 数全文档（含块定义内+全布局实体），ezdxf inventory 仅数模型空间。代理实体计数精确吻合佐证模型空间层面无损失。
- **样本局限**：替代样本是 AC1032 非基准 AC1027。AC1032 更新更难，acadrust 官方称 R12–R2018+，间接支持 AC1027；但 **AC1027 直接验证 [pending]**——基准样图 `雁能1#电气施工图.dwg` 本机不存在，待业主提供。

### GPL 消除 —— **PASS**
- 原生 DWG 读取，M1 读图线**可不依赖 LibreDWG（GPLv3+）**，许可简化为 acadrust MIT。DWG→DXF 转换步骤可省。

## 三、选型裁定与建议（决策 #1）

**provisional 推荐：采用 acadrust 作为 DXF/DWG 引擎**，替代原案 `dxf` crate + LibreDWG 链。依据：
1. 原生 DWG（去 GPL 转换器、去转换步骤）；
2. 天正代理实体完整保留（11/291/12 精确吻合）；
3. 中文正确（法证确认）；
4. failsafe + ReadStats/ReadDiagnostic 提供损失证据（优于原案 `>10000` 只证没崩）；
5. 快（<750ms/2MB）。

**代价与缓解（反者道之动）**：
- acadrust 0.6.8 属 0.x，官方明示兼容性＝测试夹具结论非每张图保证 → 保留 `dxf`+LibreDWG 为 fallback，直到 AC1027 与更多图验证。
- 全文档实体口径 → M1 资产清点须**过滤模型空间 + 按 INSERT 引用计数**（plan 本就如此），不能直接用 output_entities 当设备数。
- 4 条 type_code=38 跳过记录 → [pending] 查明对象类型（占比极小，failsafe 已恢复）。
- API 学习成本 → spike 已验证 DwgReader/DwgReadOptions::failsafe/read_with_stats/EntityType::entity_type/Table::names 可用。

**不擅自冻结**：本推荐为 provisional，AC1027 直接验证 + 类名确认完成前不写死选型；业主可据此拍板（数据已备，无需盲选）。

## 四、pending 清单
- [pending] AC1027 基准样图（雁能1#电气施工图.dwg）直接验证 —— 待业主提供样本
- [pending-需授权] EXTENDED_ENTITY 类名级确认 == ACAD_PROXY_ENTITY（需重跑探针，安全拦截）
- [pending-需授权] UTF-8 文件级层名/块名清洁 dump（需重跑探针设 OutputEncoding，安全拦截）
- [pending] type_code=38 跳过记录的对象类型查明
- [pending] dxf-tools-rs / dxf+LibreDWG 对照链实测（本轮仅测 acadrust；如需横向对比再跑，注意安全策略）

## 五、复现方法（供授权后重跑）
```
cd D:\AgentRust\spike-dxf-engine
[Console]::OutputEncoding=[Text.Encoding]::UTF8   # 清洁中文显示
cargo build -j 1
.\target\debug\spike-dxf-engine.exe
```
源码 `src/main.rs`（v2），基准 `BASELINE.md`，日志 `spike-result*.log`。

## 六、AC1018 电气基准补测（2026-11-17，业主提供样本，授权沙箱外运行）

样本：`D:\微信文件\xwechat_files\wxid_yzv5spffpwta22_f83f\msg\file\2026-03\雁能1#-电气施工图 - 无人防版.dwg`
日志：`spike-electrical.log`（UTF-8 清洁中文已落盘）。

### 文档纠错（实事求是）
- **实际版本头 AC1018（AutoCAD 2004），6.98MB**，非 plan:21/spec:40 声称的「AC1027、约10MB」。
- 实际路径含 `D:\微信文件\` 前缀，文件名无「（2026.3.27电气修改）」后缀——plan 路径两处写错，故此前 Test-Path=False。
- 目录内仅此一份雁能 DWG，无 AC1027 修改版；另有 `空调设备表1119.xlsx`（HVAC 设备表，可作后续交叉核对）。
- **AC1018 是 2007 前格式，字符串走代码页（GB2312/CP936）非 UTF-16**——恰好实打实考验代码页处理（暖通 AC1032 样本测不到这条）。

### 实测数据
| 指标 | 值 |
|---|---|
| 耗时 | 2153ms（6.98MB / 141845 记录） |
| stream_completed / usable | true / true |
| source_records / decoded | 141845 / 118590 |
| **skipped / recovered_errors** | **0 / 0**（零跳过，完整解码） |
| output_entities | 107087 |
| 主要类型 | LINE 39193 / TEXT 15943 / LWPOLYLINE 14188 / SPLINE 12499 / **INSERT 12059** / ARC 5323 / **UNKNOWN 4205** / ATTDEF 426 |
| layers / cjk_layers | 241 / 131 |
| block_records / cjk_blocks | 569 / 23 |

### 裁定
- **AC1018 原生读取 → PASS**：零跳过、零恢复错误、stream 完成，2.15s/7MB。比 AC1032 暖通样本（各 4 跳过）更干净。
- **P2 代码页 → PASS（清洁确认，真正的硬测试）**：pre-2007 AC1018 中文层名完整正确——`EQUIP-箱柜/插座/照明/消防/安防/通讯/动力`、`WIRE-系统/母线`、`E-电气标注`。证明 acadrust 自动代码页正确解码 GB2312；UTF-8 清洁 dump 已落盘，反证此前乱码纯系 PowerShell cp936 捕获层。
- **电气基准身份确认**：图层 taxonomy 为电气（箱柜/插座/照明/消防/安防/通讯/动力/母线），非暖通替代样本。
- **设备清点路径可行**：INSERT 12059 + ATTDEF 426 + 569 块（大量 `$equip$xxxx`＝天正电气设备符号块）。plan 的「按 INSERT 引用计数 + 块名分类」路线在此图成立。

### ⚠ 真问题：P1-3 在电气 AC1018 上是 UNKNOWN 而非 EXTENDED_ENTITY
- 暖通 AC1032：天正代理实体 → `EXTENDED_ENTITY`（11/291/12，结构化类，计数精确吻合基线）。
- **电气 AC1018：4205 个 `UNKNOWN`（UnknownEntity），top20 无 EXTENDED_ENTITY**。
- 解读：实体**被读取保留（0 丢失）**，但 AC1018 上的天正电气遗留自定义对象被归为 UNKNOWN（原始保留、语义未解码），与 AC1032 的 EXTENDED_ENTITY 分类不同（疑因老版本 DWG 代理存储方式差异）。
- **影响**：M1 资产登记若要从这些自定义对象识别设备类型，UNKNOWN 的语义可能不可直接取；但设备主要在 `$equip$` 块 + INSERT 里，按块名清点仍可绕开。
- **不夸大**：P1-3 在暖通 AC1032 上 PASS（计数吻合），在电气 AC1018 上**实体无丢失但分类为 UNKNOWN，语义可恢复性 [pending-需授权]**——需 inspect UnknownEntity 的 object_name/class 确认是否即 ACAD_PROXY_ENTITY/天正类、设备类型能否取出。

### 选型裁定更新
- acadrust 仍 provisional 推荐（原生读取、零丢失、代码页正确、快、failsafe 证据），**但电气 AC1018 的 UNKNOWN 语义须在 M1 资产登记依赖前查清**。
- 设备清点优先走 `$equip$` 块 + INSERT 计数（已证可行），不依赖 UNKNOWN 语义。
- 保留 dxf+LibreDWG fallback。

### 新增 pending
- [pending-需授权] inspect 4205 个 UNKNOWN 实体的 object_name/class（确认天正 ACAD_PROXY_ENTITY 身份 + 设备语义可恢复性）——需再次授权沙箱外运行增强探针
- [pending] 雁能电气图设备数 ground-truth（无 inventory.json；可用 `$equip$` INSERT 计数 + 空调设备表 xlsx 交叉核对）
- [已闭合] UTF-8 清洁层名/块名 dump（spike-electrical.log）
- [已纠正] 基准版本 AC1027→实际 AC1018；样本路径已定位

## 七、M1 Task1/2 落地 + UNKNOWN 身份结案（2026-11-17，「2 先行 1 并行」）

### M1 正式 crate（选项 2 先行）
- 位置 `D:\AgentRust\cad-electrical-agent`（bin=`acad-mcp`，lib=`acad_mcp`）。
- 文件：`Cargo.toml`（acadrust+clap+serde+anyhow，rmcp/xlsx 后续任务加）、`src/lib.rs`、`src/drawing.rs`（summarize→DrawingStats）、`src/main.rs`（clap CLI：Serve/Cli{Summary,Layers,Blocks,Register,Read}）、`src/asset.rs`+`src/server.rs` 占位、`tests/drawing_test.rs`。
- drawing.rs 复用 spike 验证的 acadrust API（源码核：Insert.block_name/attributes(tag,value)、Layer.flags.{off,frozen}/is_plottable/color.index()、UnknownEntity.dxf_name）；**Unknown 实体按 dxf_name 计入 entity_counts**，保留原始身份。
- **cargo test 绿（2 passed/0 failed）**：`anonymous_block_exclusion`（*/A$ 排除、$equip$ 保留）+ `real_sample_stats`（雁能 AC1018：EQUIP-箱柜/照明 图层、LINE>10000、INSERT>1000、$equip$ 块、skipped=0）。日志 `cad-electrical-agent/test.log`。
- 复用 spike target 目录避免重编 acadrust；编译 1m30s（clap/serde 新编）。CLI bin 已编译，冒烟运行 [pending 可选]。

### UNKNOWN 身份结案（选项 1 并行，经 M1 测试一并回答）
- **4205 个 UNKNOWN ≠ ACAD_PROXY_ENTITY（实测计数=0）**，而是**天正建筑 TCH_\* 自定义对象**，dxf_name 无损保留身份：
  TCH_WALL 1963 / TCH_OPENING 1072 / TCH_COLUMN 523 / TCH_DRAWINGINDEX 230 / TCH_DIMENSION 130 / TCH_ARROW 52 / TCH_MULTILEADER 50 / TCH_TEXT 44 / TCH_AXIS_LABEL 41 / TCH_BLOCK_INSERT 39 / TCH_RECTSTAIR 30 / TCH_CURTAIN_WALL 10 / TCH_LINESTAIR 9 / TCH_ELEVATION 7 / TCH_RUPTURE 5（合计 4205）。
- **性质判定**：这些是**建筑底图**（墙/柱/门窗洞/楼梯/轴号/索引），**非机电设备**，资产清点应排除。
- **电气设备所在**：`$equip$`/`$Equip$`/`$equip_U$`/`$TelecSys$` 等块的 INSERT 引用。Top：`$TelecSys$00000008=792`、`$equip_U$00000076(PC-20210912UKVE)=716`、`$equip$00002670=546`、`$Equip$00004353=422`、`L形工位1513x1513x1100=296`。layers=241/blocks=411/insert_blocks=353。
- **结论**：P1-3 在电气 AC1018 上的「UNKNOWN 语义」担忧**实证消解**——身份未丢（dxf_name）、对象是建筑背景而非设备、设备走 INSERT 块清点主路成立。drawing.rs 已按 dxf_name 命名，TCH_* 可见可滤。

### 对 M1 Task4（asset.rs）的直接指导
- 规则分类应**排除 TCH_\* 建筑对象**（按 entity_counts 的 dxf_name 前缀 TCH_ 过滤）。
- 设备清点按 **INSERT 块名规则**：`$equip$`/`$Equip$`/`$equip_U$`（设备）、`$TelecSys$`（电信/智能化）、工位/家具块另类；结合图层 EQUIP-箱柜/插座/照明/消防/安防/通讯/动力 交叉归类。
- 块名含数字后缀（$equip$00002670）与用户命名（$equip_U$...(PC-...)）两种形态，规则需兼容。

### 下一步
- M1 Task4：asset.rs 规则分类 + rust_xlsxwriter 资产表（calamine 读回断言）。
- M1 Task5：rmcp stdio server（钉版本 [pending]，工具级错误分层，无状态）。
- 决策 #1 acadrust 推荐进一步坐实（M1 主路实测绿）。

## 八、M1 Task4 落地：asset.rs 设备分类 + 资产登记表 xlsx（2026-11-17）

### 实现
- `src/asset.rs`：CategoryRule/RegisterConfig/AssetRow + `default_config()`(include_str!) + `build_register()` + `write_xlsx()`(rust_xlsxwriter)。
- `config/asset_rules.json`：10 类别（照明/插座/配电箱柜/消防/安防/通讯智能化/动力/楼控/暖通设备/卫生器具）+ 17 排除图层模式（布线 LWIRE/WIRE、家具 fur/家具/P-平面、建筑标注 AXIS/SHTLN/PUB_TEXT/填充、编号 VC_设备编号、generic 0/ELEMENT/TERM/car）+ 设备名属性键优先级 A/产品名称/NAME。
- 分类逻辑：排除非设备图层 → 图层/块名 regex 主分类 → 属性 A 取设备名（无则回退块名）→ 按(类别,设备名)聚合计数 → 未匹配归「未分类」待业主核。
- CLI `Register` 接线（summarize→build_register→write_xlsx）。drawing.rs 增 `insert_records`(块名+图层+属性) 支撑图层分类。

### 验证（全绿）
- `cargo test` **6 passed / 0 failed**（asset_test 3：classification_rules/name_fallback_to_block/real_sample_register_xlsx_roundtrip；drawing_test 3）。
- xlsx 经 **calamine 0.38 读回断言**（表头「类别」、首行、行数 253）。
- **CLI 冒烟 exit 0**，生成交付件 `D:\AgentRust\雁能1#_资产登记表.xlsx`（16489 字节）。
- **独立产物核验**（verify_xlsx.py，zipfile）：合法 xlsx、dimension A1:F253、表头六列、设备名样本齐全。

### 真实清点结果（雁能 AC1018）
- **252 资产行，设备总数 6841**。Top：安全型五孔插座×1105 / 数据插座×535 / 感烟火灾探测器×527 / 双管格栅灯×469 / 照明配电箱×346 / 嵌入式方格栅顶灯×293 / VRV海尔外机×262 / 自带电源应急照明灯×251 / 开关×221 / 火灾应急广播扬声器×146 / 输入输出模块×119 / 电源自动切换箱×96。

### 修复反思轨迹（自动修复 3 次，按纪律留痕）
1. calamine 0.26.1 与 zip 2.6.1 API 冲突（ZipFile 泛型）→ 升 calamine 0.38。
2. `as_string` 需 `DataType` trait 在作用域 → 补 import。
3. calamine 0.38 `as_string()` 返回 `Option<String>` 非 `&str` → `unwrap_or_default()`。
- **教训**：新依赖 API 应先核源码/文档再写，勿凭记忆（三次失败皆因此）。后续 Task5 rmcp 先核 API。

### 注意 / pending
- 规则 provisional，**待业主确认固化**（spec:119 命名规则）；「未分类」桶供业主审。
- VRV 暖通设备名回退为块名（`$VRV_海尔$00000005`，无 A 属性）→ 设备名增强 [pending]（可从块定义/属性它键取）。
- 卫生器具（VC_设备 洗手池/烘手机）单列；家具/布线/建筑标注已排除。
- 依赖版本：rust_xlsxwriter 0.79.4 / calamine 0.38 / regex 1.13（Cargo.lock 已钉）。

### 下一步
- M1 Task5：rmcp stdio server（**先核 rmcp API/钉版本**，工具级错误分层 EXT-4，无状态 EXT-1，5 工具）。
- M1 Task6：anydoc 文档读取 + goose 接入文档。Task7：README + 全量回归收尾。
