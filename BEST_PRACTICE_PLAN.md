# CAD 电气深化设计 Agent — 最佳实践计划（来源标注版）

- 编制日期：2026-11-17
- 适用项目：`D:\AgentRust\superpowers`（spec 125 行 / plan 301 行）+ 上午审查报告 `ARCHITECTURE_REVIEW.md`
- 方法：先查 D:/pi 原库（slice_retrieve，session `arch-audit-20261117`，4 步 / 80% 水位收口）→ 缺口联网（WebSearch 发现 + WebFetch 官方原文核验）→ 每条步骤挂来源
- 来源标签：`[KB-x]` = 原库切片（路径#行）；`[EXT-x]` = 外部官方一手（URL，核验 2026-11-17）；`[二手]` = 转述，仅作线索不作依据；`[pending]` = 未闭合

---

## 一、目标与适用范围

**目标**：把「goose 原装 + 自研 Rust MCP（acad-mcp）+ 规范网关 + 单安装包」这条已定案路线，按 2026 年最新官方规范与权威实践校准后落地，消除上午审查的 P0/P1 缺陷，产出可复核、可验收的分阶段执行清单。

**适用范围**：
- 仅 Windows（目标机 Win10 19044）；仅业主本人使用、不分发 [KB/spec:120]
- 覆盖 M1（DXF 读图清点）→ M2（规范网关+skills）→ M3（打包发行）→ M4（COM 联调）→ 验收
- **不含**：结构/给排水/暖通深化、BIM 三维、多平台、goose 内核改造 [spec:25]

**不在本计划内擅自决定**（需业主拍板，见 §五）：双 COM 通道主从、DXF 引擎是否换型、goose provider 选型。

---

## 二、证据基础（分级）

### 原库（D:/pi，已直接核验切片）
| 标签 | 路径#行 | 关键事实 |
|---|---|---|
| KB-A | `projects/hvac-cad-electrical/gbt-16895-15-2023-low-voltage-wiring-systems.md#L1-69` | draft/verified:false；2023 版 openstd 未命中、全文 pending；现行采标版 GB/T 16895.6-2014 |
| KB-B | `projects/hvac-cad-electrical/hengyang-three-building-hvac-electrical-baseline-2026-11-08.md#L1-38` | observed/verified:true；accoreconsole→DXF→ezdxf 清点；9 台 VRV 外机/197 室内机（真实图纸清点基线） |
| KB-C | `projects/ai-guide/.../mcp-registry-alignment-audit-2026-11-12.md#L43-59,#L80-88` | 版本钉死；serverInfo≠包版本；tools/list 握手验证；漂移门禁；registry 不可达≠包不存在 |
| KB-D | `projects/workbuddy-governance/mao-kb-mcp-connection-closed-2026-11-06.md#L11-29` | stdio server exe 缺失→spawn 失败→connection closed；诊断须读完整错误（mcp list --json 两行） |
| KB-E | `projects/ai-guide/.../model-context-protocol-...part3.md#L54-93` | **[二手 2025-03 转述]** 工具描述至关重要（模型靠描述选工具）；非 Claude 模型 MCP 效果难保证 → 需官方/实测复核 |
| KB-F | `projects/workbuddy-governance/mcp-config-governance-2026-10-29.md#L13-19` | 官方最佳实践＝描述清楚/凭据安全/状态灯（WorkBuddy host 类比，弱相关） |

