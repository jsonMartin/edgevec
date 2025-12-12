# EdgeVec Browser Demo

This example demonstrates how to use EdgeVec in a standard browser environment using native ES Modules.

## Prerequisites

- Python 3 (for simple HTTP server)
- EdgeVec built with `wasm-pack build --target web` (output in `../../pkg`)

## Running the Demo

1. Build the WASM package (from project root):
   ```bash
   wasm-pack build --target web
   ```

2. Navigate to this directory:
   ```bash
   cd examples/browser
   ```

3. Start a local server:
   ```bash
   python3 -m http.server
   ```

4. Open your browser to [http://localhost:8000](http://localhost:8000)

## Features Demonstrated

- Loading the WASM module
- Initializing the `EdgeVec` index
- Inserting floating point vectors
- Performing nearest neighbor search
- Performance logging

