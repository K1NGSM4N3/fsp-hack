import { defineConfig } from "@playwright/test";
import { CONTEXT } from "./lib/env";

const workers = Number(process.env.BLACKBOX_WORKERS ?? 4);

export default defineConfig({
  testDir: ".",
  outputDir: "test-results",
  timeout: 180_000,
  expect: { timeout: 15_000 },
  fullyParallel: false,
  forbidOnly: true,
  retries: 0,
  workers,
  reporter: [
    ["list"],
    ["html", { open: "never", outputFolder: "playwright-report" }],
    ["json", { outputFile: "test-results/results.json" }],
  ],
  use: {
    ...CONTEXT,
    actionTimeout: 15_000,
    navigationTimeout: 30_000,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    launchOptions: {
      // A fake camera for the proctoring step: the browser grants it and feeds a test pattern.
      args: ["--use-fake-ui-for-media-stream", "--use-fake-device-for-media-stream"],
    },
  },
  projects: [
    { name: "setup", testMatch: /setup\/.*\.setup\.ts/ },
    { name: "blackbox", testMatch: /specs\/.*\.spec\.ts/, dependencies: ["setup"] },
  ],
});
