import { useEffect, useRef, useState } from "react";

import { convertNodeText } from "../../../shared/api/studio";
import type { ConvertNodeTextResult } from "../../../shared/contracts";
import { safeErrorMessage } from "../../../shared/lib/presentation";
import { countConverterInputLines } from "../model/input";
import { ConversionRequestGuard } from "../model/requestGuard";

interface NodeConverterPageProps {
  desktopRuntime: boolean;
}

interface NodeConverterResultProps {
  result: ConvertNodeTextResult;
}

export function NodeConverterStatus({ result }: NodeConverterResultProps) {
  return (
    <div className="converter-result__summary" role="status">
      <div>
        <strong>节点片段已生成</strong>
        <span>
          {result.nodeCount} 个节点
          {result.duplicateNodeCount > 0 ? ` · 已去重 ${result.duplicateNodeCount}` : ""}
        </span>
      </div>
      <span className="resolver-badge resolver-badge--ready">
        {result.templateYaml ? "完整模版可用" : "仅 proxies 片段"}
      </span>
    </div>
  );
}

export function NodeConverterResult({ result }: NodeConverterResultProps) {
  const [outputKind, setOutputKind] = useState<"template" | "nodes">(
    result.templateYaml ? "template" : "nodes",
  );
  const [copyStatus, setCopyStatus] = useState("");
  const showingTemplate = outputKind === "template" && result.templateYaml !== null;
  const output = showingTemplate && result.templateYaml ? result.templateYaml : result.yaml;

  async function copyOutput() {
    try {
      await navigator.clipboard.writeText(output);
      setCopyStatus(showingTemplate ? "完整配置已复制" : "节点片段已复制");
    } catch {
      setCopyStatus("复制失败，请从下方文本框手动复制。");
    }
  }

  return (
    <div className="converter-result">
      <NodeConverterStatus result={result} />
      {result.warnings.map((warning) => (
        <p className="converter-result__warning" key={warning}>{warning}</p>
      ))}
      {result.templateYaml && (
        <div className="converter-result__modes" role="group" aria-label="转换结果类型">
          <button type="button" aria-pressed={showingTemplate} onClick={() => { setOutputKind("template"); setCopyStatus(""); }}>
            内置模版 · 完整配置
          </button>
          <button type="button" aria-pressed={!showingTemplate} onClick={() => { setOutputKind("nodes"); setCopyStatus(""); }}>
            仅节点片段
          </button>
        </div>
      )}
      {showingTemplate && (
        <p className="converter-result__context">
          已替换模版中的唯一节点，并同步策略组引用。建议保存为 <code>{result.templateFileName}</code>。
          模版沿用原文件的 DNS 与规则设置，未进行严格隐私或 Mihomo 运行验证。
        </p>
      )}
      {!result.templateYaml && (
        <p className="converter-result__context">
          内置模版适用于转换后仅有一个节点，且服务器为 IP 的结果；当前仍可复制节点片段。
        </p>
      )}
      <div className="converter-result__disclosure">
        <textarea
          aria-label={showingTemplate ? "内置模版完整 Clash Mihomo YAML" : "转换后的 Clash Mihomo 节点 YAML"}
          readOnly
          spellCheck={false}
          value={output}
        />
        <button
          type="button"
          className="copy-button"
          onClick={copyOutput}
        >
          {showingTemplate ? "复制完整配置" : "复制节点 YAML"}
        </button>
        {copyStatus && <p className="converter-result__feedback" role="status">{copyStatus}</p>}
      </div>
      <small>结果尚未导入 Clash Verge，也未验证节点连接或最终出口。</small>
    </div>
  );
}

export function NodeConverterPage({ desktopRuntime }: NodeConverterPageProps) {
  const [input, setInput] = useState("");
  const [result, setResult] = useState<ConvertNodeTextResult | null>(null);
  const [error, setError] = useState("");
  const [isConverting, setIsConverting] = useState(false);
  const requestGuard = useRef(new ConversionRequestGuard());
  const lineCount = countConverterInputLines(input);

  useEffect(() => () => {
    requestGuard.current.invalidate();
  }, []);

  function changeInput(value: string) {
    requestGuard.current.invalidate();
    setInput(value);
    setResult(null);
    setError("");
    setIsConverting(false);
  }

  function clearInput() {
    requestGuard.current.invalidate();
    setInput("");
    setResult(null);
    setError("");
    setIsConverting(false);
  }

  async function convert() {
    const value = input.trim();
    if (!value) return;
    if (!desktopRuntime) {
      setResult(null);
      setError("节点转换请在 Tauri 桌面运行时中执行。");
      return;
    }

    const revision = requestGuard.current.begin();
    setIsConverting(true);
    setError("");
    try {
      const converted = await convertNodeText(value);
      if (requestGuard.current.isCurrent(revision)) {
        setResult(converted);
      }
    } catch (conversionError) {
      if (requestGuard.current.isCurrent(revision)) {
        setResult(null);
        setError(safeErrorMessage(conversionError, "节点转换未完成。"));
      }
    } finally {
      if (requestGuard.current.isCurrent(revision)) {
        setIsConverting(false);
      }
    }
  }

  return (
    <main className="content content--converter">
      <section className="converter-hero">
        <span className="eyebrow">NODE CONVERTER</span>
        <h1>把节点分享内容，<br />转换成节点 YAML。</h1>
        <p>
          独立转换单条、多行或 Base64 VLESS 内容，以及 WireGuard 配置，输出可合并进 Clash/Mihomo 配置的
          <code> proxies:</code> 片段。
        </p>
      </section>

      <section className="converter-workbench" aria-busy={isConverting}>
        <div className="converter-workbench__header">
          <div className="entry-card__icon">⇄</div>
          <div>
            <h2>节点转换</h2>
            <p>转换在本机内存完成；节点内容、凭据和结果不会写入日志或持久化。</p>
          </div>
          <span className="health">独立工具</span>
        </div>

        <label className="converter-field">
          <span>VLESS 或 WireGuard 节点内容</span>
          <textarea
            aria-label="待转换的 VLESS 或 WireGuard 节点内容"
            aria-describedby="converter-input-hint converter-line-count"
            value={input}
            onChange={(event) => changeInput(event.target.value)}
            placeholder={"vless://…\nvless://…\n\n或粘贴包含 [Interface] / [Peer] 的 WireGuard 配置"}
            rows={10}
            wrap="off"
            spellCheck={false}
          />
        </label>

        <div className="converter-actions">
          <span id="converter-input-hint">
            VLESS 可每行一个或使用 Base64；WireGuard 可粘贴一份 [Interface] / [Peer] 配置
          </span>
          <span id="converter-line-count" aria-live="polite">{lineCount} 行</span>
          <button
            type="button"
            className="converter-clear"
            onClick={clearInput}
            disabled={isConverting || lineCount === 0}
          >
            清空
          </button>
          <button
            type="button"
            className="converter-submit"
            onClick={convert}
            disabled={isConverting || lineCount === 0}
          >
            {isConverting ? "正在转换…" : "转换为 proxies YAML"}<span>→</span>
          </button>
        </div>

        {error && (
          <div className="converter-state converter-state--error" role="alert" aria-live="assertive">
            <strong>转换未完成</strong>
            <span>{error}</span>
          </div>
        )}

        {result && <NodeConverterResult result={result} />}
      </section>

      <p className="converter-footnote">
        转换器与配置生成工作区相互独立；转换结果不会自动加入节点来源。
      </p>
    </main>
  );
}
