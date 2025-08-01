// Type declarations for node-pty
// These are simplified types - the actual library has more detailed types

declare module 'node-pty' {
  export interface IPtyForkOptions {
    name?: string;
    cols?: number;
    rows?: number;
    cwd?: string;
    env?: { [key: string]: string };
    encoding?: string;
  }

  export interface IPty {
    pid: number;
    cols: number;
    rows: number;
    process: string;
    handleFlowControl: boolean;

    write(data: string | Buffer): void;
    resize(cols: number, rows: number): void;
    destroy(): void;
    kill(signal?: string): void;
    pause(): void;
    resume(): void;

    on(event: 'data', listener: (data: string) => void): this;
    on(event: 'exit', listener: (exitCode: number, signal?: number) => void): this;
    on(event: string, listener: (...args: any[]) => void): this;
  }

  export function spawn(shell: string, args?: string[], options?: IPtyForkOptions): IPty;
}