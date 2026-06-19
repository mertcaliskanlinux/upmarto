import { Upmarto } from "@upmarto/sdk";
import type { EventType } from "@upmarto/sdk";
import * as vscode from "vscode";

import { isTrackableFileUri, relPath, TEST_COMMAND, workspacePathForUri } from "./utils.js";

const MODIFY_DEBOUNCE_MS = 400;

export class EventTracker {
  private readonly clients = new Map<string, Promise<Upmarto | null>>();
  private readonly modifyTimers = new Map<string, ReturnType<typeof setTimeout>>();
  private readonly subscriptions: vscode.Disposable[] = [];

  constructor(private readonly output: vscode.OutputChannel) {}

  get disposables(): readonly vscode.Disposable[] {
    return this.subscriptions;
  }

  /** Wire VS Code API listeners — all handlers are non-blocking (SDK queue only). */
  start(context: vscode.ExtensionContext): void {
    for (const folder of vscode.workspace.workspaceFolders ?? []) {
      this.ensureClient(folder.uri.fsPath);
    }

    this.subscriptions.push(
      vscode.workspace.onDidOpenTextDocument((doc) => this.onDocumentOpened(doc)),
      vscode.workspace.onDidChangeTextDocument((e) => this.onDocumentChanged(e)),
      vscode.workspace.onDidCreateFiles((e) => this.onFilesCreated(e)),
      vscode.workspace.onDidSaveTextDocument((doc) => this.onDocumentSaved(doc)),
      vscode.tasks.onDidEndTaskProcess((e) => this.onTaskEnded(e)),
      vscode.window.onDidCloseTerminal((term) => this.onTerminalClosed(term)),
    );

    this.registerTerminalHooks();

    context.subscriptions.push(...this.subscriptions);

    const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (root) {
      this.track(root, "agent_message", {
        source: "vscode",
        kind: "extension_activated",
      });
    }
  }

  async flushAll(): Promise<void> {
    await Promise.all(
      [...this.clients.values()].map(async (pending) => {
        const client = await pending;
        if (client) await client.flush().catch(() => undefined);
      }),
    );
  }

  private ensureClient(workspacePath: string): Promise<Upmarto | null> {
    const existing = this.clients.get(workspacePath);
    if (existing) return existing;

    const pending = Upmarto.fromWorkspace(workspacePath)
        .then((client) => {
          this.output.appendLine(`Upmarto: capturing for ${workspacePath}`);
          return client;
        })
        .catch((err) => {
          const msg = err instanceof Error ? err.message : String(err);
          this.output.appendLine(`Upmarto: skipped ${workspacePath} — ${msg}`);
          this.clients.delete(workspacePath);
          return null;
        });
    this.clients.set(workspacePath, pending);
    return pending;
  }

  private track(
    workspacePath: string,
    event_type: EventType,
    payload: Record<string, unknown>,
  ): void {
    void this.ensureClient(workspacePath).then((client) => {
      if (!client) return;
      try {
        client.track({ event_type, payload });
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        this.output.appendLine(`Upmarto track error: ${msg}`);
      }
    });
  }

  private onDocumentOpened(doc: vscode.TextDocument): void {
    if (!isTrackableFileUri(doc.uri)) return;
    const ws = workspacePathForUri(doc.uri);
    if (!ws) return;

    this.track(ws, "file_opened", {
      path: relPath(ws, doc.uri.fsPath),
      source: "vscode",
      language: doc.languageId,
    });
  }

  private onDocumentChanged(e: vscode.TextDocumentChangeEvent): void {
    if (!isTrackableFileUri(e.document.uri)) return;
    if (e.contentChanges.length === 0) return;
    const ws = workspacePathForUri(e.document.uri);
    if (!ws) return;

    const key = e.document.uri.toString();
    const existing = this.modifyTimers.get(key);
    if (existing) clearTimeout(existing);

    const timer = setTimeout(() => {
      this.modifyTimers.delete(key);
      this.track(ws, "file_modified", {
        path: relPath(ws, e.document.uri.fsPath),
        source: "vscode",
        change_count: e.contentChanges.length,
      });
    }, MODIFY_DEBOUNCE_MS);

    this.modifyTimers.set(key, timer);
  }

  private onDocumentSaved(doc: vscode.TextDocument): void {
    if (!isTrackableFileUri(doc.uri)) return;
    const ws = workspacePathForUri(doc.uri);
    if (!ws) return;

    this.track(ws, "file_modified", {
      path: relPath(ws, doc.uri.fsPath),
      source: "vscode",
      saved: true,
    });
  }

  private onFilesCreated(e: vscode.FileCreateEvent): void {
    for (const uri of e.files) {
      if (!isTrackableFileUri(uri)) continue;
      const ws = workspacePathForUri(uri);
      if (!ws) continue;

      this.track(ws, "file_created", {
        path: relPath(ws, uri.fsPath),
        source: "vscode",
      });
    }
  }

  private onTaskEnded(e: vscode.TaskProcessEndEvent): void {
    const ws = this.taskWorkspace(e.execution);
    if (!ws) return;

    const name = e.execution.task.name;
    const command = name.slice(0, 500);

    this.track(ws, "command_executed", {
      command,
      source: "vscode",
      kind: "task",
      exit_code: e.exitCode,
    });

    if (TEST_COMMAND.test(name)) {
      const base = { command: command.slice(0, 200), source: "vscode" };
      this.track(ws, "test_run", base);
      if (e.exitCode === 0) {
        this.track(ws, "test_passed", { ...base, test: name });
      } else {
        this.track(ws, "test_failed", {
          ...base,
          test: name,
          exit_code: e.exitCode,
        });
      }
    }
  }

  private onTerminalClosed(term: vscode.Terminal): void {
    const ws = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!ws) return;

    const name = term.name?.slice(0, 200);
    if (!name) return;

    this.track(ws, "command_executed", {
      command: name,
      source: "vscode",
      kind: "terminal",
    });
  }

  /** Shell integration hooks (VS Code 1.93+) — command_executed with real shell line. */
  private registerTerminalHooks(): void {
    const win = vscode.window as typeof vscode.window & {
      onDidStartTerminalShellExecution?: (
        listener: (e: TerminalShellExecutionStartEvent) => unknown,
      ) => vscode.Disposable;
    };

    if (typeof win.onDidStartTerminalShellExecution !== "function") return;

    this.subscriptions.push(
      win.onDidStartTerminalShellExecution((e: TerminalShellExecutionStartEvent) => {
        const ws = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (!ws) return;

        const command = e.execution?.commandLine?.value?.slice(0, 500);
        if (!command) return;

        this.track(ws, "command_executed", {
          command,
          source: "vscode",
          kind: "terminal_shell",
        });

        if (TEST_COMMAND.test(command)) {
          const base = { command: command.slice(0, 200), source: "vscode" };
          this.track(ws, "test_run", base);
        }
      }),
    );
  }

  private taskWorkspace(execution: vscode.TaskExecution): string | undefined {
    const scope = execution.task.scope;
    if (typeof scope === "object" && scope !== null && "uri" in scope) {
      return (scope as vscode.WorkspaceFolder).uri.fsPath;
    }
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  }
}

/** Minimal typing for TerminalShellExecution API (not in older @types/vscode). */
interface TerminalShellExecutionStartEvent {
  execution?: { commandLine?: { value?: string } };
}
