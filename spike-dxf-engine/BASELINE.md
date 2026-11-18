# Spike Ground-Truth 基线（ezdxf observed，AC1032）

来源：`D:/AI/cad/workbuddy/_probe/*.inventory.json`（KB-B 衡阳三栋，provenance=observed/verified:true，
accoreconsole/ODA → DXF → ezdxf 1.4.4 清点）。样本版本头实测 AC1032（AutoCAD 2018+）。

| 栋 | 文件 | entities | ACAD_PROXY_ENTITY | layers | blocks_defined | layouts | Top 类型 |
|---|---|---|---|---|---|---|---|
| A | A栋2~4F暖通施工图.dwg | 6436 | 11 | 310 | 639 | 2 | LINE 2920 / TEXT 1318 / INSERT 1305 / LWPOLYLINE 442 |
| C | C栋1~2F国检集团实验室暖通施工图.dwg | 1482 | 291 | 233 | 344 | 2 | LINE 661 / TEXT 370 / PROXY 291 / INSERT 116 |
| D | D栋1F德山表业暖通施工图.dwg | 479 | 12 | 181 | 304 | 2 | LINE 157 / TEXT 147 / DIMENSION 71 / INSERT 43 |

设备基线（KB-B）：VRV 外机 A6/C2/D1=9；室内机 A151(109天正+42美的)/C31/D15=197。
A 栋底图灯具 C-light 369、JD-强电插座图例 121。

## 判据
- **损失率**：acadrust `output_entities` 与上表 entities 同量级；显著偏低（如 C 栋 <1000）即判转换/读取损失。
- **代理实体（P1-3 关键）**：C 栋应识别 ~291 个代理对象（ProxyGraphics/ProxyGraphicRecord）。归零或报错＝红旗。
- **代码页/中文（P2）**：图层/块名含 CJK 且不乱码。
- **注意**：inventory 的 entities 为 DXF 模型空间口径；acadrust output_entities 可能含块内/布局实体，口径差异需说明，不直接等同。
- **样本局限**：AC1032 非基准 AC1027；结论对 AC1032 有效，AC1027 待雁能电气样本复核 [pending]。
