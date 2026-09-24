import type { CompileProfileResult, ProfileDraft } from "../../../shared/contracts";

interface PreviewPanelProps {
  draft: ProfileDraft;
  selectedSourceCount: number;
  desktopRuntime: boolean;
  isCompiling: boolean;
  compiledProfile: CompileProfileResult | null;
  compileError: string;
  onCompile: () => void;
}

export function PreviewPanel({
  draft,
  selectedSourceCount,
  desktopRuntime,
  isCompiling,
  compiledProfile,
  compileError,
  onCompile,
}: PreviewPanelProps) {
  const healthLabel = isCompiling
    ? "正在编译"
    : compiledProfile
      ? "YAML 已生成"
      : compileError
        ? "生成未完成"
        : draft.readyForCompilation
          ? "可进入编译"
          : "等待来源";

  return (
    <section id="preview" className="preview-panel" aria-busy={isCompiling}>
      <div className="preview-panel__header">
        <div><span className="step-tag">步骤 3</span><h2>生成与结果</h2></div>
        <span className={draft.readyForCompilation ? "health health--ready" : "health"}>
          {healthLabel}
        </span>
      </div>

      <div className="preview-panel__footer">
        <div className="generation-copy">
          <strong>{compiledProfile ? "配置已生成" : "准备生成 Mihomo YAML"}</strong>
          <span>
            {compiledProfile
              ? "静态回读通过；YAML 含节点凭据，请按敏感文件处理。"
              : "转换节点、编排规则并执行严格隐私静态检查。"}
          </span>
        </div>
        <button type="button" className="compile-button" onClick={onCompile} disabled={isCompiling}>
          {isCompiling
            ? "正在严格编译…"
            : desktopRuntime
              ? "生成严格隐私 YAML"
              : "生成方案预览"}<span>→</span>
        </button>
      </div>

      {isCompiling && (
        <div className="compile-state compile-state--running" role="status" aria-live="polite">
          <strong>正在生成</strong>
          <span>转换节点并执行严格隐私静态检查…</span>
        </div>
      )}
      {compileError && !isCompiling && (
        <div className="compile-state compile-state--error" role="alert" aria-live="assertive">
          <strong>生成未完成</strong>
          <span>{compileError}</span>
        </div>
      )}

      {compiledProfile && (
        <div className="compiled-profile">
          <div className="preview-title"><span>生成结果</span><small>静态校验通过</small></div>
          <div className="compile-report-grid">
            <div><strong>{compiledProfile.report.nodeCount}</strong><span>节点</span></div>
            <div><strong>{compiledProfile.report.ruleProviderCount}</strong><span>Provider</span></div>
            <div><strong>{compiledProfile.report.ruleCount}</strong><span>规则</span></div>
          </div>
          <code className="content-hash">SHA-256 {compiledProfile.report.contentSha256}</code>
          <details className="yaml-disclosure">
            <summary>查看敏感 YAML</summary>
            <textarea
              aria-label="生成的 Mihomo YAML"
              className="yaml-preview"
              readOnly
              spellCheck={false}
              value={compiledProfile.yaml}
            />
            <button
              className="copy-button"
              onClick={() => navigator.clipboard.writeText(compiledProfile.yaml)}
            >
              复制 YAML
            </button>
          </details>
          <div className="warning-box">
            {compiledProfile.report.warnings.map((warning) => <p key={warning}>{warning}</p>)}
          </div>
        </div>
      )}

      <details className="draft-disclosure">
        <summary>
          <span>查看编译计划</span>
          <small>{draft.resolvedNodeCount} 节点 · {selectedSourceCount} 规则仓库 · {draft.groups.length} 策略组</small>
        </summary>
        <div className="draft-disclosure__content">
          <div className="summary-grid summary-grid--four">
            <div><strong>{draft.inputSourceCount}</strong><span>节点来源</span></div>
            <div><strong>{draft.resolvedNodeCount}</strong><span>识别节点</span></div>
            <div><strong>{selectedSourceCount}</strong><span>规则仓库</span></div>
            <div><strong>{draft.groups.length}</strong><span>策略组</span></div>
          </div>

          <div className="draft-plan-grid">
            <div className="preview-section">
              <div className="preview-title"><span>策略组</span><small>自动编排</small></div>
              <div className="group-list">
                {draft.groups.map((group, index) => (
                  <div className="group-row" key={group}>
                    <span className="group-index">{String(index + 1).padStart(2, "0")}</span>
                    <strong>{group}</strong>
                    <span className="group-arrow">→</span>
                  </div>
                ))}
              </div>
            </div>

            <div className="preview-section">
              <div className="preview-title"><span>规则顺序</span><small>从上到下</small></div>
              <ol className="order-list">
                {draft.ruleOrder.map((rule) => <li key={rule}>{rule}</li>)}
              </ol>
              <div className="final-rule"><span>LAST</span><code>{draft.finalRule}</code></div>
            </div>

            <div className="preview-section">
              <div className="preview-title"><span>严格隐私</span><small>编译硬约束</small></div>
              <ol className="order-list">
                <li>DNS：{draft.privacy.dnsMode} · 仅代理出口</li>
                <li>DNS 劫持：必须 · TUN strict route：必须</li>
                <li>IPv6：关闭 · DIRECT：禁止</li>
                <li>节点域名 bootstrap：必须受保护</li>
                <li>Provider 下载与刷新：仅代理出口</li>
              </ol>
            </div>
          </div>

          {draft.warnings.length > 0 && (
            <div className="warning-box">
              {draft.warnings.map((warning) => <p key={warning}>{warning}</p>)}
            </div>
          )}
        </div>
      </details>

      <p className="preview-footnote">
        {compiledProfile
          ? "尚未证明 Clash Verge 已加载、DNS 路径或最终公网出口。"
          : "严格隐私编译会拒绝无映射域名节点、DIRECT 和未代理的 Provider 更新。"}
      </p>
    </section>
  );
}
