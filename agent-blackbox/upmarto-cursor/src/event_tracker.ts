import { Upmarto, NOT_CONFIGURED_MSG } from "@upmarto/sdk";
import type { EventType } from "@upmarto/sdk";

const TEST_COMMAND =
  /\b(cargo test|npm test|pnpm test|yarn test|pytest|jest|vitest|go test|dotnet test)\b/i;

const clients = new Map<string, Promise<Upmarto>>();

function getClient(workspacePath: string): Promise<Upmarto> {
  let pending = clients.get(workspacePath);
  if (!pending) {
    pending = Upmarto.fromWorkspace(workspacePath);
    clients.set(workspacePath, pending);
  }
  return pending;
}

function logHookError(err: unknown): void {
  const message = err instanceof Error ? err.message : NOT_CONFIGURED_MSG;
  process.stderr.write(`[upmarto-cursor] ${message}\n`);
}

export interface HookContext {
  hookName: string;
  workspacePath: string;
  input: Record<string, unknown>;
}

function relPath(workspacePath: string, filePath: string | undefined): string | undefined {
  if (!filePath) return undefined;
  const norm = filePath.replace(/\\/g, "/");
  const ws = workspacePath.replace(/\\/g, "/");
  if (norm.startsWith(ws)) return norm.slice(ws.length + 1);
  return norm;
}

function str(value: unknown): string | undefined {
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

function num(value: unknown): number | undefined {
  return typeof value === "number" ? value : undefined;
}

/** Map Cursor hook payloads to v1 Upmarto events via SDK. */
export function trackHookEvent(ctx: HookContext): void {
  const { hookName, workspacePath, input } = ctx;

  void getClient(workspacePath)
    .then((sdk) => {
      const track = (event_type: EventType, payload: Record<string, unknown>) => {
        sdk.track({ event_type, payload });
      };

      switch (hookName) {
    case "sessionStart":
      track("agent_message", {
        source: "cursor",
        kind: "session_start",
        conversation_id: str(input.conversation_id),
      });
      break;

    case "beforeReadFile": {
      const path = str(input.path) ?? str(input.file_path);
      track("file_opened", {
        path: relPath(workspacePath, path) ?? path,
        source: "cursor",
      });
      break;
    }

    case "afterFileEdit": {
      const path = str(input.path) ?? str(input.file_path);
      track("file_modified", {
        path: relPath(workspacePath, path) ?? path,
        source: "cursor",
        tool: str(input.tool),
      });
      break;
    }

    case "beforeShellExecution":
    case "afterShellExecution": {
      const command = str(input.command);
      if (!command) break;

      track("command_executed", {
        command: command.slice(0, 500),
        source: "cursor",
        phase: hookName === "beforeShellExecution" ? "before" : "after",
        exit_code: num(input.exit_code),
      });

      if (hookName === "afterShellExecution" && TEST_COMMAND.test(command)) {
        const exitCode = num(input.exit_code);
        const base = { command: command.slice(0, 200), source: "cursor" };
        track("test_run", base);
        if (exitCode !== undefined) {
          track(exitCode === 0 ? "test_passed" : "test_failed", {
            ...base,
            exit_code: exitCode,
          });
        }
      }
      break;
    }

    case "afterAgentResponse": {
      const text = str(input.response) ?? str(input.text) ?? str(input.content);
      track("agent_message", {
        source: "cursor",
        preview: text ? text.slice(0, 200) : undefined,
        length: text?.length,
      });
      break;
    }

    case "afterAgentThought": {
      const thought = str(input.thought) ?? str(input.text);
      track("agent_message", {
        source: "cursor",
        kind: "thought",
        preview: thought ? thought.slice(0, 200) : undefined,
      });
      break;
    }

    case "postToolUse": {
      const tool = str(input.tool_name) ?? str(input.tool);
      if (tool === "Write" || tool === "StrReplace") {
        const path = str(input.path) ?? str(input.file_path);
        track("file_modified", {
          path: relPath(workspacePath, path) ?? path,
          source: "cursor",
          tool,
        });
      }
      break;
    }

    default:
      break;
      }
    })
    .catch(logHookError);
}

export function defaultHookResponse(hookName: string): Record<string, unknown> {
  switch (hookName) {
    case "beforeShellExecution":
    case "beforeMCPExecution":
      return { permission: "allow" };
    case "preToolUse":
      return { permission: "allow" };
    case "subagentStart":
      return { permission: "allow" };
    default:
      return {};
  }
}
