export default {
  preset: 'ts-jest/presets/default-esm',
  testEnvironment: 'node',
  extensionsToTreatAsEsm: ['.ts'],
  moduleNameMapper: {
    '^(\\.{1,2}/.*)\\.js$': '$1',
  },
  transform: {
    '^.+\\.ts$': ['ts-jest', {
      useESM: true,
    }],
  },
  // Mock WASM modules for Node.js testing
  modulePathIgnorePatterns: ['<rootDir>/../pkg/'],
  // Unit tests (*.unit.test.ts) run in Node.js with mocked WASM
  // Integration tests (*.test.ts without .unit.) require browser/WASM environment
  // See wasm/examples/test-harness.html for browser integration tests
  testMatch: ['<rootDir>/__tests__/**/*.unit.test.ts'],
  testPathIgnorePatterns: ['<rootDir>/node_modules/', '<rootDir>/dist/'],
  // Coverage is tracked via browser integration tests (W17.3)
  collectCoverage: false,
};
