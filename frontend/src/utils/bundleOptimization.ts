// Bundle optimization and analysis utilities

export interface BundleAnalysis {
  totalSize: number;
  gzippedSize: number;
  chunks: ChunkInfo[];
  dependencies: DependencyInfo[];
  duplicates: DuplicateInfo[];
  recommendations: OptimizationRecommendation[];
  loadingPerformance: LoadingPerformance;
}

export interface ChunkInfo {
  name: string;
  size: number;
  gzippedSize: number;
  modules: ModuleInfo[];
  loadTime: number;
  isAsync: boolean;
  priority: 'high' | 'medium' | 'low';
}

export interface ModuleInfo {
  name: string;
  size: number;
  reasons: string[];
  isEntry: boolean;
  isVendor: boolean;
}

export interface DependencyInfo {
  name: string;
  version: string;
  size: number;
  usageCount: number;
  isTreeShakeable: boolean;
  alternatives: string[];
}

export interface DuplicateInfo {
  module: string;
  instances: number;
  totalSize: number;
  chunks: string[];
}

export interface OptimizationRecommendation {
  type: 'code-splitting' | 'tree-shaking' | 'compression' | 'lazy-loading' | 'dependency';
  priority: 'high' | 'medium' | 'low';
  description: string;
  potentialSavings: number;
  implementation: string;
}

export interface LoadingPerformance {
  firstChunkTime: number;
  totalLoadTime: number;
  parallelLoadingEfficiency: number;
  criticalPathLength: number;
  renderBlockingResources: string[];
}

class BundleAnalyzer {
  private performanceObserver?: PerformanceObserver;
  private chunkLoadTimes = new Map<string, number>();
  private moduleRegistry = new Map<string, ModuleInfo>();

  constructor() {
    this.initializePerformanceMonitoring();
    this.analyzeCurrentBundle();
  }

