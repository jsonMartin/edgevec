// wasm/EdgeVecConfig.ts

import { EdgeVecClientConfig } from './EdgeVecClient';

/**
 * Fluent builder for EdgeVec configuration.
 *
 * @example
 * ```typescript
 * const config = new EdgeVecConfigBuilder(128)
 *   .withMetric('cosine')
 *   .withQuantization('sq8')
 *   .build();
 * ```
 */
export class EdgeVecConfigBuilder {
  private config: EdgeVecClientConfig;

  constructor(dimensions: number) {
    if (dimensions <= 0) {
      throw new Error('Dimensions must be positive');
    }
    this.config = { dimensions };
  }

  /**
   * Set the distance metric.
   *
   * @param metric - 'l2' (Euclidean), 'cosine', or 'dot' (inner product)
   */
  withMetric(metric: 'l2' | 'cosine' | 'dot'): this {
    this.config.metric = metric;
    return this;
  }

  /**
   * Enable scalar quantization for 4x memory reduction.
   *
   * @param quantization - 'none' or 'sq8' (8-bit scalar quantization)
   */
  withQuantization(quantization: 'none' | 'sq8'): this {
    this.config.quantization = quantization;
    return this;
  }

  /**
   * Build the configuration object.
   */
  build(): EdgeVecClientConfig {
    return { ...this.config };
  }
}
