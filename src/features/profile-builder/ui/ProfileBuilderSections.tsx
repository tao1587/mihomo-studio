import type { RuleSource, StudioMode } from "../../../shared/contracts";

interface ModeSectionProps {
  mode: StudioMode;
  onChange: (mode: StudioMode) => void;
}

export function ModeSection({ mode, onChange }: ModeSectionProps) {
  return (
    <section id="mode" className="section-block">
      <div className="section-heading">
        <div><span className="step-tag">步骤 2</span><h2>选择配置复杂度</h2></div>
      </div>
      <div className="mode-switch" role="radiogroup" aria-label="配置模式">
        <button
          className={mode === "simple" ? "mode-card mode-card--active" : "mode-card"}
          onClick={() => onChange("simple")}
          role="radio"
          aria-checked={mode === "simple"}
        >
          <span className="mode-card__radio" />
          <div><strong>简单模式</strong><p>保守广告、AI、流媒体、国内直连和自动地区组。</p></div>
          <span className="recommended">推荐</span>
        </button>
        <button
          className={mode === "full" ? "mode-card mode-card--active" : "mode-card"}
          onClick={() => onChange("full")}
          role="radio"
          aria-checked={mode === "full"}
        >
          <span className="mode-card__radio" />
          <div><strong>完全模式</strong><p>开放规则源优先级、细分服务、DNS/TUN 和高级去重。</p></div>
        </button>
      </div>
    </section>
  );
}

interface RuleCatalogSectionProps {
  sources: RuleSource[];
  selectedSourceIds: string[];
  onToggle: (id: string) => void;
}

export function RuleCatalogSection({
  sources,
  selectedSourceIds,
  onToggle,
}: RuleCatalogSectionProps) {
  return (
    <section className="section-block">
      <div className="section-heading">
        <div><span className="step-tag">规则目录</span><h2>开源规则来源</h2></div>
        <span className="section-hint">完整模式可查看全部</span>
      </div>
      <div className="rule-grid">
        {sources.map((source) => {
          const selected = selectedSourceIds.includes(source.id);
          return (
            <button
              className={`${selected ? "rule-card rule-card--selected" : "rule-card"}${source.selectionKind === "upstream-data" ? " rule-card--reference" : ""}`}
              key={source.id}
              onClick={() => onToggle(source.id)}
              aria-pressed={selected}
              disabled={source.selectionKind === "upstream-data"}
            >
              <div className="rule-card__top">
                <span className={`maintenance maintenance--${source.maintenance}`}>
                  {source.maintenance === "active" ? "活跃" : source.maintenance === "stable" ? "稳定" : "观察"}
                </span>
                <span className="checkmark">{selected ? "✓" : "+"}</span>
              </div>
              <strong>{source.name}</strong>
              <p>{source.summary}</p>
              <div className="tag-row">
                {source.categories.slice(0, 3).map((category) => <span key={category}>{category}</span>)}
              </div>
              <div className="rule-meta">
                <small>{source.license}</small>
                <small>★ {source.popularity.stars.toLocaleString()} · Fork {source.popularity.forks.toLocaleString()}</small>
              </div>
            </button>
          );
        })}
      </div>
    </section>
  );
}

interface MirrorSectionProps {
  enabled: boolean;
  url: string;
  onEnabledChange: (enabled: boolean) => void;
  onUrlChange: (url: string) => void;
}

export function MirrorSection({
  enabled,
  url,
  onEnabledChange,
  onUrlChange,
}: MirrorSectionProps) {
  return (
    <section className="section-block mirror-block">
      <div>
        <span className="step-tag">下载路径</span>
        <h2>GitHub 镜像</h2>
        <p>只改变下载地址，配置仍保留 canonical URL 和来源 hash。</p>
      </div>
      <label className="switch-row">
        <input
          type="checkbox"
          checked={enabled}
          onChange={(event) => onEnabledChange(event.target.checked)}
        />
        <span className="toggle" />
        <span>启用镜像</span>
      </label>
      <input
        aria-label="GitHub 镜像地址"
        disabled={!enabled}
        value={url}
        onChange={(event) => onUrlChange(event.target.value)}
      />
    </section>
  );
}
