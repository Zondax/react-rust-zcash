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
