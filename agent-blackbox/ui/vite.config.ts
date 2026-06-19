import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const proxyTarget = env.VITE_API_PROXY_TARGET?.trim();
  const devPort = parseInt(env.VITE_DEV_PORT ?? "", 10);

  const server: import("vite").UserConfig["server"] = {};

  if (Number.isFinite(devPort) && devPort > 0) {
    server.port = devPort;
  }

  if (proxyTarget) {
    const routes = ["/config", "/event", "/timeline", "/explain", "/session", "/project", "/debug"];
    server.proxy = Object.fromEntries(
      routes.map((route) => [route, { target: proxyTarget, changeOrigin: true }]),
    );
  }

  return {
    plugins: [react()],
    server,
  };
});
