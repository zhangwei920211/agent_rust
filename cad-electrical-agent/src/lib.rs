//! acad-mcp 库入口。M1：DXF/DWG 读图清点线。
//! 模块按 plan 划分：drawing（读图统计）、asset（规则分类→xlsx）、server（rmcp 工具面）。

pub mod asset;
pub mod drawing;
pub mod ops;
pub mod server;
