use std::env;
use walrus::{FunctionBuilder, FunctionId, FunctionKind, LocalFunction, LocalId, Memory, MemoryId, Module, ModuleFunctions, ValType, ir::{LoadKind, MemArg, StoreKind}};

fn main() -> walrus::Result<()> {
    env_logger::init();

    // Load the given wasm file in argument
    let ar = env::args().nth(1).unwrap();
    let out = env::args().nth(2).unwrap_or_else(|| "output.wasm".to_string());
    let mut module = Module::from_file(ar).unwrap();

    // Get the already defined default memory
    let default_mem = module.memories.iter().next().expect("No default memory found").id();

    // Define an imported shared memory (1 page = 64KB)
    let (sm_mem, sm_import)  = module.add_import_memory("env", "shared_mem", true, false, 1, Some(1), None);

    // Replace the imported stub functions (if they exist) with the generated ones
    let imported_funcs = module.funcs.iter().filter_map(|func| {
        if let FunctionKind::Import(import) = &func.kind {
            Some(func.id())
        } else {
            None
        }
    }).collect::<Vec<_>>();
    let mut imported_fns = vec![];
    for func_id in imported_funcs {
        if let Some(func) = module.imports.get_imported_func(func_id) {
            if func.module == "shared_helper" {
                imported_fns.push((func.name.clone(), func_id, func.id()));
            }
        } 
    }
    for (name, func_id, import_id) in imported_fns {
        let new_func = match name.as_str() {
            "shared_load_word" => gen_shared_load_word(&mut module, sm_mem),
            "shared_store_word" => gen_shared_store_word(&mut module, sm_mem),
            "shared_copyfrom" => gen_shared_copyfrom(&mut module, default_mem, sm_mem),
            "shared_copyto" => gen_shared_copyto(&mut module, default_mem, sm_mem),
            "shared_atomic_wait" => gen_shared_atomic_wait(&mut module, sm_mem),
            "shared_atomic_store" => gen_shared_atomic_store(&mut module, sm_mem),
            _ => continue,
        };
        let function = module.funcs.get_mut(func_id);
        function.kind = FunctionKind::Local(new_func);
        println!("Function {} @{}", name, function.id().index());
        module.imports.delete(import_id);
    }

    // Remove data segments (if any) since we don't need them and they might conflict with the shared memory
    let data_ids = module.data.iter().map(|data| data.id()).collect::<Vec<_>>();
    for data_id in data_ids {
        module.data.delete(data_id);
    }

    // Write the modified module to a new file
    module.emit_wasm_file(out)?;

    Ok(())
}

/// Generate a function that loads a word from the shared memory to the default memory.
/// 
/// ```wat
/// # share_load_word(i32 offset) -> i32
/// shared_load_word:
///     local.get 0 # offset
///     i32.load 1  # load 32-bit word in memory 1
///     end
/// ```
fn gen_shared_load_word(module: &mut Module, shared_mem: MemoryId) -> LocalFunction {
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let offset = module.locals.add(ValType::I32);
    builder.func_body()
        .local_get(offset)
        .load(shared_mem, LoadKind::I32 { atomic: false }, MemArg { align: 4, offset: 0 });
    builder.local_func(vec![offset])
}

/// Generate a function that stores a word from the default memory to the shared memory.
/// 
/// ```wat
/// # shared_store_word(i32 offset, i32 value) -> ()
/// shared_store_word:
///     local.get 0 # offset
///     local.get 1 # value
///     i32.store 1 # store 32-bit word in memory 1
///     end
/// ```
fn gen_shared_store_word(module: &mut Module, shared_mem: MemoryId) -> LocalFunction {
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let offset = module.locals.add(ValType::I32);
    let value = module.locals.add(ValType::I32);
    builder.func_body()
        .local_get(offset)
        .local_get(value)
        .store(shared_mem, StoreKind::I32 { atomic: false }, MemArg { align: 4, offset: 0 });
    builder.local_func(vec![offset, value])
}

/// Generate a function that copy from the shared memory to the default memory.
/// 
/// ```wat
/// # shared_copyfrom(i32 dest_offset, i32 src_offset, i32 length) -> ()
/// shared_copyfrom:
///     local.get 0 # dest_offset
///     local.get 1 # src_offset
///     local.get 2 # length
///     memory.copy 1 0 # copy from memory 0 to memory 1
///     end
/// ```
fn gen_shared_copyfrom(module: &mut Module, default_mem: MemoryId, shared_mem: MemoryId) -> LocalFunction {
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32, ValType::I32], &[]);
    let dest_offset = module.locals.add(ValType::I32);
    let src_offset = module.locals.add(ValType::I32);
    let length = module.locals.add(ValType::I32);
    builder.func_body()
        .local_get(dest_offset)
        .local_get(src_offset)
        .local_get(length)
        .memory_copy(shared_mem, default_mem);
    builder.local_func(vec![dest_offset, src_offset, length])
}

/// Generate a function that copy to shared memory from the default memory.
/// 
/// ```wat
/// # shared_copyto(i32 dest_offset, i32 src_offset, i32 length) -> ()
/// shared_copyto:
///     local.get 0 # dest_offset
///     local.get 1 # src_offset
///     local.get 2 # length
///     memory.copy 0 1 # copy from memory 1 to memory 0
///     end
/// ```
fn gen_shared_copyto(module: &mut Module, default_mem: MemoryId, shared_mem: MemoryId) -> LocalFunction {
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32, ValType::I32], &[]);
    let dest_offset = module.locals.add(ValType::I32);
    let src_offset = module.locals.add(ValType::I32);
    let length = module.locals.add(ValType::I32);
    builder.func_body()
        .local_get(dest_offset)
        .local_get(src_offset)
        .local_get(length)
        .memory_copy(default_mem, shared_mem);
    builder.local_func(vec![dest_offset, src_offset, length])
}

/// Generate a function that atomicly wait on an address of the shared memory
/// 
/// ```wat
/// # shared_atomic_wait(i32 offset, i32 expr, i64 timeout_ns) -> i32
/// shared_atomic_wait:
///     local.get 0 # offset
///     local.get 1 # expr
///     local.get 2 # timeout_ns
///     memory.atomic.wait 1 # wait on address in memory 1
///     end
/// ```
fn gen_shared_atomic_wait(module: &mut Module, shared_mem: MemoryId) -> LocalFunction {
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32, ValType::I64], &[ValType::I32]);
    let offset = module.locals.add(ValType::I32);
    let expr = module.locals.add(ValType::I32);
    let timeout_ns = module.locals.add(ValType::I64);
    builder.func_body()
        .local_get(offset)
        .local_get(expr)
        .local_get(timeout_ns)
        .atomic_wait(shared_mem, MemArg { align: 4, offset: 0 }, false);
    builder.local_func(vec![offset, expr, timeout_ns])
}

/// Generate a function that atomicly store a value at an address of the shared memory
/// 
/// ```wat
/// # shared_atomic_store(i32 offset, i32 value) -> ()
/// shared_atomic_store:
///     local.get 0 # offset
///     local.get 1 # value
///     memory.atomic.store 1 # store value at address in memory 1
///     end
/// ```
fn gen_shared_atomic_store(module: &mut Module, shared_mem: MemoryId) -> LocalFunction {
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let offset = module.locals.add(ValType::I32);
    let value = module.locals.add(ValType::I32);
    builder.func_body()
        .local_get(offset)
        .local_get(value)
        .store(shared_mem, StoreKind::I32 { atomic: true }, MemArg { align: 4, offset: 0 });
    builder.local_func(vec![offset, value])
}