/// Client-to-worker message types for IPC (Inter-Process Communication).
export enum IpcClientMessageType {
    /*
    Pass arguments to the worker (includes the shared memory object for instance).
    */
    InitArgs,
    /* 
    Pass the new ROM bytes to use
    */
    LoadRom,
    /*
    Start the wasm module execution.
    */
    Start
}

/// Worker-to-client message types for IPC (Inter-Process Communication).
export enum IpcWorkerMessageType {
    /* 
    Worker is initialized and awaiting "arguments" from the client side.
    */
    Hello,
    /* Worker is ready to start the wasm module execution */
    Ready,
    /* Worker is running the wasm module */
    Running,
    /* Worker has completed the wasm module execution */
    Completed,
    /* Wasm module initialization failure */
    InitFailed,
    /* Hint that the visible registers state is available */
    HintRegisterStateVisible,
    /* Console log message from the worker */
    ConsoleLog
}

export interface IpcClientMessageBase {
    type: IpcClientMessageType;
}

export interface IpcClientMsgInitArgs extends IpcClientMessageBase {
    type: IpcClientMessageType.InitArgs;
    shared_mem: WebAssembly.Memory;
}

export interface IpcClientMsgLoadRom extends IpcClientMessageBase {
    type: IpcClientMessageType.LoadRom;
    rom: Uint8Array;
}

export interface IpcClientMsgStart extends IpcClientMessageBase {
    type: IpcClientMessageType.Start;
}

export type IpcClientMessage = IpcClientMsgInitArgs | IpcClientMsgStart | IpcClientMsgLoadRom;

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

export interface IpcWorkerMsgHintRegisterStateVisible extends IpcWorkerMessageBase {
    type: IpcWorkerMessageType.HintRegisterStateVisible;
}

export interface IpcWorkerMsgReady extends IpcWorkerMessageBase {
    type: IpcWorkerMessageType.Ready;
}

export interface IpcWorkerMsgCompleted extends IpcWorkerMessageBase {
    type: IpcWorkerMessageType.Completed;
}

export interface IpcWorkerMsgConsoleLog extends IpcWorkerMessageBase {
    type: IpcWorkerMessageType.ConsoleLog;
    logString: string;
}

export type IpcWorkerMessage = IpcWorkerMsgHello | IpcWorkerMsgInitFailed | IpcWorkerMsgRunning |
                               IpcWorkerMsgHintRegisterStateVisible | IpcWorkerMsgReady | 
                               IpcWorkerMsgCompleted | IpcWorkerMsgConsoleLog;