### 外部官方一手（核验 2026-11-17）
| 标签 | URL | 关键事实 |
|---|---|---|
| EXT-1 | modelcontextprotocol.io/specification/2026-07-28/basic | **无状态**：server MUST NOT 依赖先前请求；跨调用状态用显式 handle；**stdio 进程≠会话**；JSON Schema 2020-12 默认；$ref 禁自动网络解引用；组合关键字深度/广度设界（DoS）；**STDIO 凭据从环境取**，不走 HTTP auth；每请求 _meta 带 protocolVersion；serverInfo 自报、不得用于安全决策 |
| EXT-2 | modelcontextprotocol.io/specification/2026-07-28/basic/patterns | **MRTR**：server 回 `InputRequiredResult`，client 带 `inputResponses` 重试；server MUST NOT 主动发请求；elicitation/sampling/roots 走此模式 |
| EXT-3 | github.com/block/goose/blob/main/CUSTOM_DISTROS.md | 三档定制（config-only/extension-based/deep）；捆绑扩展＝config.yaml extensions + built-in-extensions.json + bundled-extensions.json；init-config.yaml 首跑应用；凭据走 keyring 或 secrets.yaml（GOOSE_DISABLE_KEYRING=1）+ `goose configure set-secret`；`GOOSE_DISABLE_TELEMETRY=1`；Apache-2.0 合规（附 LICENSE/NOTICE、声明修改、不用 Goose 商标背书）；recipes（YAML）编排工作流；goose serve ACP；配置优先级 env→config.yaml→默认 |
| EXT-4 | github.com/modelcontextprotocol/rust-sdk（README raw） | 实现 **2026-07-28** 且兼容 2025-11-25；`cargo add rmcp --features server`；stdio 用 transport-io；纯工具用 `#[tool_router(server_handler)]`，多能力用 `#[tool_handler]`；**schemars 从字段文档派生 schema**（类型名/类型文档被忽略）；SEP-2106 outputSchema 可任意 JSON 类型；**工具级错误 `Ok(CallToolResult::error)` 用户可见 vs 协议级 `Err(McpError)` 用户不可见**；ClientLifecycleMode Auto 探测 10s 超时；3.x 破坏性变更见 migration guide |
| EXT-5 | github.com/modelcontextprotocol/rust-sdk（LICENSE raw） | **MIT→Apache-2.0 过渡混合**：新贡献 Apache-2.0，未授权重许可的旧贡献仍 MIT，文档（除规范）CC-BY-4.0；版权 LF Projects LLC |
| EXT-6 | github.com/modelcontextprotocol/rust-sdk（releases/提交） | 最新发布 **v3.10.0（2026-11-12）**；README 未印版本号，确切版本以 crates.io 为准 [pending 钉版本] |
| EXT-7 | docs.rs/acadrust | DXF/DWG **R12–R2018+，AC1027 读/写**；48 实体类型+ACIS；**failsafe** `read_with_stats`→ReadStats/ReadDiagnostic（output_entities 计数）；**自动代码页**处理旧图；Rayon 并行可限流；**ProxyGraphics/ProxyGraphicRecord/ProxyUnicodeText**（代理实体）；audit/RepairAction 结构审计；SemanticInventoryV1；v0.6.8；官方声明「兼容性是测试夹具结论，非每张图保证」 |
| EXT-8 | crates.io/crates/dxf-tools-rs · docs.rs/dxf-tools-rs | AC1012–AC1032、30+ 实体、zero-copy、并行、v0.1.3（**候选，未深核** [pending]） |
| EXT-9 | 上午审查 §3（官方核验） | LibreDWG **GPLv3+**、约 90% 覆盖、dwg2dxf AC1027 限定；`dxf` crate 无 AC1027 承诺；GB 50168-2018/GB 51348-2019/JGJ 242-2011 限定（部分强条被替代）；GB/T 16895.15-2023 **反驳** |

> 二手线索（不作依据）：developersdigest.tech goose MCP 指南（Windows 配置路径 `%APPDATA%\Block\goose`、支持 stdio+Streamable HTTP 不支持 SSE、sampling 已移除）；gitcode/csdn/importstatic 博客。均需官方复核。

---

## 三、分阶段执行清单

> 纪律：每步先写失败测试再实现（superpowers TDD）；每步有「怎么知道做完了」；YAGNI/DRY；结论带证据。来源标签见 §二。

### Phase 0 — 选型复核与前置闭合（开工前，阻塞后续）

