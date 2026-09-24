# Capability: 验证与导出

| 字段 | 值 |
|---|---|
| ID | `validation-export` |
| 状态 | `partial` |
| 当前 IPC | `compile_strict_profile` |

## Purpose

把领域编译结果安全转换为确定性 YAML，通过绑定 Mihomo 内核验证，原子导出，并保持静态证据与外部运行态证据的严格分层。

## Current State

已实现：

- Node/Rule/Provider/Group IR 结构检查；
- 确定性 YAML serializer；
- YAML 安全回读与严格隐私字段检查；
- secret-bearing YAML 与无秘密生成报告分离；
- IPv4 Mihomo YAML、VLESS URI、域名/hosts bootstrap 与独立 WireGuard 节点四份脱敏 golden file；
- 使用官方 Mihomo v1.19.29 darwin arm64、固定压缩包 SHA-256 对四份 golden YAML 执行可复现的本地加载门禁。

尚未实现：应用内绑定 sidecar 生命周期、跨平台 sidecar bundle、原子导出、last-known-good、Clash Verge 集成或网络出口验证。官方内核的一次 fixture 验证不等于任意用户配置已由绑定 sidecar 验证。

独立节点转换可把单个 IP 节点放入内置单节点模版并安全回读，但该模版沿用用户样本的 DNS、规则和 TUN 设置，不属于严格隐私编译输出，也没有通过绑定 Mihomo sidecar 验证。应用目前只允许复制该结果供用户自行保存，不报告“已导出”。

## Requirements

### Requirement: 编译后结构校验

YAML 序列化前，系统 SHALL 验证：

- 节点、provider 和组名称唯一；
- 全部规则 target 和组引用存在；
- 策略组图无循环；
- 空组有显式回退；
- provider path 唯一且位于受管 HomeDir；
- 恰有一个最终 `MATCH`。

#### Scenario: 悬空策略引用

- **GIVEN** 规则 target 不存在
- **WHEN** 编译结果进入导出管线
- **THEN** 结构校验失败
- **AND** 不创建或覆盖目标文件

### Requirement: YAML 回读

生成的 YAML SHALL 使用安全 parser 回读，并验证关键语义与领域结果一致。远程 YAML 始终作为数据处理，不执行脚本、模板表达式或命令。

### Requirement: 严格隐私静态校验

严格隐私 YAML 在序列化前与回读后 SHALL 验证：

- `dns.enable=true`、`enhanced-mode=fake-ip`、`respect-rules=true`；
- 上游 nameserver 明确绑定代理策略，不存在系统 DNS、普通 53 或直连 DoH fallback；
- TUN 启用 DNS hijack、auto route 与 `strict-route`；
- IPv6 在未验证闭环时关闭；
- rules、provider 更新和辅助网络连接不存在未经批准的 `DIRECT` 路径；
- 节点域名不存在无保护 bootstrap。
- 每个域名节点恰有一个 IPv4 `hosts` 映射，且不存在悬空映射。

任一检查失败 SHALL 阻止导出和 last-known-good 覆盖。

### Requirement: 绑定 Mihomo 验证

系统 SHALL 使用版本和 SHA-256 均固定的 Mihomo sidecar 验证配置。sidecar SHALL 使用临时 HomeDir、loopback Controller、随机临时 secret、关闭健康检查，并在超时或完成后终止和清理。

#### Scenario: Mihomo 拒绝配置

- **WHEN** 绑定内核不能加载生成配置
- **THEN** 状态停留在“已生成但验证失败”
- **AND** 报告记录内核版本、错误类别和无秘密摘要
- **AND** 不覆盖 last-known-good

### Requirement: 原子导出

导出 SHALL 先写同文件系统临时文件、flush/fsync、设置私有权限，再原子替换目标。替换前 SHALL 保留 last-known-good；失败时原文件保持可用。

POSIX 文件目标权限为当前用户读写；Windows 使用当前用户 ACL。

### Requirement: 导出内容提示

生成报告不得包含 secret URL 或节点凭据；但导出的 YAML 可能包含凭据。UI SHALL 在保存前明确提示文件敏感性和保存位置。

### Requirement: 证据分层

系统 SHALL 分别记录：

| 层级 | 最低证据 |
|---|---|
| 已生成 | serializer 和结构校验成功 |
| Mihomo 已验证 | 固定版本内核实际加载成功 |
| 已导出 | 目标路径原子写入成功 |
| Clash Verge 已加载 | 外部应用当前配置状态证据 |
| DNS 路径 | Controller/连接或抓包证明 DNS 使用指定代理且无意外 53/DoH 直连 |
| 规则命中 | Controller rule/connection 与目标流量归因 |
| 最终出口 | 目标流量对应的 IPv4/IPv6 公网出口与应用旁路检查 |

较早层级不得推出较晚层级。

#### Scenario: 文件成功导出

- **WHEN** 原子替换完成
- **THEN** UI 可以显示“已导出”及安全路径摘要
- **AND** 不显示“Clash Verge 已加载”或“出口已验证”

### Requirement: Clash Verge 边界

首版只提供用户主动导入说明，不直接修改 Clash Verge 内部数据库或配置目录。未来受控同步必须单独设计备份、预览、回滚、版本探测和失败恢复。

## Implementation Map

| 路径 | 当前责任 |
|---|---|
| `src-tauri/src/domain/compile/model.rs` | 编译请求、结果、IR 与 typed error |
| `src-tauri/src/domain/compile/template.rs` | 内置模版组合与 YAML 回读，不替代严格隐私验证 |
| `src-tauri/src/domain/compile/templates/clash-single-node.yml` | 去敏的模版配置骨架 |
| `src-tauri/src/domain/compile/node.rs` | Node IR、跨来源精确去重和 bootstrap 检查 |
| `src-tauri/src/domain/compile/rule.rs` | Rule/Provider IR、strict target 与 provider transport |
| `src-tauri/src/domain/compile/serializer.rs` | 确定性 Mihomo YAML serializer |
| `src-tauri/src/domain/compile/validation.rs` | IR 图、引用、隐私字段和 YAML 回读检查 |
| `src-tauri/src/domain/compile/service.rs` | 纯编译管线、报告和 golden tests |
| `src-tauri/src/application/compile.rs` | 来源 material 与领域编译用例编排 |
| `src-tauri/src/interface/ipc/compile.rs` | Tauri 编译命令 |
| `src-tauri/sidecars/mihomo-v1.19.29.json` | 官方 fixture gate 的版本、下载地址与压缩包 SHA-256 pin；不代表已 bundle runtime |
| `scripts/validate-mihomo-fixture.sh` | darwin arm64 官方内核下载、hash 校验、临时 HomeDir 与 golden 加载门禁 |
| `src/features/profile-preview/ui/PreviewPanel.tsx` | 已生成状态、敏感 YAML 折叠预览和无秘密报告 |

未来 sidecar 与导出分别进入 infrastructure/application adapters；`src-tauri/src/lib.rs` 只注册命令和装配依赖。

## Required Verification

- 结构不变量单元测试；
- 脱敏 fixture 的 YAML golden tests；
- 绑定 Mihomo 版本/hash 校验；
- sidecar 超时、崩溃和清理测试；
- 原子写入和回滚集成测试；
- macOS、Windows、Linux 文件权限测试；
- Clash Verge 与实际流量验证作为独立手动/E2E 证据。
