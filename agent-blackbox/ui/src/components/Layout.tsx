import { Link, useLocation } from "react-router-dom";
import { useI18n } from "../i18n";

interface LayoutProps {
  children: React.ReactNode;
  sessionId?: string;
}

export function Layout({ children, sessionId }: LayoutProps) {
  const { pathname } = useLocation();
  const { t, locale, setLocale } = useI18n();

  return (
    <div className="app-shell">
      <header className="app-header">
        <h1>{t.app.title}</h1>
        <nav>
          <Link to="/sessions" className={pathname.startsWith("/sessions") || pathname === "/" ? "active" : ""}>
            {t.nav.sessions}
          </Link>
          {sessionId && (
            <>
              <Link
                to={`/timeline/${sessionId}`}
                className={pathname.startsWith("/timeline") ? "active" : ""}
              >
                {t.nav.timeline}
              </Link>
              <Link
                to={`/explain/${sessionId}`}
                className={pathname.startsWith("/explain") ? "active" : ""}
              >
                {t.nav.why}
              </Link>
            </>
          )}
        </nav>
        <div className="header-actions">
          <div className="lang-switch" role="group" aria-label="Language">
            <button
              type="button"
              className={locale === "tr" ? "active" : ""}
              onClick={() => setLocale("tr")}
            >
              {t.lang.tr}
            </button>
            <button
              type="button"
              className={locale === "en" ? "active" : ""}
              onClick={() => setLocale("en")}
            >
              {t.lang.en}
            </button>
          </div>
          <span className="version-tag">{t.app.apiVersion}</span>
        </div>
      </header>
      <main className="app-main">{children}</main>
    </div>
  );
}