  private initializePerformanceMonitoring(): void {
    if (!window.PerformanceObserver) return;

    this.performanceObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      
      entries.forEach((entry) => {
        if (entry.entryType === 'resource' && this.isJavaScriptResource(entry.name)) {
          this.recordChunkLoadTime(entry.name, entry.duration);
        }
      });
    });

    this.performanceObserver.observe({ entryTypes: ['resource'] });
  }

  private isJavaScriptResource(url: string): boolean {
    return url.includes('.js') || url.includes('.mjs') || url.includes('.ts');
  }

  private recordChunkLoadTime(url: string, duration: number): void {
    const chunkName = this.extractChunkName(url);
    this.chunkLoadTimes.set(chunkName, duration);
  }

  private extractChunkName(url: string): string {
    const match = url.match(/\/([^\/]+)\.(js|mjs|ts)$/);
    return match ? match[1] : url;
  }

  private analyzeCurrentBundle(): void {
    // Analyze webpack chunks if available
    if (typeof __webpack_require__ !== 'undefined') {
      this.analyzeWebpackChunks();
    }

    // Analyze script tags
    this.analyzeScriptTags();
  }

  private analyzeWebpackChunks(): void {
    try {
      // Access webpack chunk information
      const webpackChunks = (window as any).__webpack_require__.cache || {};
      
      Object.keys(webpackChunks).forEach(moduleId => {
        const module = webpackChunks[moduleId];
        if (module && module.exports) {
          this.registerModule(moduleId, {
            name: moduleId,
            size: this.estimateModuleSize(module),
            reasons: [],
            isEntry: false,
            isVendor: this.isVendorModule(moduleId),
          });
        }
      });
    } catch (error) {
      console.warn('Failed to analyze webpack chunks:', error);
    }
  }

  private analyzeScriptTags(): void {
    const scripts = Array.from(document.querySelectorAll('script[src]'));
    
    scripts.forEach(script => {
      const src = (script as HTMLScriptElement).src;
      const size = this.estimateScriptSize(script as HTMLScriptElement);
      
      this.registerModule(src, {
        name: this.extractChunkName(src),
        size,
        reasons: ['script-tag'],
        isEntry: script.hasAttribute('data-entry'),
        isVendor: this.isVendorModule(src),
      });
    });
  }

  private registerModule(id: string, info: ModuleInfo): void {
    this.moduleRegistry.set(id, info);
  }

  private estimateModuleSize(module: any): number {
    try {
      return JSON.stringify(module).length;
    } catch {
      return 0;
    }
  }

  private estimateScriptSize(script: HTMLScriptElement): number {
    // Try to get size from performance API
    const entries = performance.getEntriesByName(script.src, 'resource');
    if (entries.length > 0) {
      const entry = entries[0] as PerformanceResourceTiming;
      return entry.transferSize || entry.encodedBodySize || 0;
    }
    return 0;
  }

  private isVendorModule(moduleId: string): boolean {
    return moduleId.includes('node_modules') || 
           moduleId.includes('vendor') ||
           moduleId.includes('chunk-vendors');
  }

  public analyzeBundlePerformance(): BundleAnalysis {
    const chunks = this.analyzeChunks();
    const dependencies = this.analyzeDependencies();
    const duplicates = this.findDuplicates();
    const loadingPerformance = this.analyzeLoadingPerformance();
    
    const totalSize = chunks.reduce((sum, chunk) => sum + chunk.size, 0);
    const gzippedSize = Math.round(totalSize * 0.3); // Estimate

    const recommendations = this.generateRecommendations(chunks, dependencies, duplicates);

    return {
      totalSize,
      gzippedSize,
      chunks,
      dependencies,
      duplicates,
      recommendations,
      loadingPerformance,
    };
  }

  private analyzeChunks(): ChunkInfo[] {
    const chunks: ChunkInfo[] = [];
    const chunkGroups = new Map<string, ModuleInfo[]>();

    // Group modules by chunk
    this.moduleRegistry.forEach((module, id) => {
      const chunkName = this.determineChunkName(id, module);
      
      if (!chunkGroups.has(chunkName)) {
        chunkGroups.set(chunkName, []);
      }
      chunkGroups.get(chunkName)!.push(module);
    });

    // Create chunk info
    chunkGroups.forEach((modules, chunkName) => {
      const size = modules.reduce((sum, module) => sum + module.size, 0);
      const loadTime = this.chunkLoadTimes.get(chunkName) || 0;
      
      chunks.push({
        name: chunkName,
        size,
        gzippedSize: Math.round(size * 0.3),
        modules,
        loadTime,
        isAsync: this.isAsyncChunk(chunkName),
        priority: this.determineChunkPriority(chunkName, modules),
      });
    });

    return chunks.sort((a, b) => b.size - a.size);
  }

  private determineChunkName(moduleId: string, module: ModuleInfo): string {
    if (module.isVendor) return 'vendor';
    if (module.isEntry) return 'main';
    
    // Try to extract chunk name from module path
    const pathParts = moduleId.split('/');
    if (pathParts.length > 1) {
      return pathParts[pathParts.length - 2] || 'unknown';
    }
    
    return 'unknown';
  }

  private isAsyncChunk(chunkName: string): boolean {
    return !['main', 'vendor', 'runtime'].includes(chunkName);
  }

  private determineChunkPriority(chunkName: string, modules: ModuleInfo[]): 'high' | 'medium' | 'low' {
    if (['main', 'vendor', 'runtime'].includes(chunkName)) return 'high';
    
    const hasEntryModule = modules.some(m => m.isEntry);
    if (hasEntryModule) return 'high';
    
    const totalSize = modules.reduce((sum, m) => sum + m.size, 0);
    if (totalSize > 100000) return 'medium'; // 100KB
    
    return 'low';
  }

  private analyzeDependencies(): DependencyInfo[] {
    const dependencies: DependencyInfo[] = [];
    
    // Analyze package.json dependencies if available
    try {
      const packageInfo = this.getPackageInfo();
      if (packageInfo) {
        Object.entries(packageInfo.dependencies || {}).forEach(([name, version]) => {
          const usage = this.analyzeDependencyUsage(name);
          dependencies.push({
            name,
            version: version as string,
            size: usage.size,
            usageCount: usage.count,
            isTreeShakeable: this.isTreeShakeable(name),
            alternatives: this.suggestAlternatives(name),
          });
        });
      }
    } catch (error) {
      console.warn('Failed to analyze dependencies:', error);
    }

    return dependencies.sort((a, b) => b.size - a.size);
  }

  private getPackageInfo(): any {
    // In a real implementation, this would be injected during build
    return null;
  }

  private analyzeDependencyUsage(packageName: string): { size: number; count: number } {
    let size = 0;
    let count = 0;

    this.moduleRegistry.forEach((module, id) => {
      if (id.includes(packageName)) {
        size += module.size;
        count++;
      }
    });

    return { size, count };
  }

  private isTreeShakeable(packageName: string): boolean {
    // Common tree-shakeable packages
    const treeShakeablePackages = [
      'lodash-es',
      'ramda',
      'date-fns',
      'rxjs',
      '@material-ui/icons',
    ];
    
    return treeShakeablePackages.some(pkg => packageName.includes(pkg));
  }

  private suggestAlternatives(packageName: string): string[] {
    const alternatives: Record<string, string[]> = {
      'moment': ['date-fns', 'dayjs'],
      'lodash': ['lodash-es', 'ramda'],
      'axios': ['fetch', 'ky'],
      'jquery': ['vanilla-js', 'cash-dom'],
      'bootstrap': ['tailwindcss', 'bulma'],
    };

    return alternatives[packageName] || [];
  }

  private findDuplicates(): DuplicateInfo[] {
    const duplicates: DuplicateInfo[] = [];
    const moduleNames = new Map<string, string[]>();

    // Group modules by name
    this.moduleRegistry.forEach((module, id) => {
      const name = module.name;
      if (!moduleNames.has(name)) {
        moduleNames.set(name, []);
      }
      moduleNames.get(name)!.push(id);
    });

    // Find duplicates
    moduleNames.forEach((instances, name) => {
      if (instances.length > 1) {
        const totalSize = instances.reduce((sum, id) => {
          const module = this.moduleRegistry.get(id);
          return sum + (module?.size || 0);
        }, 0);

        duplicates.push({
          module: name,
          instances: instances.length,
          totalSize,
          chunks: instances.map(id => this.determineChunkName(id, this.moduleRegistry.get(id)!)),
        });
      }
    });

    return duplicates.sort((a, b) => b.totalSize - a.totalSize);
  }

  private analyzeLoadingPerformance(): LoadingPerformance {
    const navigationEntry = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
    const resourceEntries = performance.getEntriesByType('resource') as PerformanceResourceTiming[];
    
    const jsResources = resourceEntries.filter(entry => this.isJavaScriptResource(entry.name));
    
    const firstChunkTime = jsResources.length > 0 ? Math.min(...jsResources.map(r => r.duration)) : 0;
    const totalLoadTime = jsResources.length > 0 ? Math.max(...jsResources.map(r => r.responseEnd)) : 0;
    
    // Calculate parallel loading efficiency
    const totalSequentialTime = jsResources.reduce((sum, r) => sum + r.duration, 0);
    const parallelLoadingEfficiency = totalLoadTime > 0 ? (totalSequentialTime / totalLoadTime) : 1;
    
    // Find render-blocking resources
    const renderBlockingResources = jsResources
      .filter(r => r.responseEnd < (navigationEntry?.domContentLoadedEventStart || 0))
      .map(r => r.name);

    return {
      firstChunkTime,
      totalLoadTime,
      parallelLoadingEfficiency,
      criticalPathLength: renderBlockingResources.length,
      renderBlockingResources,
    };
  }

  private generateRecommendations(
    chunks: ChunkInfo[],
    dependencies: DependencyInfo[],
    duplicates: DuplicateInfo[]
  ): OptimizationRecommendation[] {
    const recommendations: OptimizationRecommendation[] = [];

    // Large chunk recommendations
    chunks.forEach(chunk => {
      if (chunk.size > 500000 && !chunk.isAsync) { // 500KB
        recommendations.push({
          type: 'code-splitting',
          priority: 'high',
          description: `Split large chunk "${chunk.name}" (${(chunk.size / 1024).toFixed(1)}KB) into smaller chunks`,
          potentialSavings: chunk.size * 0.3,
          implementation: `Use dynamic imports or React.lazy() to split ${chunk.name}`,
        });
      }
    });

    // Large dependency recommendations
    dependencies.forEach(dep => {
      if (dep.size > 100000 && dep.usageCount < 5) { // 100KB, low usage
        recommendations.push({
          type: 'dependency',
          priority: 'medium',
          description: `Consider replacing large dependency "${dep.name}" (${(dep.size / 1024).toFixed(1)}KB)`,
          potentialSavings: dep.size,
          implementation: dep.alternatives.length > 0 
            ? `Replace with: ${dep.alternatives.join(', ')}`
            : 'Consider removing or finding lighter alternative',
        });
      }

      if (!dep.isTreeShakeable && dep.size > 50000) {
        recommendations.push({
          type: 'tree-shaking',
          priority: 'medium',
          description: `Enable tree-shaking for "${dep.name}"`,
          potentialSavings: dep.size * 0.5,
          implementation: 'Use ES6 imports and ensure the package supports tree-shaking',
        });
      }
    });

    // Duplicate recommendations
    duplicates.forEach(duplicate => {
      if (duplicate.totalSize > 50000) { // 50KB
        recommendations.push({
          type: 'code-splitting',
          priority: 'high',
          description: `Remove duplicate module "${duplicate.module}" (${duplicate.instances} instances)`,
          potentialSavings: duplicate.totalSize * 0.8,
          implementation: 'Use webpack optimization.splitChunks to deduplicate modules',
        });
      }
    });

    // Compression recommendations
    const totalSize = chunks.reduce((sum, chunk) => sum + chunk.size, 0);
    if (totalSize > 1000000) { // 1MB
      recommendations.push({
        type: 'compression',
        priority: 'medium',
        description: 'Enable Brotli compression for better compression ratios',
        potentialSavings: totalSize * 0.2,
        implementation: 'Configure server to serve Brotli-compressed assets',
      });
    }

    return recommendations.sort((a, b) => {
      const priorityOrder = { high: 3, medium: 2, low: 1 };
      return priorityOrder[b.priority] - priorityOrder[a.priority];
    });
  }

  public generateOptimizationReport(): string {
    const analysis = this.analyzeBundlePerformance();
    
    let report = '# Bundle Optimization Report\n\n';
    
    report += `## Summary\n`;
    report += `- Total Bundle Size: ${(analysis.totalSize / 1024).toFixed(1)}KB\n`;
    report += `- Gzipped Size: ${(analysis.gzippedSize / 1024).toFixed(1)}KB\n`;
    report += `- Number of Chunks: ${analysis.chunks.length}\n`;
    report += `- Dependencies: ${analysis.dependencies.length}\n`;
    report += `- Duplicates Found: ${analysis.duplicates.length}\n\n`;
    
    report += `## Performance Metrics\n`;
    report += `- First Chunk Load Time: ${analysis.loadingPerformance.firstChunkTime.toFixed(1)}ms\n`;
    report += `- Total Load Time: ${analysis.loadingPerformance.totalLoadTime.toFixed(1)}ms\n`;
    report += `- Parallel Loading Efficiency: ${(analysis.loadingPerformance.parallelLoadingEfficiency * 100).toFixed(1)}%\n`;
    report += `- Render Blocking Resources: ${analysis.loadingPerformance.renderBlockingResources.length}\n\n`;
    
    if (analysis.recommendations.length > 0) {
      report += `## Optimization Recommendations\n\n`;
      analysis.recommendations.forEach((rec, index) => {
        report += `### ${index + 1}. ${rec.description}\n`;
        report += `- **Type**: ${rec.type}\n`;
        report += `- **Priority**: ${rec.priority}\n`;
        report += `- **Potential Savings**: ${(rec.potentialSavings / 1024).toFixed(1)}KB\n`;
        report += `- **Implementation**: ${rec.implementation}\n\n`;
      });
    }
    
    return report;
  }

  public destroy(): void {
    if (this.performanceObserver) {
      this.performanceObserver.disconnect();
    }
  }
}

