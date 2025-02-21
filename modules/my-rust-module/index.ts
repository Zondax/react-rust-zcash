import {
  NativeModulesProxy,
  EventEmitter,
  Subscription,
} from "expo-modules-core";

// Import the native module. On web, it will be resolved to MyRustModule.web.ts
// and on native platforms to MyRustModule.ts
import MyRustModule from "./src/MyRustModule";
import MyRustModuleView from "./src/MyRustModuleView";
import {
  ChangeEventPayload,
  MyRustModuleViewProps,
} from "./src/MyRustModule.types";
// Export the native constant value
export const PI = MyRustModule.PI;

// Export the fee calculation function
export async function calculateFee(
  nTxin: number,
  nTxout: number,
  nSpend: number,
  nSout: number,
): Promise<number> {
  console.log("Calling native calculateFee with:", {
    nTxin,
    nTxout,
    nSpend,
    nSout,
  });
  return await MyRustModule.calculateFee(nTxin, nTxout, nSpend, nSout);
}

export async function setValueAsync(value: string) {
  return await MyRustModule.setValueAsync(value);
}

const emitter = new EventEmitter(MyRustModule);

export function addChangeListener(
  listener: (event: ChangeEventPayload) => void,
): Subscription {
  return emitter.addListener<ChangeEventPayload>("onChange", listener);
}

export { MyRustModuleView, MyRustModuleViewProps, ChangeEventPayload };
