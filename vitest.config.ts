import { mergeConfig, defineConfig } from 'vitest/config';

import viteConfig from './vite.config.ts';

// Thresholds mirror Rules.md 15.1 / Testing.md Section 3. They are declared per glob
// so each layer is gated at its own documented level rather than a blended average.
const LOGIC_COVERAGE = { lines: 80, functions: 80, branches: 80, statements: 80 };
const COMPONENT_COVERAGE = { lines: 70, functions: 70, branches: 70, statements: 70 };

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      globals: true,
      environment: 'jsdom',
      setupFiles: ['./vitest.setup.ts'],
      include: ['src/**/*.test.{ts,tsx}'],
      coverage: {
        provider: 'v8',
        reporter: ['text', 'lcov'],
        include: ['src/**/*.{ts,tsx}'],
        exclude: ['src/**/*.test.{ts,tsx}', 'src/**/types/**', 'src/main.tsx', 'src/vite-env.d.ts'],
        thresholds: {
          'src/shared/lib/**/*.ts': LOGIC_COVERAGE,
          'src/shared/hooks/**/*.ts': LOGIC_COVERAGE,
          'src/modules/**/hooks/**/*.ts': LOGIC_COVERAGE,
          'src/modules/**/api/**/*.ts': LOGIC_COVERAGE,
          'src/**/*.tsx': COMPONENT_COVERAGE,
        },
      },
    },
  }),
);
