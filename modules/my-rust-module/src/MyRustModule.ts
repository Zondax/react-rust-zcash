import { requireNativeModule } from "expo-modules-core";
import {
  TransparentInput,
  TransparentOutput,
  Signatures,
} from "./MyRustModule.types";

// It loads the native module object from the JSI or falls back to
// the bridge module (from NativeModulesProxy) if the remote debugger is on.
const MyRustModule = requireNativeModule("MyRustModule");

export default {
  PI: MyRustModule.PI,
  NETWORK_MAINNET: MyRustModule.NETWORK_MAINNET,
  NETWORK_TESTNET: MyRustModule.NETWORK_TESTNET,
  calculateFee: (
    nTxin: number,
    nTxout: number,
    nSpend: number,
    nSout: number,
  ) => MyRustModule.calculateFee(nTxin, nTxout, nSpend, nSout),
  createBuilder: (fee: number, networkType: number) =>
    MyRustModule.createBuilder(fee, networkType),
  destroyBuilder: (builderId: number) => MyRustModule.destroyBuilder(builderId),
  addTransparentInput: (builderId: number, input: TransparentInput) =>
    MyRustModule.addTransparentInput(builderId, input),
  addTransparentOutput: (builderId: number, output: TransparentOutput) =>
    MyRustModule.addTransparentOutput(builderId, output),
  buildTransaction: (
    builderId: number,
    spendPath: string,
    outputPath: string,
    txVersion: number,
  ) =>
    MyRustModule.buildTransaction(builderId, spendPath, outputPath, txVersion),
  addSignatures: (builderId: number, signatures: Signatures) =>
    MyRustModule.addSignatures(builderId, signatures),
  finalizeTransaction: (builderId: number) =>
    MyRustModule.finalizeTransaction(builderId),
  getErrorDescription: (errorCode: number) =>
    MyRustModule.getErrorDescription(errorCode),
  setValueAsync: (value: string) => MyRustModule.setValueAsync(value),
} as const;
