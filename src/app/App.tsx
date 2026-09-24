import { useState } from "react";

import { NodeConverterPage } from "../features/node-converter/ui/NodeConverterPage";
import { SourceSection } from "../features/source-inspection/ui/SourceSection";
import {
  MirrorSection,
  ModeSection,
  RuleCatalogSection,
} from "../features/profile-builder/ui/ProfileBuilderSections";
import { PreviewPanel } from "../features/profile-preview/ui/PreviewPanel";
import { BrandMark } from "../shared/ui/BrandMark";
import { useStudioWorkspace } from "./useStudioWorkspace";
import "./App.css";

export default function App() {
  const workspace = useStudioWorkspace();
  const [activeTool, setActiveTool] = useState<"profile" | "converter">("profile");

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="brand">
          <BrandMark />
          <div>
            <strong>Mihomo Studio</strong>
            <span>规则与订阅配置生成器</span>
          </div>
        </div>
        <nav className="tool-switch" aria-label="主要功能">
          <button
            type="button"
            className={activeTool === "profile" ? "tool-switch__item tool-switch__item--active" : "tool-switch__item"}
            aria-current={activeTool === "profile" ? "page" : undefined}
            onClick={() => setActiveTool("profile")}
          >
            配置生成
          </button>
          <button
            type="button"
            className={activeTool === "converter" ? "tool-switch__item tool-switch__item--active" : "tool-switch__item"}
            aria-current={activeTool === "converter" ? "page" : undefined}
            onClick={() => setActiveTool("converter")}
          >
            节点转换
          </button>
        </nav>
        <div className="topbar__status">
          <span className="status-dot" />
          {workspace.desktopRuntime ? "本地解析引擎已连接" : "浏览器界面预览"}
          <button className="icon-button" aria-label="打开设置">•••</button>
        </div>
      </header>

      {activeTool === "profile" ? <div className="workspace">
        <main className="content">
          <section className="hero">
            <div>
              <span className="eyebrow">NEW PROFILE</span>
              <h1>把分散的节点和规则，<br />变成一份清晰的配置。</h1>
              <p>添加来源，选择复杂度，最后导出一份可在 Clash Verge Rev 使用的 Mihomo YAML。</p>
            </div>
            <div className="hero__metric">
              <strong>{workspace.selectedSources.length}</strong>
              <span>已选规则仓库</span>
              <small>目录研究快照 {workspace.catalog.researchedAt}</small>
            </div>
          </section>

          <SourceSection
            subscriptionUrl={workspace.subscriptionUrl}
            directNodes={workspace.directNodes}
            bootstrapMappings={workspace.bootstrapMappings}
            inputSources={workspace.inputSources}
            inspectingKind={workspace.inspectingKind}
            sourceError={workspace.sourceError}
            onSubscriptionChange={workspace.setSubscriptionUrl}
            onDirectNodesChange={workspace.setDirectNodes}
            onBootstrapMappingsChange={workspace.setBootstrapMappings}
            onAddSubscription={workspace.addSubscription}
            onAddDirectNodes={workspace.addDirectNodes}
            onRemoveSource={workspace.removeInputSource}
          />
          <ModeSection mode={workspace.mode} onChange={workspace.changeMode} />
          <RuleCatalogSection
            sources={workspace.shownSources}
            selectedSourceIds={workspace.selectedSourceIds}
            onToggle={workspace.toggleRuleSource}
          />
          <MirrorSection
            enabled={workspace.mirrorEnabled}
            url={workspace.mirrorUrl}
            onEnabledChange={workspace.setMirrorEnabled}
            onUrlChange={workspace.setMirrorUrl}
          />
          <PreviewPanel
            draft={workspace.draft}
            selectedSourceCount={workspace.selectedSources.length}
            desktopRuntime={workspace.desktopRuntime}
            isCompiling={workspace.isCompiling}
            compiledProfile={workspace.compiledProfile}
            compileError={workspace.compileError}
            onCompile={workspace.compileDraft}
          />
        </main>
      </div> : <NodeConverterPage desktopRuntime={workspace.desktopRuntime} />}
    </div>
  );
}
