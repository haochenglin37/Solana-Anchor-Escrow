module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  extends: ['eslint:recommended', 'plugin:react-hooks/recommended', 'prettier'],
  plugins: ['@typescript-eslint'],
  env: { browser: true, node: true, es2022: true }
};
