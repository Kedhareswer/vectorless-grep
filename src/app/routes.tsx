import { useEffect, useState } from "react";

import { App } from "./App";
import { AppShell } from "./AppShell";
import { SettingsPage } from "./SettingsPage";

function normalizePath(path: string): "/" | "/settings" {
  return path === "/settings" ? "/settings" : "/";
}

export function AppRoutes() {
  const [path, setPath] = useState<"/" | "/settings">(normalizePath(window.location.pathname));

  useEffect(() => {
    const onPopState = () => {
      setPath(normalizePath(window.location.pathname));
    };
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  const navigate = (nextPath: "/" | "/settings") => {
    if (nextPath === path) {
      return;
    }
    window.history.pushState({}, "", nextPath);
    setPath(nextPath);
  };

  return (
    <AppShell path={path} navigate={navigate}>
      {path === "/settings" ? <SettingsPage navigate={navigate} /> : <App />}
    </AppShell>
  );
}