- [ ] **0.1 DXF 引擎选型 spike（reuse-first 复核，不盲换）**
  - 执行要点：在雁能1#（AC1027、天正代理实体、路径含 `#`/空格）上并行实测三条链：①原案 `dxf` crate + LibreDWG dwg2dxf；②`acadrust` 原生 DWG/DXF 读取；③`dxf-tools-rs`。记录：实体总数、与图纸台账比对的**缺失率**、代理实体识别数、中文图层/块名正确性、耗时/内存。
  - 判据：缺失率超阈值或代理实体丢失即判该链不合格；acadrust 用 `read_with_stats` 取 `output_entities`+`diagnostics` 作为损失证据，而非只看「没崩」。
  - 依据：[EXT-7]（failsafe/代理实体/代码页/AC1027）、[EXT-8]、[EXT-9]（dxf 无 AC1027 承诺、LibreDWG 90%）、[KB-B]（真实清点基线）、上午 P1-2/P1-3/P2
  - 验证：spike 报告落 `tests/fixtures/README.md`，含三链对比表 + 选定理由；**选型变更须业主确认**（§五）
  - 注意：acadrust/dxf-tools-rs 均 0.x，官方明示兼容性非保证 [EXT-7]；若 acadrust 原生 DWG 达标，可**去掉 GPL 的 LibreDWG 依赖**（许可简化），但必须以实测为准 [pending]

- [ ] **0.2 标准版本闭合（P0-1）**
  - 执行要点：GB/T 16895.15-2023 取正式文本核条款；未闭合则缓存按「GB/T 16895.6-2014 现行 + 2023 pending」双记录，禁止把 2023 标「已核实现行」。
  - 依据：[KB-A]、[EXT-9]（反驳）
  - 验证：缓存条目带 `verified` 布尔 + 来源 URL + 核验日期；2023 条目 `verified:false`

- [ ] **0.3 规范核验源归口契约（P0-2）**
  - 执行要点：按标准类型分派核验源——工程建设国标（GB 50168/51348）走住建部发布渠道；行业标准（JGJ 242）走行业主管渠道；GB/T 推荐性走 openstd/全国标准信息平台。**「未命中」≠「不现行」**，单源零命中不得直接阻断。
  - 依据：[EXT-9]、[KB-A]、spec:20/103/115
  - 验证：契约文档列出每本标准的归口源 + 判定矩阵；S6/S7 依赖标准在离线首装下不被误杀

- [ ] **0.4 环境前置**
  - 执行要点：确认仓库根盘符（plan:15 写 `G:\cad agent`，本次落地 D 盘）；确认正版 AutoCAD 安装计划（COM 线阻塞项，DXF 线不受阻）。
  - 依据：plan:15/21、spec:37/108
  - 验证：`git init` 目标盘可写；AutoCAD 状态记录在案

### Phase 1 — M1 DXF 读图与资产登记管线

- [ ] **1.1 crate 脚手架 + 版本钉死**
  - 执行要点：单 crate `acad-mcp`（lib+bin）；`Cargo.lock` 提交、依赖钉版本（rmcp 确切版本以 crates.io 核 [EXT-6 pending]）；核 rmcp MSRV 与本机 toolchain（3.x 有破坏性变更，读 migration guide）。
  - 依据：plan:5-9、[EXT-4]（3.x 迁移）、[EXT-6]、[KB-C]（版本钉死/漂移门禁）
  - 验证：`cargo build` 通过；`cargo tree` 无意外版本漂移；记录 MSRV

- [ ] **1.2 DXF 统计核心（drawing.rs）**
  - 执行要点：按 0.1 选定引擎实现 `summarize`；区分块定义数与 INSERT 引用数；匿名块 `*`/`A$` 排除；**代理实体单独计数并降级读取**（天正）；迭代器遍历禁 collect 全实体；大图纸用并行限流（Rayon pool install）防 OOM。
  - 依据：plan:112-122/137、[EXT-7]（ProxyGraphics/并行限流/failsafe）、[KB-B]、上午 P1-3
  - 验证：手写 fixture 精确断言（insert_counts["风机-1"]==3、匿名块排除、CJK 块名）；**新增 GB2312 编码 fixture**（spec:38）；样图集成断言实体数 + 缺失率 + 代理实体数

