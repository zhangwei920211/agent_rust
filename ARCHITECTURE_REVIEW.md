# 架构审查报告：CAD 电气深化设计 Agent（superpowers 下发文档）

- 审查对象：`D:\AgentRust\superpowers\specs\2026-11-16-cad-electrical-agent-design.md`（125 行）、`D:\AgentRust\superpowers\plans\2026-11-16-m1-dxf-asset-pipeline.md`（301 行）
- 审查日期：2026-11-17
- 知识图谱：`D:\workbuddy\2026-11-17-10-05-59\graphify-out\graph.html`（交互图）、`graph.json`（原始数据）、`GRAPH_REPORT.md`（图谱报告）——34 节点 / 45 边 / 7 社区

## 方法与工具（可复核）

| 环节 | 工具 | 产物 |
|---|---|---|
| 原文核对 | Read 分段读完两份文档全文（含被截断的后续段） | 本报告行号引用 |
| 库内取证 | `obsidian-cli/slice_retrieve.py --vault D:\pi --scope projects/hvac-cad-electrical`（session `arch-audit-20261117`，2 步 / 6 步预算） | `graphify-out/kb-standards.json`、`kb-dxf.json` |
| 库内初筛 | 宿主 Explore 子代理（只读，未联网） | 原库路径与行号 |
| 外部核验 | 宿主 general-purpose 子代理，仅官方一手页面 | 见 §3 官方 URL |
| 图谱构建 | graphify 全量流水线（语义抽取子代理 + 确定性建图） | `graphify-out/` 三件产物 |

**证据分级声明**：§1 结论全部锚定本次实际读到的原文行号；§2 中标注「已直接核验」的为本次亲自读到切片，标注「子代理取证」的需抽查复读；§3 全部来自官方公开页面，URL 已在表中列出。

---

## 1. 设计范式核对（文档声明 vs 落地证据）

| 范式声明 | 落地证据 | 判定 |
|---|---|---|
| agent 零 fork，价值全在自研 MCP server 与 skills（spec:49 选 A） | plan 单 crate `acad-mcp` lib+bin，rmcp stdio 暴露 5 工具（plan:5,7,216-227） | **一致** |
| 反目标：不在原图上动手（spec:30） | plan:20 全部只读 DXF，唯一写动作是生成 xlsx 到用户指定路径 | **一致**，且是全文最强的一条约束落地 |
| 全部自研部分为 Rust（spec:21） | plan 技术栈纯 Rust（plan:9）；第三方 Python autocad-mcp-pro 以 standalone 内嵌（spec:52,69） | **措辞自洽**（"自研部分"）但打包复杂度集中在 Python 侧 |
| TDD：先写失败测试（superpowers 强制） | 每个 Task 均「Step 1 写失败测试 → Step 2 看 FAIL → 实现 → PASS」（plan:124-138、191-201、232-…） | **一致**，且 fixture 给了精确断言值 |
| 规范网关先行，查不到即拒绝执行（spec:20） | 门禁范围切割清楚：只门禁引用规范条文/数值的动作，S4/S5 不受门禁（spec:102） | **一致**，切割理由成立（否则流水线被堵死） |
| GPL 不传染 | LibreDWG 仅外部命令行调用，不链接进二进制（plan:7,17）；业主自用无分发义务（spec:120） | **一致**（官方许可证确认为 GPLv3+，见 §3） |
| 匿名/布局块不进设备清单 | 判定规则明确：块名以 `*` 或 `A$` 开头即排除（plan:122），并有断言（plan:27,130） | **一致** |
| 区分块定义数与 INSERT 引用数 | `blocks` 与 `insert_counts` 分列（plan:114-116），断言 `insert_counts["风机-1"]==3`（plan:131） | **一致**，避免了最常见的清点错误 |
| 任意 Windows 路径可用 | 专设 Task 3 Step 3：复制到含空格+中文+`#` 的临时路径再跑（plan:29,165） | **一致** |

**范式总评**：这份文档的纪律性明显高于平均水准——反目标可执行、验收可判定、TDD 编译进了任务粒度。下面 §2/§3 是它与现实对不上的地方。

