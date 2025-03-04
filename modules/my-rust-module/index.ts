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

// Export the native constant values
export const PI = MyRustModule.PI;
export const NETWORK_MAINNET = MyRustModule.NETWORK_MAINNET;
export const NETWORK_TESTNET = MyRustModule.NETWORK_TESTNET;

// Interface for transparent input
export interface TransparentInput {
  outp: string;
  pk: string;
  address: string;
  value: number;
}

// Interface for transparent output
export interface TransparentOutput {
  address: string;
  value: number;
}

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

// Export transaction builder functions
export async function createBuilder(
  fee: number,
  networkType: number,
): Promise<number> {
  console.log("Creating builder with fee:", fee, "networkType:", networkType);
  return await MyRustModule.createBuilder(fee, networkType);
}

export async function destroyBuilder(builderId: number): Promise<number> {
  console.log("Destroying builder:", builderId);
  return await MyRustModule.destroyBuilder(builderId);
}

export async function addTransparentInput(
  builderId: number,
  input: TransparentInput,
): Promise<number> {
  console.log("Adding transparent input to builder:", builderId, input);
  return await MyRustModule.addTransparentInput(builderId, input);
}

export async function addTransparentOutput(
  builderId: number,
  output: TransparentOutput,
): Promise<number> {
  console.log("Adding transparent output to builder:", builderId, output);
  return await MyRustModule.addTransparentOutput(builderId, output);
}

export async function buildTransaction(
  builderId: number,
  spendPath: string,
  outputPath: string,
  txVersion: number,
): Promise<string> {
  console.log("Building transaction with builder:", builderId);
  return await MyRustModule.buildTransaction(
    builderId,
    spendPath,
    outputPath,
    txVersion,
  );
}

export async function finalizeTransaction(builderId: number): Promise<string> {
  console.log("Finalizing transaction with builder:", builderId);
  return await MyRustModule.finalizeTransaction(builderId);
}

export async function getErrorDescription(errorCode: number): Promise<string> {
  return await MyRustModule.getErrorDescription(errorCode);
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
