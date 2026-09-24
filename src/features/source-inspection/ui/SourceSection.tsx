import type { InputSource } from "../../../shared/contracts";
import { formatSourceLabel } from "../../../shared/lib/presentation";
import { countNonEmptyNodeLines } from "../model/directNodeInput";

interface SourceSectionProps {
  subscriptionUrl: string;
  directNodes: string;
  bootstrapMappings: string;
  inputSources: InputSource[];
  inspectingKind: "subscription" | "nodes" | null;
  sourceError: string;
  onSubscriptionChange: (value: string) => void;
  onDirectNodesChange: (value: string) => void;
  onBootstrapMappingsChange: (value: string) => void;
  onAddSubscription: () => void;
  onAddDirectNodes: () => void;
  onRemoveSource: (id: string) => void;
}

export function SourceSection({
  subscriptionUrl,
  directNodes,
  bootstrapMappings,
  inputSources,
  inspectingKind,
  sourceError,
  onSubscriptionChange,
  onDirectNodesChange,
  onBootstrapMappingsChange,
  onAddSubscription,
  onAddDirectNodes,
  onRemoveSource,
}: SourceSectionProps) {
  const directNodeLineCount = countNonEmptyNodeLines(directNodes);

  return (
    <section id="sources" className="section-block">
      <div className="section-heading">
        <div><span className="step-tag">步骤 1</span><h2>添加节点来源</h2></div>
        <span className="section-hint">秘密信息默认遮罩</span>
      </div>

      <div className="source-entry-grid">
        <article className="entry-card">
          <div className="entry-card__icon">↗</div>
          <div className="entry-card__copy">
            <h3>订阅链接</h3><p>支持按 User-Agent 探测 YAML、URI 和 Base64。</p>
          </div>
          <div className="input-row">
            <input
              aria-label="订阅链接"
              type="password"
              value={subscriptionUrl}
              onChange={(event) => onSubscriptionChange(event.target.value)}
              placeholder="https://example.com/sub?token=••••"
            />
            <button onClick={onAddSubscription} disabled={inspectingKind !== null}>
              {inspectingKind === "subscription" ? "探测中…" : "添加"}
            </button>
          </div>
        </article>

        <article className="entry-card entry-card--direct-nodes">
          <div className="entry-card__icon">⌁</div>
          <div className="entry-card__copy">
            <h3>直接粘贴节点</h3><p>支持 Mihomo YAML、VLESS URI 和 Base64 VLESS 进入严格编译。</p>
          </div>
          <div className="input-row input-row--node-editor">
            <textarea
              aria-label="节点内容"
              aria-describedby="direct-node-input-hint direct-node-line-count"
              value={directNodes}
              onChange={(event) => onDirectNodesChange(event.target.value)}
              placeholder={"vless://…\nvmess://…\ntrojan://…"}
              rows={8}
              wrap="off"
              spellCheck={false}
            />
            <div className="node-editor__footer">
              <span id="direct-node-input-hint">每行一个节点，长链接可横向滚动</span>
              <span id="direct-node-line-count" aria-live="polite">
                {directNodeLineCount} 行
              </span>
              <button
                onClick={onAddDirectNodes}
                disabled={inspectingKind !== null || directNodeLineCount === 0}
              >
                {inspectingKind === "nodes" ? "识别中…" : "识别并添加"}
              </button>
            </div>
          </div>
        </article>

        <article className="entry-card entry-card--bootstrap">
          <div className="entry-card__icon">IP</div>
          <div className="entry-card__copy">
            <h3>域名节点 bootstrap</h3>
            <p>仅为域名 server 填写经受保护路径确认的 IPv4；不会自动直连解析。</p>
          </div>
          <div className="input-row input-row--bootstrap">
            <textarea
              aria-label="域名节点 bootstrap 映射"
              aria-describedby="bootstrap-mapping-hint"
              value={bootstrapMappings}
              onChange={(event) => onBootstrapMappingsChange(event.target.value)}
              placeholder={"1:1=192.0.2.10\n2:3=198.51.100.20"}
              rows={3}
              wrap="off"
              spellCheck={false}
            />
            <span id="bootstrap-mapping-hint">
              每行 source:node=IPv4；来源移除后会清空，映射只进入当前敏感 YAML。
            </span>
          </div>
        </article>
      </div>

      {sourceError && <div className="source-error" role="alert">{sourceError}</div>}

      {inputSources.length > 0 && (
        <div className="source-list" aria-label="已添加来源">
          {inputSources.map((source) => (
            <div className="source-pill" key={source.id}>
              <span className="source-pill__type">
                {source.kind === "subscription" ? "SUB" : "URI"}
              </span>
              <div>
                <strong>{source.name}</strong>
                <small>{source.safeLabel}</small>
                <span className="source-pill__meta">
                  {formatSourceLabel(source.sourceFormat)} · {source.nodeCount} 节点
                  {source.duplicateCount > 0 ? ` · 已去重 ${source.duplicateCount}` : ""}
                  {source.protocols.length > 0
                    ? ` · ${source.protocols.map((item) => `${item.protocol.toUpperCase()} ${item.count}`).join(" / ")}`
                    : ""}
                </span>
                {source.requiresMihomoResolver && (
                  <span className="resolver-badge">需要 Mihomo resolver</span>
                )}
                {source.formatReadyForCompilation
                  && !["mihomo-yaml", "base64-mihomo-yaml"].includes(source.sourceFormat)
                  && <span className="resolver-badge resolver-badge--ready">可转换并编译</span>}
                {!source.formatReadyForCompilation && !source.requiresMihomoResolver
                  && <span className="resolver-badge">
                    {source.protocols.length === 1 && source.protocols[0]?.protocol === "vless"
                      ? "参数预检未通过"
                      : "当前协议尚未接通编译"}
                  </span>}
                {source.warnings.map((warning) => (
                  <span className="source-pill__warning" key={warning}>{warning}</span>
                ))}
              </div>
              <button aria-label={`移除 ${source.name}`} onClick={() => onRemoveSource(source.id)}>
                ×
              </button>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