---

## 2. 规范符合性缺陷（按优先级）

### P0-1　GB/T 16895.15-2023 的「现行版」断言被反证

- 文档断言：安装包预置基线缓存含 **GB/T 16895.15-2023（2026-11-16 核实为现行版，替代 2002 版）**（spec:103）
- 库内直接核验（`kb-standards.json` S1，`projects/hvac-cad-electrical/gbt-16895-15-2023-low-voltage-wiring-systems.md#L1-69`）：该笔记 `status: draft`、`verified: false`、`provenance: derived-not-observed`；正文明确 **2023 新版在 openstd 检索未命中，全文逐条核对标 pending**；现行采标版为 **GB/T 16895.6-2014**；2002 版已废止
- 外部官方核验：国家平台精确检索无 2023 版记录，同号旧版为 2002 版且已废止 → **反驳**
- 影响：预置缓存一旦按 2023 版建档，首装即携带**未核实版本戳**，而网关的阻断逻辑恰恰依赖版本戳
- 处置：M2 建缓存前必须闭合该版本；未闭合则降级为「按 GB/T 16895.6-2014 + 待核 2023 双记录」，并把 2023 标 pending

### P0-2　openstd 单源核验会误杀 S6/S7 全线的规范

- 文档口径：联网侧用 openstd 核验现行性（spec:103,115）
- 库内证据（子代理取证，待抽查）：GB 50168-2018、GB 51348-2019 为**工程建设国家标准**，国家标准全文公开系统明确不纳入；JGJ 242-2011 为**行业标准**，该系统不收录 → 三者检索均为 0
- 与硬性要求的冲突：spec:20 规定「**查不到即拒绝执行**」。若核验源只有 openstd，则 S6（引 GB 50054/GB/T 16895.15）、S7（引 GB 50168-2018/GB 51348-2019）在首装离线场景下**全部被阻断**
- 处置（建议写入 M2 契约）：核验源按标准归口分派——工程建设国标走住建部发布渠道，行业标准走对应行业主管渠道，国际对标（IEC 转化）单独标注；「未命中」与「不现行」必须区分，禁止单源零命中直接判阻断

### P1-1　双 COM 通道职责重叠，没有裁决规则（本设计的**主要矛盾**）

| 通道 | 声明位置 | 能力 |
|---|---|---|
| Python `autocad-mcp-pro`（MIT，247 工具，COM+ezdxf） | spec:52「**CAD 工具主力**」 | 开图、画线、插符号、COM 全量 |
| 自研 Rust `acad-mcp.exe` 内 COM 桥 | spec:64「开图/存副本/画线画块/插符号/查实体/改图层/跑 LISP/出图」 | 与上面**逐项重叠** |

同时 plan:7 声明「COM 线与规范网关属 M2/M4」，与 spec:63-68 把 COM 桥画进系统结构图并存。两份文档对「谁负责在原图画图」没有主从定义、没有冲突裁决、没有能力边界。这是架构层未定案项，不是细节。**必须在 M2 开工前拍板：要么瘦身 autocad-mcp 的 COM 桥（只留 DXF 与规范网关），要么裁剪 `TOOL_PACKS` 后把窄口 COM 归 autocad-mcp-pro。**

### P1-2　AC1027 转链可行性尚未闭合，但已被当成基准假设

- spec:56 自认「样图 AC1027 经 LibreDWG 转 DXF 的可行性**在实施期验证**」，而 plan:21 已把该样图定为**基准样图**，plan:159 断言实体数 `> 10000`
- 外部核验：LibreDWG `dwg2dxf` 有 R2013/AC1027 版本证据，但官方自述约 90% 覆盖率、部分 R2010+ 高级对象跳过（**限定支持**）；Rust `dxf` crate 无完整 AC1027 兼容承诺（**未证实**）
- 缺口：`> 10000` 只能证明「没崩」，**不能证明转换无损**。当前没有任何断言能区分「图里本来就没那么多实体」和「转换丢了实体」
- 处置：Task 3 增加损失判定基线——与 AutoCAD 侧实体总数（或图纸台账）比对，缺失率超阈值即判失败并落 README；同时把「代理实体/无法转换对象」单独计数

