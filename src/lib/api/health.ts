import { commands } from "../bindings";
import type { HealthInfo } from "../bindings";

export async function healthCheck(): Promise<HealthInfo> {
  const result = await commands.healthCheck();
  if (result.status === "error") {
    throw new Error(result.error);
  }
  return result.data;
}
