//! This script is the entry point for the Web Worker that runs the RvEmu emulator.
//!
//! It communicates with the main thread via the `postMessage` API, receiving commands to control the emulator
//! and sending back updates on the emulator's state, such as the contents of the memory etc.
import { IpcClientMessageType, type IpcClientMessage, IpcWorkerMessageType, type IpcWorkerMessage } from "./worker-ipc.mts";

if (self.isSecureContext) {
    console.log("Worker is running in a secure context.");
}
if (self.crossOriginIsolated) {
    console.log("Worker is running in a cross-origin isolated environment.");
} else {
    console.log("Worker is NOT running in a cross-origin isolated environment. Shared memory may not work properly.");
}

self.postMessage({ type: IpcWorkerMessageType.Hello } as IpcWorkerMessage);

const wasmPath = "/rvemu.wasm";
const wasm = fetch(wasmPath, { mode: 'cors' });
let wasmInstance: WebAssembly.Instance | null = null;

function getMem() {
    if (!wasmInstance) {
        throw new Error("WebAssembly instance is not initialized yet.");
    }
    return wasmInstance.exports.memory as WebAssembly.Memory;
}

function buildEnv(shared_mem: WebAssembly.Memory) {
    return  {
        shared_mem,
        keepalive(arg: number) {
            console.log(`Keepalive called with argument: ${arg}`);
        },
        mem_debug(tag: number) {
            console.log("Memory debug (tag: ", tag, "):\nshared_mem:");
            console.log(new Uint8Array(shared_mem.buffer));
            console.log("memory:")
            console.log(new Uint8Array(getMem().buffer));
            console.log("End of memory debug");
        }
    }
};

onmessage = async (event: MessageEvent) => {
    const message = event.data;
    console.log("Worker received message:", message.type);

    if (message.type === IpcClientMessageType.InitArgs) {
        const shared_mem: WebAssembly.Memory = message.shared_mem;
        // Here you can initialize your WebAssembly module with the shared memory.
        wasm.then(response => response.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes, { env: buildEnv(shared_mem) }))
            .then(result => {
                wasmInstance = result.instance;
                console.log("WebAssembly module instantiated successfully.");
                // You can now call functions from the WebAssembly instance as needed.
                const { run } = wasmInstance.exports as any;
                self.postMessage({ type: IpcWorkerMessageType.Running } as IpcWorkerMessage);
                run();
            })
            .catch(error => {
                console.error("Error loading or instantiating WebAssembly module:", error);
                self.postMessage({ type: IpcWorkerMessageType.InitFailed } as IpcWorkerMessage);
            }); 
    } else {
        console.log("Worker received unknown message type:", message.type);
    }
}