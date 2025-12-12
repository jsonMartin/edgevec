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
  coverageThreshold: {
    global: {
      lines: 80,
      functions: 80,
      branches: 80,
      statements: 80,
    },
  },
  collectCoverageFrom: [
    '*.ts',
    '!**/__tests__/**',
    '!index.ts',
  ],
  // Mock WASM modules for Node.js testing
  modulePathIgnorePatterns: ['<rootDir>/../pkg/'],
};
