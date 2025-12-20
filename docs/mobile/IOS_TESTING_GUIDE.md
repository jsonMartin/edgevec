# iOS Safari Testing Guide — Step by Step

**Version:** EdgeVec v0.5.3
**Date:** 2025-12-19
**Purpose:** Quick manual testing on real iOS device

---

## Prerequisites

- ✅ iPhone or iPad with iOS 17+ and Safari
- ✅ Computer and phone on the **same WiFi network**
- ✅ Python installed on computer (for simple HTTP server)

---

## Step 1: Start the Local Server (2 minutes)

Open a terminal/command prompt on your computer:

```bash
# Navigate to EdgeVec project
cd "C:\Users\matte\Desktop\Desktop OLD\AI\Università AI\courses\personal_project\fortress_problem_driven\research_fortress\edgevec"

# Start simple HTTP server
python -m http.server 8080
```

You should see:
```
Serving HTTP on :: port 8080 (http://[::]:8080/) ...
```

**Keep this terminal open!**

---

## Step 2: Find Your Computer's IP Address (1 minute)

Open a **new** terminal/command prompt:

**On Windows:**
```bash
ipconfig | findstr "IPv4"
```

Look for something like: `IPv4 Address. . . . . . . . . . . : 192.168.1.100`

**On macOS/Linux:**
```bash
ifconfig | grep "inet " | grep -v 127.0.0.1
```

**Write down your IP address:** `___.___.___.___ `

---

## Step 3: Test on iPhone/iPad (10 minutes)

### 3.1 Open Safari on your iOS device

### 3.2 Navigate to Demo Catalog

Enter this URL in Safari (replace with YOUR IP):

```
http://YOUR_IP_HERE:8080/wasm/examples/index.html
```

**Example:** `http://192.168.1.100:8080/wasm/examples/index.html`

---

## Test Checklist

### Test A: Demo Catalog Page

| Check | How to Verify | Result |
|:------|:--------------|:-------|
| Page loads | See "EdgeVec Demos" title | ⬜ PASS / ⬜ FAIL |
| All demo links visible | See Filter, Benchmark, Soft Delete links | ⬜ PASS / ⬜ FAIL |
| Links are tappable | Tap responds immediately | ⬜ PASS / ⬜ FAIL |

---

### Test B: Filter Playground

Navigate to: `http://YOUR_IP:8080/wasm/examples/filter-playground.html`

| Check | How to Verify | Result |
|:------|:--------------|:-------|
| Page loads | See "Filter Playground" title | ⬜ PASS / ⬜ FAIL |
| WASM loads | No JavaScript errors (check console*) | ⬜ PASS / ⬜ FAIL |
| Input works | Tap text input, keyboard appears | ⬜ PASS / ⬜ FAIL |
| Filter parses | Type `category = "test"`, see parsed result | ⬜ PASS / ⬜ FAIL |
| Error shows | Type `invalid syntax`, see error message | ⬜ PASS / ⬜ FAIL |

**Quick filter to try:** `price > 100 AND category = "electronics"`

---

### Test C: Benchmark Dashboard

Navigate to: `http://YOUR_IP:8080/wasm/examples/benchmark-dashboard.html`

| Check | How to Verify | Result |
|:------|:--------------|:-------|
| Page loads | See "Benchmark Dashboard" title | ⬜ PASS / ⬜ FAIL |
| Charts render | See Chart.js visualizations | ⬜ PASS / ⬜ FAIL |
| Run benchmark | Tap "Run" button, see progress | ⬜ PASS / ⬜ FAIL |
| Results display | See timing results after completion | ⬜ PASS / ⬜ FAIL |

---

### Test D: Soft Delete Demo

Navigate to: `http://YOUR_IP:8080/wasm/examples/soft_delete.html`

