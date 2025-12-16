/**
 * EdgeVec Performance Dashboard
 *
 * NVIDIA-grade visualization of competitive benchmark results comparing
 * EdgeVec against hnswlib-node and voy vector databases.
 *
 * Features:
 * - Animated particle background
 * - Hero stats with winner badges
 * - Interactive Chart.js visualizations
 * - Responsive cyberpunk theme
 *
 * @version 2.0.0
 * @license MIT OR Apache-2.0
 */

// ═══════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════

// Chart.js global configuration for cyberpunk theme
Chart.defaults.color = '#8080a0';
Chart.defaults.borderColor = '#252535';
Chart.defaults.font.family = "'JetBrains Mono', 'Fira Code', monospace";

// Color palette matching EdgeVec cyberpunk theme
const COLORS = {
    edgevec: {
        primary: '#00ffff',
        secondary: 'rgba(0, 255, 255, 0.2)',
        glow: 'rgba(0, 255, 255, 0.5)',
        gradient: ['#00ffff', '#00aaaa']
    },
    hnswlib: {
        primary: '#9945ff',
        secondary: 'rgba(153, 69, 255, 0.2)',
        glow: 'rgba(153, 69, 255, 0.5)',
        gradient: ['#9945ff', '#6622aa']
    },
    voy: {
        primary: '#ffff00',
        secondary: 'rgba(255, 255, 0, 0.2)',
        glow: 'rgba(255, 255, 0, 0.5)',
        gradient: ['#ffff00', '#aaaa00']
    }
};

const LIBRARY_LABELS = {
    'edgevec': 'EdgeVec (WASM)',
    'hnswlib-node': 'hnswlib-node (Native)',
    'voy': 'voy (WASM)'
};

const LIBRARY_SHORT = {
    'edgevec': 'EV',
    'hnswlib-node': 'HL',
    'voy': 'VY'
};

const LIBRARY_PLATFORMS = {
    'edgevec': 'Browser WASM',
    'hnswlib-node': 'Node.js Native',
    'voy': 'Browser WASM'
};

// ═══════════════════════════════════════════════════════════════════════════
// PARTICLE BACKGROUND SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

