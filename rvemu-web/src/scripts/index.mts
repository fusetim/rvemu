import { IpcClientMessageType, type IpcClientMessage, IpcWorkerMessageType, type IpcWorkerMessage } from "./worker-ipc.mts";

console.log("Hello Main Thread!");

// Ensure we are in a secure context and cross-origin isolated environment
if (self.isSecureContext) {
    console.log("Main thread is running in a secure context.");
}

if (self.crossOriginIsolated) {
    console.log("Main thread is running in a cross-origin isolated environment.");
} else {
    console.log("Main thread is NOT running in a cross-origin isolated environment. Shared memory may not work properly.");
    alert("Warning: The main thread is NOT running in a cross-origin isolated environment. Shared memory may not work properly.");
}

// Launch the Web Worker that runs the RvEmu emulator.
const shared_mem = new WebAssembly.Memory({ initial: 1, maximum: 1, shared: true });
const worker = new Worker(new URL("worker.mts", import.meta.url), { type: 'module' });

function shared_mem_interaction() {
    // Example interaction with the shared memory.
    const sharedArray = new Int32Array(shared_mem.buffer);
    console.log("Shared memory contents before modification:", sharedArray);

    // Modify the shared memory (for demonstration purposes).
    Atomics.store(sharedArray, 0, 18); // Store the value 18 at index 0 in a thread-safe manner.
    Atomics.notify(sharedArray, 0, 1); // Notify any waiting threads that the value has changed.

    console.log("Shared memory contents after modification:", sharedArray);
}

worker.onmessage = (event: MessageEvent) => {
    const message = event.data;
    console.log("Main thread received message from worker:", message);

    if (message.type === IpcWorkerMessageType.Hello) {
        console.log("Worker is initialized and awaiting arguments.");
        // Send the shared memory to the worker.
        const initMessage: IpcClientMessage = {
            type: IpcClientMessageType.InitArgs,
            shared_mem
        };
        worker.postMessage(initMessage);
    } else if (message.type === IpcWorkerMessageType.InitFailed) {
        console.error("Worker failed to initialize the WebAssembly module.");
    } else if (message.type === IpcWorkerMessageType.Running) {
        console.log("Worker is running the WebAssembly module.");

        setTimeout(() => shared_mem_interaction(), 4000); // Delay to ensure the worker has started running.
    } else {
        console.log("Main thread received unknown message type from worker:", message.type);
    }
}