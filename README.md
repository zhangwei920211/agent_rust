# agent_rust

CAD 电气深化设计 Agent：自研 Rust MCP server（`acad-mcp`），当前处于 M1「DXF/DWG 读图与资产登记管线」。

## 目录结构

| 路径 | 说明 |
|---|---|
| `cad-electrical-agent/` | M1 正式 crate：lib `acad_mcp`、bin `acad-mcp`，基于 acadrust 读图并生成资产登记表 |
| `spike-dxf-engine/` | DXF/DWG 引擎选型 spike 与实测报告 |
| `superpowers/plans/` | M1 实施计划 |
| `superpowers/specs/` | 设计文档 |
| `BEST_PRACTICE_PLAN.md` | 带来源标注的最佳实践计划 |
| `ARCHITECTURE_REVIEW.md` | 架构审查报告 |

## 构建与测试

```powershell
cd cad-electrical-agent
cargo build
cargo test
```

真实样本集成测试由环境变量门控：

```powershell
$env:CAD_FIXTURE_DWG = "<样图.dwg 路径>"
cargo test -- --nocapture
```

## 使用

```powershell
# 统计图纸
cargo run -- cli summary <样图.dwg>

# 生成资产登记表
cargo run -- cli register <样图.dwg> out.xlsx
```

## 说明

- 仅 Windows；只读源图，唯一写动作是生成 xlsx。
- 依赖版本见 `cad-electrical-agent/Cargo.lock`。
