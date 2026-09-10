declare namespace NodeJS {
  interface ProcessEnv {
    [key: string]: string | undefined;
  }
  interface Process {
    env: ProcessEnv;
  }
}

declare const process: NodeJS.Process | undefined;
declare const Buffer: {
  isBuffer(obj: any): boolean;
} | undefined;
