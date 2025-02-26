export type ChangeEventPayload = {
  value: string;
};

export type MyRustModuleViewProps = {
  name: string;
};

export interface TransparentInput {
  outp: string;
  pk: string;
  address: string;
  value: number;
}

export interface TransparentOutput {
  address: string;
  value: number;
}

export interface TinData {
  path: number[]; // Array of 5 uint32 values
  address: string; // Hex-encoded string representing a Script
  value: number; // uint64 value represented as a number in JS
}

export interface ToutData {
  address: string; // Hex-encoded string representing a Script
  value: number; // uint64 value represented as a number in JS
}

export interface SaplingInData {
  path: number; // Single uint32 value
  address: string; // Hex-encoded string representing a PaymentAddress
  value: number; // uint64 value represented as a number in JS
}

export interface SaplingOutData {
  address: string; // Hex-encoded string representing a PaymentAddress
  value: number; // uint64 value represented as a number in JS
  memoType: number; // uint8 value (0-255)
  ovk?: Uint8Array; // Optional 32-byte OutgoingViewingKey
}

export interface InitData {
  tIn: TinData[];
  tOut: ToutData[];
  sSpend: SaplingInData[];
  sOutput: SaplingOutData[];
}
