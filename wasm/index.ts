// wasm/index.ts

export { EdgeVecClient, SearchResult, EdgeVecClientConfig } from './EdgeVecClient';
export { EdgeVecConfigBuilder } from './EdgeVecConfig';
export {
  VectorId,
  DistanceMetric,
  QuantizationMode,
  EdgeVecStats,
  CompactionResult,
  SoftDeleteStats
} from './types';

// Re-export raw WASM bindings for advanced users
export * from '../pkg/edgevec.js';
