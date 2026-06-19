import * as vscode from "vscode";

import { EventTracker } from "./event_tracker.js";

let tracker: EventTracker | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const enabled = vscode.workspace
    .getConfiguration("upmarto")
    .get<boolean>("enabled", true);
  if (!enabled) {
    return;
  }

  if (!vscode.workspace.workspaceFolders?.length) {
    return;
  }

  const output = vscode.window.createOutputChannel("Upmarto");
  context.subscriptions.push(output);

  tracker = new EventTracker(output);
  tracker.start(context);

  output.appendLine(
    "Upmarto VS Code extension active — config via .upmarto/config.json (@upmarto/sdk)",
  );
}

export async function deactivate(): Promise<void> {
  if (tracker) {
    await tracker.flushAll();
    tracker = undefined;
  }
}
