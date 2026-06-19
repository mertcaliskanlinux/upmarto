import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { LanguageProvider } from "./i18n";
import { ExplainPage } from "./pages/ExplainPage";
import { SessionsPage } from "./pages/SessionsPage";
import { TimelinePage } from "./pages/TimelinePage";
import "./styles/global.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <LanguageProvider>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Navigate to="/sessions" replace />} />
        <Route path="/sessions" element={<SessionsPage />} />
        <Route path="/timeline/:sessionId" element={<TimelinePage />} />
        <Route path="/explain/:sessionId" element={<ExplainPage />} />
      </Routes>
    </BrowserRouter>
    </LanguageProvider>
  </StrictMode>,
);