### P1-3　天正代理实体的降级路径缺失

- spec:116 已定案：业主装天正电气 → **执行代理实体降级读取**
- plan Task 2 只处理 INSERT/块定义/匿名块（plan:112-122），**未定义 `ACAD_PROXY_ENTITY`（天正自定义对象）的降级路径**
- 而 plan:21 的基准样图正是天正产出的电气施工图——降级路径不是可选增强，是 M1 的必经路径；缺它则 Task 3 的样图断言可能直接不成立

### P1-4　goose 配置 schema 未冻结，但 M1 要交付配置示例

- spec:96 明确「goose 配置文件的确切 schema 按其当前版本文档落实」（= 未冻结）
- plan:279 要求 Task 6 交付 `config/goose-config-example.yaml`
- 处置：Task 6 该步标注「按核实时的 goose 版本填充，与官方文档链接一并记录」，否则交付物一出即过期

### P2　次级缺口

| 项 | 证据 | 处置 |
|---|---|---|
| 规范网关口径双版本并存 | spec:20「按标准号联网核实现行版本」 vs spec:115「业主 KB + openstd 仅核现行性」 | 把 §2 表述回填为 §8.1 定案版，否则施工者会建两套 |
| DXF 编码未处理 | spec:38 规定 GB2312↔UTF-8 在桥接层处理；M1 无 DXF code page 断言（fixture 为 UTF-8，plan:126-133） | 增加一条 GB2312 编码 DXF 的 fixture |
| 仓库根与环境不一致 | plan:15 仓库根 `G:\cad agent`；本次落地 `D:\AgentRust` | 实施前确认 G 盘可用，或统一到 D 盘 |
| 人工验证无自动断言 | plan:166,202（Top-10 块名人工抽查） | 可接受，但结论必须写入 `tests/fixtures/README.md` 才算交付 |

---

## 3. 外部官方规范核验（一手页面）

| 断言 | 结论 | 官方依据（URL / 页面） |
|---|---|---|
| MCP host 建 client、server 暴露 tools；`tools/list` 发现、`tools/call` 调用；应保留人工否决 | **支持** | modelcontextprotocol.io — Architecture overview；Specification `latest`（读取时为 2026-07-28 版）— Tools |
| stdio 由 client 启动 server 子进程，stdout 不得混入非 MCP 内容 | **支持** | modelcontextprotocol.io — Specification / Transports / stdio |
| goose 支持 MCP servers / Skills / Recipes / Custom Distributions | **支持** | block.github.io/goose 官方文档（MCP、Skills、Recipes、Custom Distributions 页） |
| 「goose 生态直接形成 Windows 单安装包」 | **未证实** | Custom Distributions 文档未承诺打包形态 |
| rmcp（官方 Rust SDK）支持服务端 stdio 与 tool router | **支持** | docs.rs/rmcp — `transport-io`、`#[tool_router]` |
| LibreDWG 许可证 | **GPLv3 or later**（注意不是 "GPLv3 only"） | gnu.org LibreDWG 页 |
| `dwg2dxf` 支持 AC1027 | **限定**：有 R2013/AC1027 证据，官方自述约 90% 覆盖、R2010+ 高级对象跳过 | LibreDWG 官方仓库/手册 |
| Rust `dxf` crate 完整支持 AC1027 | **未证实** | crates.io / docs.rs `dxf`（版本 0.6.x），无兼容矩阵 |
| GB 50168-2018 | **限定**：未发现整部废止 | 住建部发布渠道 |
| GB 51348-2019 | **限定**：原列强制性条文已被 GB 55024-2022 公告替代，非整部废止 | 住建部公告 |
| JGJ 242-2011 | **限定**：原四个强制性条文已被替代，非整部废止 | 住建部公告 |
| GB/T 16895.15-2023 为现行版 | **反驳**：官方精确检索无记录，同号旧版 2002 已废止 | 国家标准全文公开系统 / 全国标准信息公共服务平台 |