- [ ] **1.3 资产登记 + xlsx（asset.rs）**
  - 执行要点：规则分类（regex 不分大小写）→ `rust_xlsxwriter` 生成资产表；表头中文；`calamine` 读回断言；规则配置 `asset_rules.json` 内置（include_str!）。
  - 依据：plan:169-203、spec:14
  - 验证：`cargo test --test asset_test` 全绿；样图全链人工抽查 Top-10 块名分类，结论写 README

- [ ] **1.4 MCP 工具面（server.rs，无状态 + 错误分层）**
  - 执行要点：`#[tool_router(server_handler)]` 暴露 5 工具；**工具无状态**，跨调用状态用显式 handle（如 drawing_id），不依赖 stdio 进程生命周期；schemars **字段文档**写清参数语义（模型靠描述选工具）；**业务错误走工具级 `Ok(CallToolResult::error)`**（用户可见），仅协议异常用 `Err(McpError)`。
  - 依据：[EXT-1]（无状态/handle）、[EXT-4]（tool_router/schemars 字段文档/错误分层）、[KB-E]（工具描述至关重要，二手待实测）、plan:216-227
  - 验证：rmcp 进程内 client/server 对测试，`tools/list` 列出 5 工具、`tools/call` 成功；断言错误信息用户可见

- [ ] **1.5 CLI 直调 + 路径健壮性**
  - 执行要点：clap `cli` 子命令与 MCP 工具共用同一 ops 实现；任意 Windows 路径（空格/中文/`#`/括号）可用。
  - 依据：plan:48-75/165/230
  - 验证：复制到含空格+中文+`#` 临时路径再跑，同 PASS

- [ ] **1.6 anydoc 文档读取 + goose 接入文档**
  - 执行要点：anydoc 内嵌读 docx/pdf/xlsx→Markdown；`docs/goose-setup.md` + 配置示例**标注「按核实时的 goose 版本填充」**（schema 未冻结）。
  - 依据：plan:257-281、spec:96、[EXT-3]
  - 验证：`cargo run -- cli read <xlsx>` 输出含中文 Markdown

### Phase 2 — M2 规范网关 + skills

- [ ] **2.1 规范网关（多源核验 + 工具级阻断）**
  - 执行要点：按 0.3 归口契约核验现行性；缓存带版本戳+verified+来源；**阻断时返回工具级错误并写明「缺哪本标准/哪个源未命中」**（不用协议级错误，否则用户看不到）；区分「未命中」与「不现行」。
  - 依据：spec:20/86-88/102/115、[EXT-4]（错误分层）、[EXT-9]、[KB-A]、上午 P0-1/P0-2
  - 验证：离线首装下 S6/S7 依赖标准不被误杀；缺标准时错误信息含标准号

- [ ] **2.2 KB 适配器（业主自建规范库）**
  - 执行要点：本地目录契约 `标准号/版本/内容文件 + 索引清单`，兼容 md/txt/pdf；openstd 仅核现行性，不作全文源。
  - 依据：spec:115、[KB-A]
  - 验证：适配器读业主库 + 现行性核验分离测试

- [ ] **2.3 半自动确认（S6/S7，MRTR/elicitation）**
  - 执行要点：「提案→确认→成图」用官方 **MRTR**：工具回 `InputRequiredResult` 请求 elicitation，goose client 带 `inputResponses`（用户确认/拒绝）重试；**不自造确认协议**。先验证 goose 客户端对 2026-07-28 MRTR/elicitation 的支持，不支持则回退 legacy elicitation/create。
  - 依据：[EXT-2]（MRTR）、[EXT-3]（goose elicitation）、spec:117（业主选 C 半自动）、[pending goose 兼容性]
  - 验证：goose 实测一次「提案→用户确认→成图」闭环；记录所用协议版本