| Check | How to Verify | Result |
|:------|:--------------|:-------|
| Page loads | See demo interface | ⬜ PASS / ⬜ FAIL |
| Vectors insert | Tap insert, see vector count increase | ⬜ PASS / ⬜ FAIL |
| Delete works | Tap delete, see tombstone count increase | ⬜ PASS / ⬜ FAIL |
| Search works | Perform search, see results | ⬜ PASS / ⬜ FAIL |
| Compact works | Tap compact, see tombstones cleared | ⬜ PASS / ⬜ FAIL |

---

## Quick Console Check (Optional but Recommended)

### Enable Web Inspector on iOS:
1. On iPhone: **Settings > Safari > Advanced > Web Inspector = ON**
2. Connect iPhone to Mac via USB cable
3. On Mac: Open Safari > Develop menu > [Your iPhone] > [Demo page]
4. Check Console tab for any errors

### What to Look For:
```
✅ GOOD: "[EdgeVec] WASM module loaded"
✅ GOOD: "[EdgeVec] Ready"
❌ BAD:  "RangeError: Out of memory"
❌ BAD:  "WebAssembly.instantiate failed"
❌ BAD:  Any red error text
```

---

## Quick Smoke Test (Advanced)

If you have console access, paste this JavaScript:

```javascript
// Quick EdgeVec smoke test
(async function() {
    console.log('=== EdgeVec iOS Smoke Test ===');

    // Test 1: Check if EdgeVec loaded
    if (typeof EdgeVec === 'undefined') {
        console.error('FAIL: EdgeVec not loaded');
        return;
    }
    console.log('PASS: EdgeVec loaded');

    // Test 2: Create config
    try {
        const config = new EdgeVecConfig(128);
        console.log('PASS: Config created');
    } catch (e) {
        console.error('FAIL: Config error:', e);
        return;
    }

    // Test 3: Create index
    try {
        const config = new EdgeVecConfig(128);
        const db = new EdgeVec(config);
        console.log('PASS: Index created');

        // Test 4: Insert vectors
        for (let i = 0; i < 100; i++) {
            const v = new Float32Array(128);
            for (let j = 0; j < 128; j++) v[j] = Math.random();
            db.insert(v);
        }
        console.log('PASS: Inserted 100 vectors');

        // Test 5: Search
        const query = new Float32Array(128).fill(0.5);
        const results = db.search(query, 10);
        console.log('PASS: Search returned', results.length, 'results');

        // Test 6: Filter API
        if (typeof Filter !== 'undefined') {
            const filter = Filter.parse('category = "test"');
            console.log('PASS: Filter parsed');
        }

        console.log('=== ALL TESTS PASSED ===');
    } catch (e) {
        console.error('FAIL:', e);
    }
})();
```

---

## What to Report

After testing, update `docs/mobile/IOS_TEST_RESULTS.md` with:

1. **Device:** (e.g., "iPhone 14 Pro, iOS 17.4")
2. **Test Date:** YYYY-MM-DD
3. **Results:** PASS/FAIL for each test
4. **Any errors:** Copy exact error messages
5. **Screenshots:** If any visual issues

---

## Common Issues and Fixes

### "Cannot connect to server"
- Check both devices are on same WiFi
- Try disabling Windows Firewall temporarily
- Verify IP address is correct

### "Page loads but nothing works"
- WASM may not have loaded
- Check console for errors
- Ensure Python server is still running

### "RangeError: Out of memory"
- This is the iOS WASM memory limit
- Expected for very large operations
- EdgeVec should work for normal use cases

### "Keyboard doesn't appear"
- Tap directly on text input
- Try double-tap if single tap doesn't work
- This is an iOS Safari behavior, not EdgeVec bug

---

## Estimated Time

| Task | Time |
|:-----|:-----|
| Setup server | 2 min |
| Find IP | 1 min |
| Test all demos | 10-15 min |
| **Total** | **~15 minutes** |

---

## After Testing

1. **Update** `docs/mobile/IOS_TEST_RESULTS.md` with actual results
2. **Commit** the results
3. **Report** any issues found

**Command to commit results:**
```bash
git add docs/mobile/IOS_TEST_RESULTS.md
git commit -m "docs(mobile): Add actual iOS Safari test results"
```

---

**Happy Testing!** 🎉