**许可侧结论**：plan:7,17 的 GPL 隔离路线在许可证层面成立（外部进程调用 ≠ 链接）；配合 spec:120「仅业主自用、无分发义务」，本项目**当前不承担 LibreDWG 的分发义务**。但一旦将来对外分发安装包，GPLv3+ 的传染边界需重新评估——建议现在就把「外部工具调用」这一边界写进架构约束，而不是留在口头共识里。

---

## 4. 知识图谱结果

- 规模：34 节点 / 45 边 / 7 社区；边 100% EXTRACTED、0% INFERRED（**含义：图谱只反映"文档这么写"，不代表"事实成立"**，与 §2/§3 的核对结论不冲突）
- 覆盖率：两份文档分别读到 301/301 与 125/125 行，无部分读取
- 健康检查：无悬挂端点、无缺失端点、无自环、无同端点折叠边

**枢纽节点（按连接度）**：`规范网关`(9) > `acad-mcp`(6) = `S1-S7 七步工作流`(6) > `M1 DXF 读图与资产登记管线`(5) = `S6 配电规划`(5) = `S7 桥架走向`(5) = `goose 原装发行版`(5)

**社区划分**：规范网关与强制标准 / 七步工作流与图纸操作约束 / goose 发行版与 MCP 工具链 / M1 读图管线接口 / M1 运行时集成 / 资产登记与 xlsx 交付 / DXF 解析与统计

**图谱暴露的结构事实**：`规范网关`连接度最高（9 边），但 M1 计划明确不含网关（plan:7）——**连接度最高的节点恰好是当前实施里程碑的边界外**。这与 §2 的 P0 结论互相印证：门禁是全局枢纽，而它的核验源假设目前站不住。

**注意**：本次未捕获子代理 token 用量（图谱报告计为 0），该数字仅表示"未计量"，非"零消耗"。

---

## 5. 建议修正清单（按开工顺序）

1. **M1 开工前**：确认仓库根盘符（G: vs D:）；把 AC1027 转链验证提前为 Task 3 的**阻塞前置**，并定义转换损失判据
2. **M1 实施中**：Task 2 补代理实体降级路径；Task 3 补 GB2312 编码 fixture；Task 6 标注 goose schema 待核实
3. **M2 开工前**：拍板双 COM 通道主从与边界（P1-1）；按标准归口定义多源核验契约，"未命中≠不现行"
4. **M2 建缓存前**：闭合 GB/T 16895.15-2023 版本（P0-1），未闭合则双记录 + pending
5. **文档维护**：spec §2/§6 的网关表述回填为 §8.1 定案口径，消除双版本并存

## 6. 未决与不确定项

- 待抽查：GB 50168-2018 / GB 51348-2019 / JGJ 242-2011 的库内原片为子代理取证，本轮未逐条复读（P0-2 结论方向已由外部官方核验独立支持）
- 未能核验：GB/T 16895.15-2023 的 2023 版全文与条款号级内容（需正式文本）
- 未涉及：M2-M5 里程碑文档不在本次两文件范围内，图谱未覆盖

## 7. 证据索引

| 编号 | 来源 |
|---|---|
| spec:20/21/30/38/49/52/56/63-68/96/102/103/115/116/118/120 | 设计文档原文行 |
| plan:7/9/15/17/20/21/25-29/112-122/124-138/159-160/165/191-201/216-227/279/297-300 | M1 计划原文行 |
| KB-1（已直接核验） | `projects/hvac-cad-electrical/gbt-16895-15-2023-low-voltage-wiring-systems.md#L1-69`（`kb-standards.json` slice 1） |
| KB-2（子代理取证） | `gb-50168-2018-cable-line-construction-acceptance.md`、`gb-51348-2019-civil-building-electrical-standards.md`、`jgj-242-2011-residential-architectural-electrical-standards.md` |
| EXT | 见 §3 表中官方 URL，核验日期 2026-11-17 |
