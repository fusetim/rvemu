//! This script is the client-side entry point for the Web Worker that runs the RvEmu emulator.
//!
//! It communicates with the main thread via the `postMessage` API, receiving commands to control the emulator
//! and sending back updates on the emulator's state, such as the contents of the memory etc.

console.log('Worker started and ready to run the emulator.');

// Create the environment for the rvemu WebAssembly module, which includes the shared memory and any imported functions.
const shared_mem = new WebAssembly.Memory({ initial: 1, maximum: 1, shared: true });
const env = {
    shared_mem,
    keepalive(arg: number) {
        console.log(`Keepalive called with argument: ${arg}`);
    },
    debug(arg: number) {
        // Print the first 64 i32 of shared_mem
        console.log(`Debug called with argument: ${arg}`);
    }
};

const wasmPath = "/rvemu.wasm";
fetch(wasmPath)
    .then(response => {
        if (!response.ok) {
            console.log(`Failed to load WebAssembly module from ${wasmPath}: ${response.statusText}`);
        }
            console.log(`Successfully fetched WebAssembly module from ${wasmPath}`);
            return response.arrayBuffer();
    })
    .then(bytes => WebAssembly.instantiate(bytes, { env }))
    .then(result => {
        console.log(`WebAssembly module instantiated successfully.`);
        return result.instance;
    })
    .catch(error => {
        console.log(`Error loading or instantiating WebAssembly module: ${error}`);
    })
    .then(instance => {
        if (instance) {
            console.log(`Calling the 'run' function exported by the WebAssembly module.`);
            const { run } = instance.exports as { run: () => void };
            run();
            console.log("rvemu quitted!");
        }
    })
    .catch(error => {
        console.log(`Error: ${error}`);
    });

console.log(`Loading WebAssembly module from: ${wasmPath}`);