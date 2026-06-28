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

const statusEl = document.getElementById("status-value");
const loadButton = document.getElementById("control-load") as HTMLButtonElement | null;
const stepButton = document.getElementById("control-step") as HTMLButtonElement | null;
const runButton = document.getElementById("control-run") as HTMLButtonElement | null;
const resetButton = document.getElementById("control-reset") as HTMLButtonElement | null;
const logsContentEl = document.querySelector("#sim-logs .panel-content") as HTMLElement | null;

function onHintRegisterStateVisible() {
    // The worker has indicated that the visible registers state is available in the shared memory.
    const registers_base_addr = 0x0000; // Base address of the registers in shared memory
    const registers_length = 4 * 33; // 32 registers + PC
    const mem = new Uint32Array(shared_mem.buffer, registers_base_addr, registers_length / 4);
    console.info("Hint visible state:\nRegs:", mem.subarray(0, 32), "\nPC:", mem[32].toString(16).padStart(8, '0'));

    const registers_table = document.getElementById("registers-table") as HTMLTableElement | null;
    for (let i = 0; i < 32; i++) {
        const rowIndex = Math.floor(i / 8);
        const colIndex = i % 8;
        const cell = registers_table?.rows[rowIndex*2 + 1]?.cells[colIndex];
        if (cell) {
            cell.textContent = `0x${mem[i].toString(16).padStart(8, '0')}`;
            cell.setAttribute("data-value", mem[i].toString());
        }
    }

    const pc_value_span = document.getElementById("pc-value");
    if (pc_value_span) {
        pc_value_span.textContent = `0x${mem[32].toString(16).padStart(8, '0')}`;
        pc_value_span.setAttribute("data-value", mem[32].toString());
    }
}

function onReady() {
    statusEl!.textContent = "Ready";
    loadButton!.disabled = false;
    stepButton!.disabled = true;
    runButton!.disabled = true;
    resetButton!.disabled = true;

    loadButton!.addEventListener("click", onLoad);
}

function onLoad() {
    if (loadButton?.disabled) return;
    logsContentEl!.innerHTML = ""; // Clear logs
    statusEl!.textContent = "Loading ROM...";
    loadButton!.disabled = true;
    stepButton!.disabled = true;
    runButton!.disabled = true;
    resetButton!.disabled = true;

    // Open a file picker dialog to select a ROM file
    const fileInput = document.createElement("input");
    fileInput.type = "file";
    fileInput.accept = ".bin"; // Accept only binary files
    fileInput.onchange = async (event) => {
        const file = (event.target as HTMLInputElement).files?.[0];
        if (file) {
            const arrayBuffer = await file.arrayBuffer();
            const romBytes = new Uint8Array(arrayBuffer);
            console.log(`Loaded ROM file: ${file.name}, size: ${romBytes.length} bytes`);

            // Send the ROM bytes to the worker
            const loadRomMessage: IpcClientMessage = {
                type: IpcClientMessageType.LoadRom,
                rom: romBytes
            };
            worker.postMessage(loadRomMessage);

            // Enable the start button after loading the ROM
            statusEl!.textContent = "ROM Loaded. Ready to run.";
            runButton!.disabled = false;
            runButton!.addEventListener("click", onReset, { once: true });
        } else {
            console.warn("No ROM file selected.");
            loadButton!.disabled = false;
        }
    }
    fileInput.onabort = () => {
        console.warn("ROM file selection was aborted.");
        loadButton!.disabled = false;
    }
    fileInput.click(); // Trigger the file picker dialog
}

function onReset() {
    if (resetButton?.disabled && runButton!.disabled) return;
    logsContentEl!.innerHTML = ""; // Clear logs
    statusEl!.textContent = "Loading ROM...";
    loadButton!.disabled = true;
    stepButton!.disabled = true;
    runButton!.disabled = true;
    resetButton!.disabled = false;

    const startMessage: IpcClientMessage = {
            type: IpcClientMessageType.Start,
        };
    worker.postMessage(startMessage);
}


function onRunning() {
    statusEl!.textContent = "Running";
    loadButton!.disabled = true;
    stepButton!.disabled = true;
    runButton!.disabled = true;
    resetButton!.disabled = true;
}

function onCompleted() {
    statusEl!.textContent = "Completed";
    loadButton!.disabled = false;
    stepButton!.disabled = true;
    runButton!.disabled = true;
    resetButton!.disabled = false;

    loadButton!.addEventListener("click", onLoad);
    resetButton!.addEventListener("click", onReset, { once: true });
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
    } else if (message.type === IpcWorkerMessageType.Ready) {
        console.log("Worker is ready to start the WebAssembly module.");
        onReady();
    } else if (message.type === IpcWorkerMessageType.Running) {
        console.log("Worker is running the WebAssembly module.");
        onRunning();
    } else if (message.type === IpcWorkerMessageType.Completed) {
        console.log("Worker has completed the WebAssembly module execution.");
        onCompleted();
    } else if (message.type === IpcWorkerMessageType.HintRegisterStateVisible) {
        onHintRegisterStateVisible();
    } else if (message.type === IpcWorkerMessageType.ConsoleLog) {
        const logString = message.logString;
        console.info("[Worker Log]", logString);
        if (logsContentEl) {
            logsContentEl.appendChild(document.createElement("p"));
            logsContentEl.lastChild!.textContent = logString;
        }
    } else {
        console.log("Main thread received hint about visible register state from worker.");
    }
}