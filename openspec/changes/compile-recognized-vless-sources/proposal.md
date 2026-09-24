# Proposal: 让已识别 VLESS 来源进入严格编译

状态：`accepted`

## 问题

当前来源检查能够识别 URI 列表和 Base64 URI 列表并统计 VLESS 节点，但严格 Node IR 只读取 `proxies:` YAML。前端仅依据“节点数大于零”显示“可进入编译”，导致用户点击生成后才收到格式拒绝；错误位于预览滚动区中，视觉上近似无响应。

这是“已识别”“格式可编译”“严格隐私校验通过”三层状态被错误合并造成的主流程阻塞，不应留到导出或 sidecar 阶段。

## 目标

1. URI 列表与 Base64 URI 列表中的 VLESS 分享链接直接转换为 Mihomo Node IR；
2. 保留现有 IPv4 literal、域名 bootstrap、IPv6 和旁路字段的 fail-closed 隐私边界；
3. 来源摘要显式报告格式是否已具备当前编译器支持，草案只有在全部来源格式就绪时才显示“可进入编译”；
4. 点击生成后持续显示进行中、成功或带下一步的失败状态，失败区域固定在主操作附近并可被辅助技术播报；
5. 分享链接的 UUID、Reality key、路径、host 和名称仍是秘密，不进入日志、错误或无秘密报告。

## 当前范围

- 接通标准 `vless://` URL 以及其 Base64 订阅包装；
- 支持 Mihomo 当前 VLESS 字段及常用 `tcp`、`ws`、`grpc`、`http`/`h2`、`xhttp` 传输映射；
- 未映射协议、重复参数或未映射选项稳定失败，不静默丢弃；
- VMess、Trojan、SS 和 provider resolver 继续显示为未接通，不再误报可编译。

## 验收场景

### VLESS URI 来源

- **GIVEN** 来源为 VLESS URI 或 Base64 包装的 VLESS URI 列表
- **AND** 节点 server 为 IPv4 literal
- **WHEN** 用户点击生成
- **THEN** 系统把分享参数转换为 Mihomo Node IR
- **AND** 继续执行跨来源去重、严格隐私校验和确定性 YAML serializer。

### 未支持协议

- **GIVEN** 来源已识别出节点但包含当前未映射协议
- **WHEN** 草案更新
- **THEN** 状态不显示“可进入编译”
- **AND** 来源卡与主操作附近显示稳定、无秘密的阻塞原因。

### 域名 bootstrap

- **GIVEN** VLESS 分享链接的 server 是域名
- **WHEN** 进入严格编译
- **THEN** 继续按严格隐私策略拒绝
- **AND** 不回显 server、UUID 或完整分享链接。

## 回滚

修改前副本：`/tmp/mihomo-studio-before-uri-compiler-20260815T0035`。当前目录无 Git 元数据；不初始化仓库，不提交或推送。
