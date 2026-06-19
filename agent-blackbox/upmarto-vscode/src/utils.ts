import * as vscode from "vscode";

/** Relative path from workspace root, forward slashes. */
export function relPath(workspacePath: string, filePath: string): string {
  const norm = filePath.replace(/\\/g, "/");
  const ws = workspacePath.replace(/\\/g, "/");
  if (norm.startsWith(ws)) {
    const rel = norm.slice(ws.length + 1);
    return rel || norm;
  }
  return norm;
}

export function workspacePathForUri(uri: vscode.Uri): string | undefined {
  return vscode.workspace.getWorkspaceFolder(uri)?.uri.fsPath;
}

export function isTrackableFileUri(uri: vscode.Uri): boolean {
  return uri.scheme === "file";
}

export const TEST_COMMAND =
  /\b(cargo test|npm test|pnpm test|yarn test|pytest|jest|vitest|go test|dotnet test)\b/i;
