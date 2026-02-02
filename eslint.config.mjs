import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
    eslint.configs.recommended,
    ...tseslint.configs.recommended,
    {
        files: ['src/**/*.ts'],
        languageOptions: {
            parserOptions: {
                ecmaVersion: 2020,
                sourceType: 'module',
            },
        },
        rules: {
            '@typescript-eslint/no-unused-vars': 'error',
            '@typescript-eslint/no-explicit-any': 'warn',
            'no-console': 'warn',
            'prefer-const': 'error',
        },
    },
    {
        ignores: ['dist/**', 'node_modules/**', 'tests/**'],
    }
);
