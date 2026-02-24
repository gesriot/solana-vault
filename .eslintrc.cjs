module.exports = {
  root: true,
  env: {
    es2021: true,
    node: true,
    mocha: true,
  },
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
    project: null,
  },
  plugins: ["@typescript-eslint"],
  extends: ["eslint:recommended", "plugin:@typescript-eslint/recommended"],
  ignorePatterns: ["target/", "node_modules/"],
  rules: {
    "@typescript-eslint/no-explicit-any": "off",
  },
};
