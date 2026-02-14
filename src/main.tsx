import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { AppRoutes } from "./app/routes";
import "./styles/tokens.css";
import "./styles/base.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <AppRoutes />
  </StrictMode>,
);
