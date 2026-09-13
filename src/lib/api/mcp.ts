import { commands } from "../bindings";
import type { CallEntry, McpStatus } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function mcpStatus(): Promise<McpStatus> {
  return unwrap(await commands.mcpStatus());
}

/** Every mutation answers with the whole status: the port is only known after
 *  the server binds, so the caller would have to ask again anyway. */
export async function setMcpEnabled(enabled: boolean): Promise<McpStatus> {
  return unwrap(await commands.setMcpEnabled(enabled));
}

export async function setMcpTool(name: string, enabled: boolean): Promise<McpStatus> {
  return unwrap(await commands.setMcpTool(name, enabled));
}

export async function setMcpWindow(months: number): Promise<McpStatus> {
  return unwrap(await commands.setMcpWindow(months));
}

export async function mcpRecentCalls(): Promise<CallEntry[]> {
  return unwrap(await commands.mcpRecentCalls());
}