- [ ] **2.4 电气 skills（中文）**
  - 执行要点：工作流编排（S1-S7）、图层规范、命名规范、配电计算（数值一律经网关取）、桥架规范（GB 50168 条款映射）、符号块库；用 goose **recipes**（YAML）编排，支持 sub-recipes。
  - 依据：spec:71-77、[EXT-3]（recipes）、[KB-B]（命名/清点基线）
  - 验证：recipe 跑通 S2 审图建档；配电数值来源可追溯

### Phase 3 — M3 打包发行（goose 零 fork 路线）

- [ ] **3.1 零 fork 定制（configuration-only + extension-based）**
  - 执行要点：acad-mcp 作为**外部 stdio 扩展**经 config.yaml `extensions` 注册；用 init-config.yaml 首跑预配置；recipes/skills 随包。**不用 built-in-extensions.json**（那需要 desktop 重编译＝fork，违背 spec:22/49 零 fork）。
  - 依据：[EXT-3]（三档定制/扩展注册/init-config）、spec:22/49
  - 验证：全新机器首跑自动注册 acad-mcp，`tools/list` 握手成功

- [ ] **3.2 凭据与隐私**
  - 执行要点：provider key 走 keyring 或 secrets.yaml（`goose configure set-secret`），**禁明文 config.yaml**；acad-mcp stdio 凭据从环境取；`GOOSE_DISABLE_TELEMETRY=1`（业主图纸敏感）。
  - 依据：[EXT-3]（keyring/set-secret/遥测）、[EXT-1]（STDIO 凭据从环境）、[KB-F]（凭据安全）
  - 验证：config.yaml 无明文 key；遥测关闭确认

- [ ] **3.3 二进制存在性 + 握手验证（防 connection closed）**
  - 执行要点：打包后确保 acad-mcp.exe 在配置路径/PATH；首装跑 `tools/list` 握手断言；spawn 失败时读**完整错误**（不截断第一行）；serverInfo 版本≠包版本，单独校验。
  - 依据：[KB-D]（exe 缺失→spawn 失败/完整错误）、[KB-C]（握手/serverInfo≠包版本）
  - 验证：故意移走 exe 复现 connection closed，诊断能定位；恢复后握手通过

- [ ] **3.4 许可合规**
  - 执行要点：附 goose Apache-2.0 LICENSE/NOTICE + 修改声明、不用 Goose 商标背书；rmcp **MIT/Apache-2.0 混合**（附两份许可文本+版权）；若 0.1 选 acadrust（MIT）则简化；若仍用 LibreDWG（GPLv3+）保持外部进程调用边界。**自用无分发义务**，但 LICENSE 齐备以备将来。
  - 依据：[EXT-3]（Apache-2.0 合规）、[EXT-5]（rmcp 混合许可）、[EXT-9]（LibreDWG GPLv3+）、spec:120
  - 验证：安装包内 LICENSE/NOTICE 清单核对

- [ ] **3.5 安装包组装**
  - 执行要点：Inno Setup exe + portable zip；python-build-standalone + autocad-mcp-pro 离线 wheels（TOOL_PACKS 裁剪 token）；预置规范基线缓存（按 0.2/0.3 闭合后的版本）。
  - 依据：spec:61/69/103、plan:9
  - 验证：离线全新机装完即用；基线缓存版本戳 verified 状态正确

### Phase 4 — M4 COM 联调（依赖正版 AutoCAD）

- [ ] **4.1 双 COM 通道裁决（先拍板再动手）**
  - 执行要点：明确 Python autocad-mcp-pro 与自研 Rust COM 桥的主从/边界/冲突裁决（上午 P1-1 主要矛盾）；建议窄口 COM 归一方，另一方只留 DXF+网关。**业主拍板后**再实现。
  - 依据：spec:52/64、上午 P1-1、[pending 业主决策]
  - 验证：职责边界文档 + 无重叠能力清单

- [ ] **4.2 无头批处理路线切换**
  - 执行要点：正版完整版（2024/2025）自带 accoreconsole → 无头批处理 accoreconsole 优先，COM 后台实例兜底；GB2312↔UTF-8 转换统一在桥接层。
  - 依据：spec:37/38/101
  - 验证：accoreconsole 跑通样图；编码转换断言

