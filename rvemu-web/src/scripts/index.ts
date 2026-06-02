console.log("Hello Main Thread!")

// Launch the Web Worker that runs the RvEmu emulator.
const worker = new Worker(new URL("worker.ts", import.meta.url));

console.log(worker);

worker.onmessage = (event) => {
    console.log('Message from worker:', event.data);
};
