import js from '@eslint/js';
import importPlugin from 'eslint-plugin-import';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import reactHooks from 'eslint-plugin-react-hooks';
import pluginReactRefresh from 'eslint-plugin-react-refresh';
import globals from 'globals';
import { config as defineFlatConfig, configs as tsConfigs } from 'typescript-eslint';

export default defineFlatConfig(
  {
    ignores: ['dist/', 'coverage/', 'node_modules/', 'src-tauri/', '.claude/', '.husky/'],
  },

  {
    // The TypeScript resolver also understands package `exports` maps, so it is the
    // resolver for every file — including the ESM config files at the repository root.
    settings: {
      'import/resolver': {
        typescript: { project: './tsconfig.json' },
      },
    },
  },

  js.configs.recommended,
  ...tsConfigs.recommendedTypeChecked,
  ...tsConfigs.stylisticTypeChecked,
  importPlugin.flatConfigs.recommended,
  importPlugin.flatConfigs.typescript,
  jsxA11y.flatConfigs.recommended,
  // v7 exposes both eslintrc and flat shapes; only `configs.flat.*` is valid here.
  reactHooks.configs.flat['recommended-latest'],

  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: globals.browser,
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: { 'react-refresh': pluginReactRefresh },
    rules: {
      // Rules.md 7.2 — `any` is forbidden; `unknown` + narrowing is the only escape hatch.
      '@typescript-eslint/no-explicit-any': 'error',
      // Rules.md 7.3 — every exported function has an explicit return type.
      '@typescript-eslint/explicit-module-boundary-types': 'error',
      // Rules.md 7.4 — no non-null assertion outside tests.
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@typescript-eslint/consistent-type-imports': ['error', { prefer: 'type-imports' }],
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],

      // Rules.md 4.1 / 4.2 / 4.3 — import hygiene.
      'import/no-cycle': 'error',
      'import/order': [
        'error',
        {
          groups: ['builtin', 'external', 'internal', 'parent', 'sibling', 'index', 'type'],
          pathGroups: [
            { pattern: '@shared/**', group: 'internal', position: 'before' },
            { pattern: '@modules/**', group: 'internal', position: 'after' },
            { pattern: '@app/**', group: 'internal', position: 'after' },
          ],
          'newlines-between': 'always',
          alphabetize: { order: 'asc', caseInsensitive: true },
        },
      ],
      // Rules.md 4.3 — use path aliases instead of deep relative imports.
      'no-restricted-imports': ['error', { patterns: ['../../../*'] }],

      // Rules.md 20.x — complexity limits.
      'max-lines-per-function': ['error', { max: 40, skipBlankLines: true, skipComments: true }],
      complexity: ['error', 10],
      'max-params': ['error', 4],
      'max-depth': ['error', 3],

      // CI runs with --max-warnings=0, so every enabled rule is an error in practice.
      'react-refresh/only-export-components': ['error', { allowConstantExport: true }],
    },
  },

  {
    // Rules.md 7.4 / 15.x — tests may assert non-null and exceed the function-length limit.
    files: ['**/*.test.{ts,tsx}', 'vitest.setup.ts'],
    rules: {
      '@typescript-eslint/no-non-null-assertion': 'off',
      'max-lines-per-function': 'off',
    },
  },

  {
    files: ['*.config.{js,ts}', 'vitest.setup.ts'],
    languageOptions: { globals: globals.node },
  },

  {
    files: ['**/*.js'],
    extends: [tsConfigs.disableTypeChecked],
  },
);