- [ ] **4.3 副本操作 + 出图**
  - 执行要点：严守反目标——只在副本上动手，原图存档；净化/画图/出图经 COM。
  - 依据：spec:30、plan:20
  - 验证：原图哈希不变；副本产出可核对

### Phase 5 — 验收收尾

- [ ] **5.1 全量回归**：`cargo test` + `clippy -D warnings` + `fmt --check` 全过；样图集成（设 CAD_FIXTURE_DWG，未设硬失败）。依据 plan:289-300。
- [ ] **5.2 clone 级验证**：按 README 从零走一遍（转换样图→CLI register→xlsx）。依据 plan:290。
- [ ] **5.3 证据归档**：spike 报告、标准核验记录、握手日志、许可清单入库存档；图谱 `--update` 增量刷新。依据 [KB-C]、graphify。

---

## 四、注意事项与风险（分级）

| 级别 | 风险 | 依据 | 缓解 |
|---|---|---|---|
| **P0** | GB/T 16895.15-2023「现行版」断言被反证，预置缓存带未核实戳 | [KB-A][EXT-9] | 0.2 双记录+pending，取正式文本闭合 |
| **P0** | openstd 单源核验误杀 S6/S7（工程建设国标/行标不纳入公开系统） | [EXT-9][KB-A] | 0.3 归口多源，未命中≠不现行 |
| **P1** | 双 COM 通道职责重叠无裁决（主要矛盾） | spec:52/64 | 4.1 业主拍板主从边界 |
| **P1** | acadrust/dxf-tools-rs 0.x 成熟度，兼容性非每张图保证 | [EXT-7][EXT-8] | 0.1 真实样图 spike，不盲换 |
| **P1** | goose 客户端对 2026-07-28 MRTR/elicitation 支持未验证 | [EXT-2][EXT-3] | 2.3 先实测，回退 legacy |
| **P1** | rmcp 3.x 破坏性变更 + MSRV 待核 | [EXT-4][EXT-6] | 1.1 钉版本+核 toolchain+读迁移指南 |
| **P1** | AC1027 转换损失无判据（>10000 只证明没崩） | [EXT-7][EXT-9] | 0.1/1.2 缺失率+代理实体计数 |
| **P2** | 非 Claude provider（GLM）MCP 工具选择效果难保证 | [KB-E 二手] | 实测工具调用准确率，必要时调描述 |
| **P2** | stdio 二进制缺失/路径→connection closed | [KB-D] | 3.3 存在性+完整错误诊断 |
| **P2** | 许可混合（rmcp MIT/Apache）+ GPL LibreDWG | [EXT-5][EXT-9] | 3.4 许可齐备；acadrust 原生 DWG 可去 GPL（待实测） |
| **P2** | 遥测/凭据隐私（业主图纸敏感） | [EXT-3][EXT-1] | 3.2 禁遥测+keyring |
| **P2** | AutoCAD 2014 损坏阻塞 COM 线 | spec:37/108 | DXF 线先行，COM 待正版 |
| **P2** | 仓库根 G: vs D: 环境不一致 | plan:15 | 0.4 确认盘符 |

**通用注意**：
- MCP 已无状态化——任何「靠 stdio 进程活着记状态」的写法都违规 [EXT-1]；跨调用状态一律显式 handle。
- 业务错误（缺标准、解析失败）必须工具级可见，协议级错误用户看不到 [EXT-4]——这直接决定 spec:20「报告缺哪本」能否兑现。
- 工具描述/schemars 字段文档是模型选工具的唯一依据 [EXT-4][KB-E]，中文描述要精确。
- 所有「最新」断言以核验日期 2026-11-17 为准；rmcp/goose/MCP 规范迭代快，开工前复核版本。

---

## 五、决策日志（2026-11-17 业主反馈后更新）

> 业主反馈：第 4 题选 D:；第 1/2/3 题「能力储备不够，回答不上来」。处置原则：不要求业主盲选工程题——#1 改为实验裁定，#2/#3 给证据化 provisional 默认并标 pending，自主推进可逆内部工作。