// Webpack bundle analyzer integration
export const analyzeWebpackBundle = (): BundleAnalysis | null => {
  try {
    const analyzer = new BundleAnalyzer();
    return analyzer.analyzeBundlePerformance();
  } catch (error) {
    console.error('Failed to analyze webpack bundle:', error);
    return null;
  }
};

// Runtime bundle monitoring
export const monitorBundlePerformance = (callback: (analysis: BundleAnalysis) => void): () => void => {
  const analyzer = new BundleAnalyzer();
  
  const interval = setInterval(() => {
    const analysis = analyzer.analyzeBundlePerformance();
    callback(analysis);
  }, 30000); // Every 30 seconds
  
  return () => {
    clearInterval(interval);
    analyzer.destroy();
  };
};

// Development helper for bundle analysis
export const logBundleAnalysis = (): void => {
  if (process.env.NODE_ENV === 'development') {
    const analyzer = new BundleAnalyzer();
    const analysis = analyzer.analyzeBundlePerformance();
    
    console.group('📦 Bundle Analysis');
    console.log('Total Size:', (analysis.totalSize / 1024).toFixed(1) + 'KB');
    console.log('Gzipped Size:', (analysis.gzippedSize / 1024).toFixed(1) + 'KB');
    console.log('Chunks:', analysis.chunks.length);
    console.log('Dependencies:', analysis.dependencies.length);
    
    if (analysis.recommendations.length > 0) {
      console.group('🔧 Recommendations');
      analysis.recommendations.forEach(rec => {
        console.log(`${rec.priority.toUpperCase()}: ${rec.description}`);
      });
      console.groupEnd();
    }
    
    console.groupEnd();
    
    // Log full report
    console.log(analyzer.generateOptimizationReport());
  }
};

export default BundleAnalyzer;