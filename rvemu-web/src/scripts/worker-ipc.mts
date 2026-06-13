/// Client-to-worker message types for IPC (Inter-Process Communication).
export enum IpcClientMessageType {
    /*
    Pass arguments to the worker (includes the shared memory object for instance).
    */
    InitArgs,
}

/// Worker-to-client message types for IPC (Inter-Process Communication).
export enum IpcWorkerMessageType {
    /* 
    Worker is initialized and awaiting "arguments" from the client side.
    */
    Hello, 
    /* Worker is running the wasm module */
    Running,
    /* Wasm module initialization failure */
    InitFailed
}

export interface IpcClientMessageBase {
    type: IpcClientMessageType;
}

export interface IpcClientMsgInitArgs extends IpcClientMessageBase {
    type: IpcClientMessageType.InitArgs;
    shared_mem: WebAssembly.Memory;
}

export type IpcClientMessage = IpcClientMsgInitArgs;

export interface IpcWorkerMessageBase {
    type: IpcWorkerMessageType;
}

export interface IpcWorkerMsgHello extends IpcWorkerMessageBase {
    type: IpcWorkerMessageType.Hello;
}

export interface IpcWorkerMsgInitFailed extends IpcWorkerMessageBase {
    type: IpcWorkerMessageType.InitFailed;
}

export interface IpcWorkerMsgRunning extends IpcWorkerMessageBase {
    type: IpcWorkerMessageType.Running;
}

export type IpcWorkerMessage = IpcWorkerMsgHello | IpcWorkerMsgInitFailed | IpcWorkerMsgRunning;