1. **DXF 引擎** → **spike 已出数据，provisional 推荐 acadrust**（详见 `spike-dxf-engine/SPIKE_REPORT.md`），不盲选。
   - 环境已核：Rust 1.99.1 / cargo 1.99.1 在位；D 盘可写（剩 ~83GB）。
   - **电气基准样本已到位（业主 2026-11-17 提供）**：`D:\微信文件\xwechat_files\wxid_yzv5spffpwta22_f83f\msg\file\2026-03\雁能1#-电气施工图 - 无人防版.dwg`，实测 **AC1018（2004）、6.98MB**；plan 原路径（「2026.3.27电气修改」后缀、AC1027/约 10MB）本机不存在，plan/spec 已同步纠正。
   - 替代样本：`D:\AI\cad\workbuddy` 衡阳三栋暖通施工图（A/C/D 栋），**版本头实测 AC1032（2018+）**，带天正代理实体/中文图层；KB-B 有 observed/verified:true 的 ezdxf 清点基线（9 VRV 外机/197 室内机）作损失率 ground-truth。
   - spike crate：`D:\AgentRust\spike-dxf-engine`（acadrust 0.6.8 failsafe 探针）。**注意：替代暖通样本是 AC1032；雁能电气基准实测 AC1018（非 AC1027），AC1027 本机从未出现——AC1032/AC1018 判据均已实测通过，AC1027 若日后出现按同判据补测。**
   - **实测裁定（2026-11-17）**：acadrust 原生 DWG AC1032 读取成功（<750ms/2MB，无需 LibreDWG）；**P1-3 代理实体 PASS**——EXTENDED_ENTITY 计数与 ezdxf ACAD_PROXY_ENTITY 基线逐栋精确吻合（A11/C291/D12）；**P2 中文 PASS（法证确认）**——图层数精确吻合 310/233/181，乱码经 GBK→UTF-8 逆变换还原为「说明文字/多联机标注/送风法兰/暖通_送风_风口」等正确暖通层名，证引擎输出 UTF-8 无误、乱码系 PowerShell cp936 捕获层；failsafe 每份仅 4 条 type_code=38 跳过（~0.01%）。
   - **provisional 推荐 acadrust 替代 dxf+LibreDWG**（去 GPL、去转换步骤、代理实体保留、failsafe 损失证据）；保留 dxf+LibreDWG 为 fallback 直到 AC1027 验证。代价：0.x 成熟度、全文档实体口径（M1 须过滤模型空间+按 INSERT 计数）。
   - **安全拦截与升级**：v2 探针运行触发宿主沙箱拦截，未静默重试；经业主授权走 dangerouslyDisableSandbox 升级通道完成电气基准补测。
   - **AC1018 电气基准补测（业主提供样本）**：实际版本头 **AC1018（2004）非 AC1027**、6.98MB（plan/spec 的「AC1027/10MB」与路径均写错，已纠正）；路径 `D:\微信文件\xwechat_files\...\雁能1#-电气施工图 - 无人防版.dwg`。实测：原生读取 **0 跳过/0 恢复错误**、2.15s/141845 记录；**P2 代码页 PASS（清洁确认）**——pre-2007 GB2312 中文层名完整正确（EQUIP-箱柜/插座/照明/消防/安防/通讯/动力、WIRE-母线），UTF-8 清洁 dump 落盘；设备清点可行（INSERT 12059 + `$equip$` 天正电气块 569）。
   - **电气 AC1018 UNKNOWN 已结案**：4205 个 UNKNOWN 经 M1 测试 dump 证实为**天正建筑 TCH_\* 对象**（墙1963/门窗洞1072/柱523/楼梯/轴号等，合计4205），**ACAD_PROXY_ENTITY=0**；属建筑底图非设备，dxf_name 无损保留身份。电气设备在 `$equip$`/`$Equip$`/`$equip_U$`/`$TelecSys$` 块 INSERT（Top $TelecSys$=792/$equip_U$=716/$equip$=546）。详见 SPIKE_REPORT §七。
   - **M1 Task1/2 已落地并测试绿**：`D:\AgentRust\cad-electrical-agent`（bin acad-mcp/lib acad_mcp），drawing.rs 用 acadrust（Unknown 按 dxf_name 计入），clap CLI，asset/server 占位；`cargo test` 2 passed/0 failed（匿名块排除 + 雁能 AC1018 集成断言 skipped=0）。Task4 指导：排除 TCH_* 建筑对象、按 $equip$/$TelecSys$ 块名+图层交叉归类设备。**Task4 已完成并验证**：asset.rs+config/asset_rules.json（10 类别+17 排除+属性A取名），cargo test 6 passed/0 failed（真实样图测试经 CAD_FIXTURE_DWG 门控：未设硬失败、设后全过；drawing.rs 已改 `model_space_entities()` 模型空间口径并加直方图/全库自洽断言），calamine 0.38 读回断言，CLI 冒烟 exit 0，交付件 `D:\AgentRust\雁能1#_资产登记表.xlsx`（**203 行/设备 6460，模型空间口径**，SHA256 2BA2F01AA9DBF3A155B5EDFB57FE2EE0AC33D4A925E5ECF1B565B33AC90A5D0D；修正前全文档口径 252 行/设备 6841，差额 381 台来自块定义/布局上下文；旧表备份 `D:\AgentRust\.tmp\evidence-crosscheck\资产登记表.prefix-model-space.bak.xlsx`）。规则 provisional 待业主确认（spec:119）。下一步 Task5 rmcp stdio server（先核 API/钉版本）。
