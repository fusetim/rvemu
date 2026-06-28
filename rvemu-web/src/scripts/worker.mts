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

const wasmPath = "/rvemu.wasm";
const wasm = fetch(wasmPath, { mode: 'cors' });
let wasmInstance: WebAssembly.Instance | null = null;
let hostArena = [null] as any[]; // Start with a null entry to avoid using 0 as a valid handle
let shared_mem: WebAssembly.Memory | null = null;
let rom = await fetch("/small_s_instructions.bin").then(res => res.arrayBuffer().then(buf => new Uint8Array(buf)));

self.postMessage({ type: IpcWorkerMessageType.Hello } as IpcWorkerMessage);

function return_indirect(handle: number, len: number): number {
    // Create a HostMemoryHandle object in the host environment's memory and return its handle (pointer).
    let raw_handle = new Uint8Array(8);
    raw_handle[0] = handle & 0xFF;
    raw_handle[1] = (handle >> 8) & 0xFF;
    raw_handle[2] = (handle >> 16) & 0xFF;
    raw_handle[3] = (handle >> 24) & 0xFF;
    raw_handle[4] = len & 0xFF;
    raw_handle[5] = (len >> 8) & 0xFF;
    raw_handle[6] = (len >> 16) & 0xFF;
    raw_handle[7] = (len >> 24) & 0xFF;
    const handlePtr = hostArena.length;
    hostArena.push(raw_handle);
    return handlePtr;
}

const rvemu_host = {
    // Load the ROM bytes 
    load_rom: () => {
        // TODO: Ask for the ROM though IPC to the main thread, and wait for the response.
        const length = rom.length; 
        const handle = hostArena.length;
        hostArena.push(rom);
        console.debug(`load_rom() called, returning handle: ${handle}, length: ${length}`);
        return return_indirect(handle, length);
    },
    // Hint from the emulator that the visible registers state is present and visible inside the shared_memory
    hint_visible_registers_state: () => {
        postMessage({ type: IpcWorkerMessageType.HintRegisterStateVisible } as IpcWorkerMessage);
    }
};

function getMem() {
    if (!wasmInstance) {
        throw new Error("WebAssembly instance is not initialized yet.");
    }
    return wasmInstance.exports.memory as WebAssembly.Memory;
}

function buildEnv(shared_mem: WebAssembly.Memory) {
    return  {
        shared_mem,
        obj_free: (handle: number) => {
            // Free the object in the host arena
            console.debug(`obj_free(handle: ${handle})`);
            hostArena[handle] = undefined;
        },
        obj_copyin: (ptr: number, len: number, handle: number) => {
            // Copy an object (also known as a memory region) from the host environment to the wasm module's main memory.
            console.debug(`obj_copyin(ptr: ${ptr}, len: ${len}, handle: ${handle})`);
            const mem = new Uint8Array(getMem().buffer, ptr, len);
            mem.set(new Uint8Array(hostArena[handle]));
            return len; // Return the number of bytes copied           
        },
        obj_copyout: (ptr: number, len: number) => {
            // Copy an object (also known as a memory region) from the wasm module's main memory to the host environment.
            console.debug(`obj_copyout(ptr: ${ptr}, len: ${len})`);
            const index = hostArena.length;
            const mem = new Uint8Array(getMem().buffer, ptr, len);
            hostArena.push(new Uint8Array(mem));
            return index; // Return the handle to the copied object in the host arena
        },
        console_log: (handle: number) => {
            const logData = hostArena[handle];
            if (logData !== undefined && logData !== null) {
                const logString = new TextDecoder().decode(logData);
                postMessage({ type: IpcWorkerMessageType.ConsoleLog, logString } as IpcWorkerMessage);
                console.info("[rvemu-wasm]", logString);
            } else {
                console.error("Invalid handle for console_log:", handle);
            }
        }
    }
};

onmessage = async (event: MessageEvent) => {
    const message = event.data;
    console.log("Worker received message:", message.type);

    if (message.type === IpcClientMessageType.InitArgs) {
        shared_mem = message.shared_mem;
        // Here you can initialize your WebAssembly module with the shared memory.
        wasm.then(response => response.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes, { env: buildEnv(shared_mem), rvemu_host }))
            .then(result => {
                wasmInstance = result.instance;
                console.log("WebAssembly module instantiated successfully.");
                // You can now call functions from the WebAssembly instance as needed.
                self.postMessage({ type: IpcWorkerMessageType.Ready } as IpcWorkerMessage);
            })
            .catch(error => {
                console.error("Error loading or instantiating WebAssembly module:", error);
                self.postMessage({ type: IpcWorkerMessageType.InitFailed } as IpcWorkerMessage);
            }); 
    } else if (message.type === IpcClientMessageType.LoadRom) {
        rom = message.rom;
    } else if (message.type === IpcClientMessageType.Start) {
        if (wasmInstance) {
            const run = wasmInstance.exports.run as Function;
            self.postMessage({ type: IpcWorkerMessageType.Running } as IpcWorkerMessage);
            run();
            self.postMessage({ type: IpcWorkerMessageType.Completed } as IpcWorkerMessage);
        } else {
            console.error("Cannot start WebAssembly module: instance is not initialized.");
            self.postMessage({ type: IpcWorkerMessageType.InitFailed } as IpcWorkerMessage);
        }
    } else {
        console.log("Worker received unknown message type:", message.type);
    }
}