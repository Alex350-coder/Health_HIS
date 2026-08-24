import { spawn, spawnSync, type ChildProcess } from 'node:child_process';

/**
 * WebdriverIO + tauri-driver E2E harness (Architecture.md Section "Testing", DevelopmentWorkflow.md
 * `npm run test:e2e`). Follows the official Tauri WebDriver example: `tauri-driver` proxies
 * WebDriver sessions to the native WebView2 driver, so no browser is launched — the actual
 * compiled desktop binary is driven directly.
 *
 * Prerequisites (not installable via npm, see DevelopmentWorkflow.md "Running the E2E suite
 * locally"): `cargo install tauri-driver` and the Microsoft Edge WebDriver matching the
 * installed WebView2 runtime, both on PATH.
 *
 * The exported `config` shape is typed narrowly against the WDIO Testrunner options actually
 * used here rather than against `@wdio/types`' `Options.Testrunner`, whose `capabilities` field
 * resolves through an ambient global (`WebdriverIO.Capabilities`) that this repo's tsconfig does
 * not merge in — avoids depending on that global-augmentation chain for a handful of fields.
 */
interface WdioTestrunnerConfig {
  runner: 'local';
  specs: string[];
  maxInstances: number;
  capabilities: Record<string, unknown>[];
  logLevel: 'trace' | 'debug' | 'info' | 'warn' | 'error' | 'silent';
  framework: string;
  reporters: string[];
  mochaOpts: { ui: string; timeout: number };
  hostname: string;
  port: number;
  path: string;
  baseUrl: string;
  onPrepare: () => void;
  beforeSession: () => void;
  afterSession: () => void;
}

let tauriDriver: ChildProcess | undefined;

const TAURI_BINARY_PATH = './src-tauri/target/debug/health-project.exe';

export const config: WdioTestrunnerConfig = {
  runner: 'local',
  specs: ['./tests/e2e/**/*.spec.ts'],
  maxInstances: 1,
  capabilities: [
    {
      // tauri-driver reads this vendor-prefixed capability to launch the compiled app.
      'tauri:options': {
        application: TAURI_BINARY_PATH,
      },
    },
  ],
  logLevel: 'warn',
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60_000,
  },
  hostname: '127.0.0.1',
  port: 4444,
  path: '/',

  // The built app serves its UI from the `tauri://localhost` custom protocol (not an HTTP dev
  // server), so `browser.url('/facility')`-style relative navigation resolves against this.
  baseUrl: 'tauri://localhost',

  // The app binary must exist before a session can start; build it (debug, matching the
  // Task-level DoD's `cargo build` gate) rather than requiring a manual step first.
  onPrepare: (): void => {
    spawnSync('cargo', ['build', '--manifest-path', 'src-tauri/Cargo.toml'], {
      stdio: 'inherit',
    });
  },

  // Start `tauri-driver` before each session and stop it after — it proxies this session's
  // WebDriver requests to the native WebView2 driver.
  beforeSession: (): void => {
    tauriDriver = spawn('tauri-driver', [], {
      stdio: [null, process.stdout, process.stderr],
    });
  },
  afterSession: (): void => {
    tauriDriver?.kill();
  },
};