2. **双 COM 通道主从** → **provisional，下游依赖 #1，M4 前冻结**。倾向窄口 COM 归一方、另一方瘦身（消除 spec:52/64 重叠）；若 acadrust 文件级达标，自研 Rust 桥统一 COM+GB2312 编码（spec:38）可省 Python 打包风险。M4 阻塞于正版 AutoCAD，不卡 M1，暂不定死。
3. **goose provider** → **provisional GLM（智谱，OpenAI 兼容）**，[pending] key（业主安装时填）+ 工具调用准确率实测（KB-E 非 Claude 效果风险）。M1 测试无 LLM（plan:7），不卡。
4. **仓库根盘符** → **锁定 D:**（业主 2026-11-17 决定）。项目根 `D:\AgentRust`；正式 crate 后续 `D:\AgentRust\cad-electrical-agent`；spike 临时 crate `D:\AgentRust\spike-dxf-engine`。

---

## 六、未决 / pending 项（故障≠不存在，未核≠不成立）

- [pending] acadrust / dxf-tools-rs 真实样图实测（Phase 0.1）
- [pending] goose 客户端 2026-07-28 MRTR/elicitation 兼容性（Phase 2.3）
- [pending] rmcp 确切钉死版本 + MSRV（crates.io / Cargo.toml 核）
- [pending] GB/T 16895.15-2023 正式文本与条款号级内容
- [pending] Windows goose 配置路径官方确认（现仅二手 blog `%APPDATA%\Block\goose`）
- [pending] 双 COM 通道裁决（业主）
- 说明：scrapling 本机未安装，已知官方 URL 改用宿主 WebFetch 定向抓取（已记录，非「不存在」）；KB 取证 80% 水位收口，MCP Server 开发实践附录正文未取全（S23 为 part3 摘要，二手）。

---

## 七、来源索引（速查）

- 原库切片文件：`graphify-out/kb-standards.json`、`kb-dxf.json`、`kb-bestpractice.json`、`kb-mcp-dev.json`（session arch-audit-20261117）
- 上午审查：`D:\AgentRust\ARCHITECTURE_REVIEW.md`（P0/P1 行号级证据 + 官方 URL）
- 知识图谱：`graphify-out/graph.html`（34 节点/45 边/7 社区）
- 外部官方 URL 见 §二 EXT-1~EXT-9，核验日期 2026-11-17
