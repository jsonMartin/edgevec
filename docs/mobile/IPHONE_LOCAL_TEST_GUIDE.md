# iPhone Local Testing Guide

**Version:** 1.0.0
**Target:** EdgeVec v0.5.4 iOS Safari Testing
**Date:** 2025-12-20

---

## Quick Start (5 Minutes)

### Step 1: Find Your Computer's IP Address

**Windows:**
```cmd
ipconfig
```
Look for `IPv4 Address` under your active network adapter (e.g., `192.168.1.100`)

**Mac/Linux:**
```bash
ifconfig | grep "inet "
```

### Step 2: Start the HTTP Server

Open a terminal in the EdgeVec project root:

```bash
cd edgevec
python -m http.server 8080 --bind 0.0.0.0
```

You should see:
```
Serving HTTP on 0.0.0.0 port 8080 ...
```

**Leave this terminal open!**

### Step 3: Open on iPhone

On your iPhone, open Safari and go to:
```
http://<YOUR_IP>:8080/wasm/examples/index.html
```

Example: `http://192.168.1.100:8080/wasm/examples/index.html`

**Make sure your iPhone is on the same WiFi network as your computer!**

---

## Testing Checklist

### Test 1: Demo Catalog (index.html)

URL: `http://<IP>:8080/wasm/examples/index.html`

| Test | Expected | Result |
|:-----|:---------|:-------|
| Page loads | "EdgeVec" title visible | [ ] Pass / [ ] Fail |
| No horizontal scroll | Can't swipe left/right | [ ] Pass / [ ] Fail |
| Tap "Filter Playground" | Navigates correctly | [ ] Pass / [ ] Fail |
| Tap "Benchmark Dashboard" | Navigates correctly | [ ] Pass / [ ] Fail |
| Tap back button | Returns to index | [ ] Pass / [ ] Fail |

### Test 2: Filter Playground

URL: `http://<IP>:8080/wasm/examples/filter-playground.html`

| Test | Expected | Result |
|:-----|:---------|:-------|
| Page loads | "Filter Playground" title | [ ] Pass / [ ] Fail |
| WASM loads | Green "WASM module loaded" | [ ] Pass / [ ] Fail |
| Type `category = "test"` | "Valid filter" status | [ ] Pass / [ ] Fail |
| Type `price > 100` | Parses successfully | [ ] Pass / [ ] Fail |
| Type `invalid!!!` | Error message appears | [ ] Pass / [ ] Fail |
| Tap example buttons | Filter loads and parses | [ ] Pass / [ ] Fail |

### Test 3: Benchmark Dashboard

URL: `http://<IP>:8080/wasm/examples/benchmark-dashboard.html`

| Test | Expected | Result |
|:-----|:---------|:-------|
| Page loads | Dashboard visible | [ ] Pass / [ ] Fail |
| Charts render | See graphs with data | [ ] Pass / [ ] Fail |
| Tap "Run Benchmark" | Metrics populate | [ ] Pass / [ ] Fail |
| Check "Filter Overhead" | Percentage (NOT NaN%) | [ ] Pass / [ ] Fail |

---

## Troubleshooting

### "Cannot connect to server"

1. Check your iPhone is on the same WiFi as your computer
2. Verify the IP address is correct: `ipconfig` or `ifconfig`
3. Check your computer's firewall allows port 8080
4. Try `http://` not `https://` (WASM requires HTTPS only on some older iOS)

### "WASM module failed to load"

1. Clear Safari cache: Settings > Safari > Clear History and Website Data
2. Disable content blockers if any
3. Check console for errors (requires Mac + Web Inspector)

### "Function is undefined" Error

This means the WASM loaded but exports are missing:
1. Rebuild WASM: `wasm-pack build --target web --out-dir pkg`
2. Hard refresh: Long-press the refresh button in Safari
3. Try a different browser (Chrome for iOS uses WebKit too, so same behavior)

### Slow Performance

1. Expected: iOS Safari may be 2-3x slower than desktop Chrome
2. Use smaller vector counts (1k-5k instead of 10k+)
3. Close other Safari tabs

---

## Debug with Mac (Optional)

If you have a Mac, you can see console logs from Safari on iPhone:

1. On iPhone: Settings > Safari > Advanced > Web Inspector ON
2. Connect iPhone to Mac with USB cable
3. On Mac: Safari > Develop > [Your iPhone] > [Your Page]
4. You can now see console.log output and JavaScript errors

---

## Quick Commands

```bash
# Start server (from edgevec directory)
python -m http.server 8080 --bind 0.0.0.0

# Alternative: Node.js http-server
npx http-server -p 8080 --cors

# Find IP (Windows)
ipconfig

# Find IP (Mac/Linux)
ifconfig | grep "inet "
```

---

## Known Limitations

1. **iOS Safari Memory:** Limited to ~1GB for WASM
2. **Performance:** 2-3x slower than desktop Chrome
3. **No SharedArrayBuffer:** Limits threading (no wasm-bindgen-rayon)
4. **IndexedDB Size:** Limited in Safari compared to Chrome

---

## Report Issues

If tests fail, please report:
1. iPhone model (e.g., iPhone 12)
2. iOS version (e.g., 17.2)
3. Which test failed
4. Error message (if any)
5. Screenshot of the issue

---

**Document:** IPHONE_LOCAL_TEST_GUIDE.md
**Authority:** WASM_SPECIALIST
**Version:** 1.0.0