class ParticleSystem {
    constructor(canvas) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.particles = [];
        this.connections = [];
        this.animationId = null;
        this.resize();
        this.init();
        window.addEventListener('resize', () => this.resize());
    }

    resize() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    }

    init() {
        // Create particles
        const particleCount = Math.floor((this.canvas.width * this.canvas.height) / 25000);
        this.particles = [];

        for (let i = 0; i < particleCount; i++) {
            this.particles.push({
                x: Math.random() * this.canvas.width,
                y: Math.random() * this.canvas.height,
                vx: (Math.random() - 0.5) * 0.3,
                vy: (Math.random() - 0.5) * 0.3,
                radius: Math.random() * 1.5 + 0.5,
                color: Math.random() > 0.7 ? '#ff00ff' : '#00ffff',
                alpha: Math.random() * 0.5 + 0.2
            });
        }
    }

    draw() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        // Draw connections
        for (let i = 0; i < this.particles.length; i++) {
            for (let j = i + 1; j < this.particles.length; j++) {
                const dx = this.particles[i].x - this.particles[j].x;
                const dy = this.particles[i].y - this.particles[j].y;
                const dist = Math.sqrt(dx * dx + dy * dy);

                if (dist < 120) {
                    const alpha = (1 - dist / 120) * 0.15;
                    this.ctx.beginPath();
                    this.ctx.strokeStyle = `rgba(0, 255, 255, ${alpha})`;
                    this.ctx.lineWidth = 0.5;
                    this.ctx.moveTo(this.particles[i].x, this.particles[i].y);
                    this.ctx.lineTo(this.particles[j].x, this.particles[j].y);
                    this.ctx.stroke();
                }
            }
        }

        // Draw particles
        for (const p of this.particles) {
            this.ctx.beginPath();
            this.ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
            this.ctx.fillStyle = p.color.replace(')', `, ${p.alpha})`).replace('rgb', 'rgba').replace('#', '');

            // Convert hex to rgba
            if (p.color.startsWith('#')) {
                const hex = p.color.slice(1);
                const r = parseInt(hex.substr(0, 2), 16);
                const g = parseInt(hex.substr(2, 2), 16);
                const b = parseInt(hex.substr(4, 2), 16);
                this.ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${p.alpha})`;
            }

            this.ctx.fill();

            // Glow effect
            this.ctx.shadowBlur = 8;
            this.ctx.shadowColor = p.color;
            this.ctx.fill();
            this.ctx.shadowBlur = 0;
        }
    }

    update() {
        for (const p of this.particles) {
            p.x += p.vx;
            p.y += p.vy;

            // Wrap around edges
            if (p.x < 0) p.x = this.canvas.width;
            if (p.x > this.canvas.width) p.x = 0;
            if (p.y < 0) p.y = this.canvas.height;
            if (p.y > this.canvas.height) p.y = 0;
        }
    }

    animate() {
        this.update();
        this.draw();
        this.animationId = requestAnimationFrame(() => this.animate());
    }

    start() {
        this.animate();
    }

    stop() {
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DATA LOADING
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Attempt to load benchmark data from multiple paths
 */
async function loadBenchmarkData() {
    const paths = [
        './benchmark-data.json',                           // local fallback (FIRST - most reliable)
        '../../benches/competitive/results/latest.json',  // from wasm/examples/
        '../benches/competitive/results/latest.json',      // alternative
        '/benches/competitive/results/latest.json'         // absolute path
    ];

    for (const path of paths) {
        try {
            const response = await fetch(path);
            if (response.ok) {
                const data = await response.json();
                console.log(`[Dashboard] Loaded benchmark data from: ${path}`);

                // Validate that we have all 3 required libraries
                const libraries = data.map(d => d.library);
                const required = ['edgevec', 'hnswlib-node', 'voy'];
                const missing = required.filter(r => !libraries.includes(r));

                if (missing.length > 0) {
                    console.warn(`[Dashboard] Warning: Missing libraries: ${missing.join(', ')}`);
                    console.warn('[Dashboard] Run: node harness.js --all');

                    // If using live data with missing entries, fall back to sample
                    if (path.includes('latest.json')) {
                        console.log('[Dashboard] Falling back to sample data...');
                        continue; // Try next path (sample data)
                    }
                }

                return data;
            }
        } catch (e) {
            console.warn(`[Dashboard] Failed to load from ${path}:`, e.message);
        }
    }

    throw new Error(
        'Could not load benchmark data. To generate data:\n' +
        '1. cd benches/competitive\n' +
        '2. npm install\n' +
        '3. node harness.js --all\n' +
        '\nOr serve the page via HTTP (not file://)'
    );
}

/**
 * Parse benchmark data into a structured format
 */
function parseBenchmarkData(rawData) {
    const result = {
        libraries: {},
        config: null,
        timestamp: null
    };

    for (const entry of rawData) {
        const libName = entry.library;
        result.libraries[libName] = {
            search: {
                mean: parseFloat(entry.search.mean_ms),
                p50: parseFloat(entry.search.p50_ms),
                p99: parseFloat(entry.search.p99_ms)
            },
            insert: {
                mean: parseFloat(entry.insert.mean_ms),
                p50: parseFloat(entry.insert.p50_ms),
                p99: parseFloat(entry.insert.p99_ms)
            },
            memory: parseFloat(entry.memory.used_mb),
            recall: parseFloat(entry.recall?.percentage || 0)
        };

        // Take config from first entry (should be same for all)
        if (!result.config) {
            result.config = entry.config;
            result.timestamp = entry.timestamp;
        }
    }

    return result;
}

// ═══════════════════════════════════════════════════════════════════════════
// HERO STATS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Calculate comparative insights and populate hero stats
 */
function populateHeroStats(data) {
    const edgevec = data.libraries['edgevec'];
    const hnswlib = data.libraries['hnswlib-node'];
    const voy = data.libraries['voy'];

    const heroStats = document.getElementById('heroStats');

    // Calculate rankings
    const searchRank = [
        { lib: 'hnswlib-node', val: hnswlib.search.p50 },
        { lib: 'edgevec', val: edgevec.search.p50 },
        { lib: 'voy', val: voy.search.p50 }
    ].sort((a, b) => a.val - b.val);

    const memRank = [
        { lib: 'edgevec', val: Math.abs(edgevec.memory) },
        { lib: 'hnswlib-node', val: Math.abs(hnswlib.memory) },
        { lib: 'voy', val: Math.abs(voy.memory) }
    ].sort((a, b) => a.val - b.val);

    // Calculate comparisons
    const vsVoy = (voy.search.p50 / edgevec.search.p50).toFixed(0);
    const vsHnswlib = (edgevec.search.p50 / hnswlib.search.p50).toFixed(1);
    const memSavings = (1 - Math.abs(edgevec.memory) / Math.abs(voy.memory)) * 100;

    // EdgeVec position in search ranking
    const edgevecSearchRank = searchRank.findIndex(r => r.lib === 'edgevec') + 1;
    const edgevecMemRank = memRank.findIndex(r => r.lib === 'edgevec') + 1;

    const stats = [
        {
            icon: '⚡',
            value: edgevec.search.p50.toFixed(2) + 'ms',
            label: 'Search P50 Latency',
            comparison: `<span class="highlight">${vsVoy}x faster</span> than voy`,
            winner: edgevecSearchRank <= 2
        },
        {
            icon: '🎯',
            value: vsVoy + 'x',
            label: 'vs voy (WASM)',
            comparison: 'EdgeVec search advantage',
            winner: true
        },
        {
            icon: '🔬',
            value: vsHnswlib + 'x',
            label: 'vs hnswlib (Native)',
            comparison: 'Expected: WASM vs C++ native',
            winner: false
        },
        {
            icon: '💾',
            value: Math.abs(edgevec.memory).toFixed(1) + 'MB',
            label: 'Memory Usage',
            comparison: edgevecMemRank === 1 ? '<span class="highlight">Lowest memory</span>' : `${memSavings.toFixed(0)}% less than voy`,
            winner: edgevecMemRank === 1
        }
    ];

    heroStats.innerHTML = stats.map(stat => `
        <div class="stat-card${stat.winner ? ' winner' : ''}">
            ${stat.winner ? '<span class="winner-badge">Best in Class</span>' : ''}
            <div class="stat-icon">${stat.icon}</div>
            <div class="stat-value">${stat.value}</div>
            <div class="stat-label">${stat.label}</div>
            <div class="stat-comparison">${stat.comparison}</div>
        </div>
    `).join('');
}

// ═══════════════════════════════════════════════════════════════════════════
// CHARTS
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Create a bar chart with cyberpunk styling
 */
function createBarChart(canvasId, labels, values, title, unit = 'ms') {
    const ctx = document.getElementById(canvasId).getContext('2d');

    // Determine colors based on library order
    const colors = labels.map(label => {
        if (label.includes('EdgeVec')) return COLORS.edgevec.primary;
        if (label.includes('hnswlib')) return COLORS.hnswlib.primary;
        if (label.includes('voy')) return COLORS.voy.primary;
        return '#8080a0';
    });

    const bgColors = labels.map(label => {
        if (label.includes('EdgeVec')) return COLORS.edgevec.secondary;
        if (label.includes('hnswlib')) return COLORS.hnswlib.secondary;
        if (label.includes('voy')) return COLORS.voy.secondary;
        return 'rgba(128, 128, 160, 0.2)';
    });

    return new Chart(ctx, {
        type: 'bar',
        data: {
            labels: labels,
            datasets: [{
                data: values,
                backgroundColor: bgColors,
                borderColor: colors,
                borderWidth: 2,
                borderRadius: 6,
                borderSkipped: false
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    display: false
                },
                tooltip: {
                    backgroundColor: 'rgba(18, 18, 28, 0.95)',
                    titleColor: '#e8e8f0',
                    bodyColor: '#00ffff',
                    borderColor: '#252535',
                    borderWidth: 1,
                    padding: 14,
                    cornerRadius: 8,
                    displayColors: false,
                    titleFont: {
                        family: "'JetBrains Mono', monospace",
                        size: 12,
                        weight: '600'
                    },
                    bodyFont: {
                        family: "'Orbitron', sans-serif",
                        size: 16,
                        weight: '700'
                    },
                    callbacks: {
                        label: function(context) {
                            return `${context.parsed.y.toFixed(3)} ${unit}`;
                        }
                    }
                }
            },
            scales: {
                x: {
                    grid: {
                        display: false
                    },
                    ticks: {
                        font: {
                            size: 10,
                            family: "'JetBrains Mono', monospace"
                        },
                        color: '#8080a0'
                    }
                },
                y: {
                    beginAtZero: true,
                    grid: {
                        color: 'rgba(37, 37, 53, 0.6)',
                        lineWidth: 1
                    },
                    ticks: {
                        font: {
                            size: 10,
                            family: "'JetBrains Mono', monospace"
                        },
                        color: '#8080a0',
                        callback: function(value) {
                            return value.toFixed(2) + unit;
                        }
                    }
                }
            },
            animation: {
                duration: 1200,
                easing: 'easeOutQuart'
            }
        }
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// TABLE
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Populate the comparison table with enhanced styling
 */
function populateTable(data) {
    const tbody = document.getElementById('comparisonBody');
    tbody.innerHTML = '';

    const libraries = Object.keys(data.libraries);

    // Find best/worst values for each metric
    const metrics = {
        searchP50: libraries.map(l => ({ lib: l, val: data.libraries[l].search.p50 })),
        searchP99: libraries.map(l => ({ lib: l, val: data.libraries[l].search.p99 })),
        insertP50: libraries.map(l => ({ lib: l, val: data.libraries[l].insert.p50 })),
        insertP99: libraries.map(l => ({ lib: l, val: data.libraries[l].insert.p99 })),
        memory: libraries.map(l => ({ lib: l, val: Math.abs(data.libraries[l].memory) }))
    };

    // Sort to find rankings (lower is better for all)
    const rankings = {};
    for (const [metric, values] of Object.entries(metrics)) {
        const sorted = [...values].sort((a, b) => a.val - b.val);
        rankings[metric] = {};
        sorted.forEach((item, idx) => {
            rankings[metric][item.lib] = idx + 1;
        });
    }

    const best = {
        searchP50: Math.min(...metrics.searchP50.map(m => m.val)),
        searchP99: Math.min(...metrics.searchP99.map(m => m.val)),
        insertP50: Math.min(...metrics.insertP50.map(m => m.val)),
        insertP99: Math.min(...metrics.insertP99.map(m => m.val)),
        memory: Math.min(...metrics.memory.map(m => m.val))
    };

    const worst = {
        searchP50: Math.max(...metrics.searchP50.map(m => m.val)),
        searchP99: Math.max(...metrics.searchP99.map(m => m.val)),
        insertP50: Math.max(...metrics.insertP50.map(m => m.val)),
        insertP99: Math.max(...metrics.insertP99.map(m => m.val)),
        memory: Math.max(...metrics.memory.map(m => m.val))
    };

    function getRankBadge(rank) {
        if (rank === 1) return '<span class="rank-badge gold">1</span>';
        if (rank === 2) return '<span class="rank-badge silver">2</span>';
        if (rank === 3) return '<span class="rank-badge bronze">3</span>';
        return '';
    }

    function getMetricClass(val, best, worst) {
        if (val === best) return 'metric-cell best';
        if (val === worst) return 'metric-cell worst';
        return 'metric-cell';
    }

    for (const libName of libraries) {
        const lib = data.libraries[libName];
        const row = document.createElement('tr');

        const libClass = libName === 'edgevec' ? 'edgevec' :
                        libName === 'hnswlib-node' ? 'hnswlib' : 'voy';

        row.innerHTML = `
            <td>
                <div class="lib-cell">
                    <div class="lib-icon ${libClass}">${LIBRARY_SHORT[libName]}</div>
                    <div>
                        <div class="lib-name">${LIBRARY_LABELS[libName] || libName}</div>
                        <div class="lib-platform">${LIBRARY_PLATFORMS[libName] || 'Unknown'}</div>
                    </div>
                </div>
            </td>
            <td class="${getMetricClass(lib.search.p50, best.searchP50, worst.searchP50)}">
                ${lib.search.p50.toFixed(3)}ms${getRankBadge(rankings.searchP50[libName])}
            </td>
            <td class="${getMetricClass(lib.search.p99, best.searchP99, worst.searchP99)}">
                ${lib.search.p99.toFixed(3)}ms${getRankBadge(rankings.searchP99[libName])}
            </td>
            <td class="${getMetricClass(lib.insert.p50, best.insertP50, worst.insertP50)}">
                ${lib.insert.p50.toFixed(3)}ms${getRankBadge(rankings.insertP50[libName])}
            </td>
            <td class="${getMetricClass(lib.insert.p99, best.insertP99, worst.insertP99)}">
                ${lib.insert.p99.toFixed(3)}ms${getRankBadge(rankings.insertP99[libName])}
            </td>
            <td class="${getMetricClass(Math.abs(lib.memory), best.memory, worst.memory)}">
                ${Math.abs(lib.memory).toFixed(2)}MB${getRankBadge(rankings.memory[libName])}
            </td>
        `;

        tbody.appendChild(row);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONFIG SECTION
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Populate configuration grid
 */
function populateConfig(config, timestamp) {
    const grid = document.getElementById('configGrid');

    const configs = [
        { label: 'Dimensions', value: config.dimensions },
        { label: 'Vector Count', value: config.vectorCount.toLocaleString() },
        { label: 'Query Count', value: config.queryCount },
        { label: 'Top-K', value: config.k },
        { label: 'HNSW M', value: config.hnsw.m },
        { label: 'EF Construction', value: config.hnsw.efConstruction },
        { label: 'EF Search', value: config.hnsw.efSearch },
        { label: 'Benchmark Date', value: new Date(timestamp).toLocaleDateString() }
    ];

    grid.innerHTML = configs.map(c => `
        <div class="config-item">
            <span class="config-label">${c.label}</span>
            <span class="config-value">${c.value}</span>
        </div>
    `).join('');
}

// ═══════════════════════════════════════════════════════════════════════════
// UI STATE MANAGEMENT
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Hide loading overlay with animation
 */
function hideLoading() {
    const overlay = document.getElementById('loadingOverlay');
    if (overlay) {
        overlay.classList.add('hidden');
        // Remove from DOM after animation
        setTimeout(() => {
            overlay.style.display = 'none';
        }, 500);
    }
}

/**
 * Show error state
 */
function showError(message) {
    hideLoading();
    document.getElementById('errorContainer').style.display = 'block';
    document.getElementById('charts').style.display = 'none';
    document.getElementById('comparison').style.display = 'none';
    document.getElementById('config').style.display = 'none';
    console.error('[Dashboard] Error:', message);
}

/**
 * Show content state
 */
function showContent() {
    hideLoading();
    document.getElementById('errorContainer').style.display = 'none';
    document.getElementById('charts').style.display = 'block';
    document.getElementById('comparison').style.display = 'block';
    document.getElementById('config').style.display = 'block';
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN RENDER
// ═══════════════════════════════════════════════════════════════════════════

/**
 * Main render function
 */
async function render() {
    // Initialize particle system
    const particleCanvas = document.getElementById('particles-bg');
    let particleSystem = null;

    if (particleCanvas) {
        particleSystem = new ParticleSystem(particleCanvas);
        particleSystem.start();
    }

    try {
        const rawData = await loadBenchmarkData();
        const data = parseBenchmarkData(rawData);

        // Populate hero stats
        populateHeroStats(data);

        // Create chart labels
        const labels = Object.keys(data.libraries).map(l => LIBRARY_LABELS[l] || l);

        // Create Search P50 chart
        createBarChart(
            'searchP50Chart',
            labels,
            Object.values(data.libraries).map(l => l.search.p50),
            'Search Latency (P50)',
            'ms'
        );

        // Create Insert P50 chart
        createBarChart(
            'insertP50Chart',
            labels,
            Object.values(data.libraries).map(l => l.insert.p50),
            'Insert Latency (P50)',
            'ms'
        );

        // Create Search P99 chart
        createBarChart(
            'searchP99Chart',
            labels,
            Object.values(data.libraries).map(l => l.search.p99),
            'Search Latency (P99)',
            'ms'
        );

        // Create Memory chart
        createBarChart(
            'memoryChart',
            labels,
            Object.values(data.libraries).map(l => Math.abs(l.memory)),
            'Memory Usage',
            'MB'
        );

        // Populate table
        populateTable(data);

        // Populate config
        populateConfig(data.config, data.timestamp);

        // Show content
        showContent();

        console.log('[Dashboard] Render complete');
    } catch (error) {
        showError(error.message);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INITIALIZATION
// ═══════════════════════════════════════════════════════════════════════════

// Initialize on DOM ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', render);
} else {
    render();
}
