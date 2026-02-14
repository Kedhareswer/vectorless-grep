import { useMemo, useState } from "react";

import { setProviderKey } from "../lib/tauriApi";

interface UiSettings {
  defaultModel: string;
  traceMode: "timeline";
  sqlDebug: boolean;
}

const SETTINGS_STORAGE_KEY = "vectorless.ui.settings";

function readSettings(): UiSettings {
  try {
    const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (!raw) {
      return { defaultModel: "gemini-2.0-flash", traceMode: "timeline", sqlDebug: false };
    }
    const parsed = JSON.parse(raw) as Partial<UiSettings>;
    return {
      defaultModel: parsed.defaultModel || "gemini-2.0-flash",
      traceMode: "timeline",
      sqlDebug: Boolean(parsed.sqlDebug),
    };
  } catch {
    return { defaultModel: "gemini-2.0-flash", traceMode: "timeline", sqlDebug: false };
  }
}

function writeSettings(settings: UiSettings): void {
  localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
}

interface SettingsPageProps {
  navigate: (path: "/" | "/settings") => void;
}

export function SettingsPage({ navigate }: SettingsPageProps) {
  const initial = useMemo(() => readSettings(), []);
  const [apiKey, setApiKey] = useState("");
  const [defaultModel, setDefaultModel] = useState(initial.defaultModel);
  const [sqlDebug, setSqlDebug] = useState(initial.sqlDebug);
  const [status, setStatus] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const saveProvider = async () => {
    if (!apiKey.trim()) {
      setStatus("API key is required.");
      return;
    }
    setSaving(true);
    try {
      await setProviderKey(apiKey.trim());
      setStatus("API key saved.");
      setApiKey("");
    } catch (error) {
      setStatus(`Failed to save key: ${String(error)}`);
    } finally {
      setSaving(false);
    }
  };

  const saveUiSettings = () => {
    writeSettings({
      defaultModel,
      traceMode: "timeline",
      sqlDebug,
    });
    setStatus("Settings saved. Restart app to apply logging changes.");
  };

  return (
    <main className="settings-page">
      <section className="settings-card">
        <header>
          <h2>Settings</h2>
          <button type="button" className="back-link" onClick={() => navigate("/")}>
            Back to Workspace
          </button>
        </header>

        <div className="settings-section">
          <h3>Provider Key</h3>
          <p>Store your Gemini API key securely using OS keychain.</p>
          <div className="settings-row">
            <input
              type="password"
              value={apiKey}
              onChange={(event) => setApiKey(event.target.value)}
              placeholder="Enter Gemini API key"
            />
            <button type="button" onClick={() => void saveProvider()} disabled={saving}>
              {saving ? "Saving..." : "Save Key"}
            </button>
          </div>
        </div>

        <div className="settings-section">
          <h3>Model & Runtime</h3>
          <p>Frontend defaults used by workspace UI controls.</p>
          <label className="settings-label">
            Default model
            <input
              value={defaultModel}
              onChange={(event) => setDefaultModel(event.target.value)}
              placeholder="gemini-2.0-flash"
            />
          </label>
          <label className="settings-check">
            <input
              type="checkbox"
              checked={sqlDebug}
              onChange={(event) => setSqlDebug(event.target.checked)}
            />
            Enable SQL debug logs (sets expectation for `VECTORLESS_SQLX_DEBUG=1`)
          </label>
          <button type="button" onClick={saveUiSettings}>
            Save UI Settings
          </button>
        </div>

        {status ? <p className="settings-status">{status}</p> : null}
      </section>
    </main>
  );
}
