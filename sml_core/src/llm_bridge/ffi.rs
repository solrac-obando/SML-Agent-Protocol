use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

static LLAMA_CONTEXT: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static LLAMA_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct LlamaContext {
    ptr: NonNull<LlamaContext>,
}

#[repr(C)]
pub struct LlamaToken {
    pub id: i32,
    pub logprob: f32,
}

pub type LlamaCallback = extern "C" fn(token: *const LlamaToken, context: *mut std::ffi::c_void);

pub struct LlamaConfig {
    pub model_path: *const std::ffi::c_char,
    pub n_ctx: usize,
    pub n_threads: usize,
    pub n_gpu_layers: usize,
    pub seed: i32,
}

impl Default for LlamaConfig {
    fn default() -> Self {
        Self {
            model_path: std::ptr::null(),
            n_ctx: 2048,
            n_threads: 4,
            n_gpu_layers: 0,
            seed: -1,
        }
    }
}

pub fn initialize_ffi() -> Result<(), FfiError> {
    if LLAMA_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(());
    }

    #[cfg(feature = "llama-ffi")]
    {
        #[cfg(target_os = "linux")]
        unsafe {
            if !llama_cpp_ffi::llama_backend_init() {
                return Err(FfiError::InitFailed("llama_backend_init failed".to_string()));
            }
        }
    }

    #[cfg(not(feature = "llama-ffi"))]
    {
        let _ = LLAMA_CONTEXT.load(Ordering::SeqCst);
    }

    LLAMA_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn is_llama_available() -> bool {
    #[cfg(feature = "llama-ffi")]
    {
        unsafe { !LLAMA_CONTEXT.load(Ordering::SeqCst).is_null() }
    }
    #[cfg(not(feature = "llama-ffi"))]
    {
        false
    }
}

#[derive(Debug)]
pub enum FfiError {
    InitFailed(String),
    LoadModelFailed(String),
    InferenceFailed(String),
    NullPointer,
    NotInitialized,
}

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FfiError::InitFailed(msg) => write!(f, "[FFI_INIT_ERROR] {}", msg),
            FfiError::LoadModelFailed(msg) => write!(f, "[FFI_MODEL_ERROR] {}", msg),
            FfiError::InferenceFailed(msg) => write!(f, "[FFI_INFERENCE_ERROR] {}", msg),
            FfiError::NullPointer => write!(f, "[FFI_NULL_POINTER]"),
            FfiError::NotInitialized => write!(f, "[FFI_NOT_INITIALIZED]"),
        }
    }
}

impl std::error::Error for FfiError {}

pub struct SmlTokenizer;

impl SmlTokenizer {
    pub fn tokenize_sml(input: &str) -> Vec<u32> {
        let mut tokens = Vec::new();
        
        let mut current_word = String::new();
        for ch in input.chars() {
            match ch {
                '@' | '[' | ']' | ':' | '|' => {
                    if !current_word.is_empty() {
                        tokens.extend(tokenize_word(&current_word));
                        current_word.clear();
                    }
                    tokens.push(ch as u32);
                }
                _ => current_word.push(ch),
            }
        }
        
        if !current_word.is_empty() {
            tokens.extend(tokenize_word(&current_word));
        }
        
        tokens
    }
}

fn tokenize_word(word: &str) -> Vec<u32> {
    word.chars().map(|c| c as u32 + 256).collect()
}

pub fn estimate_token_count(sml_command: &str) -> usize {
    let chars = sml_command.chars().count();
    (chars + 3) / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_sml() {
        let tokens = SmlTokenizer::tokenize_sml("@[read:src/main.rs]");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_estimate_tokens() {
        let count = estimate_token_count("@[read:src/main.rs]");
        assert_eq!(count, 5);
    }
}