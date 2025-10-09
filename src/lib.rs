use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Input {
    min: u32,
    max: u32,
}

#[derive(Serialize)]
struct Output {
    random_number: u32,
}

// Use WASI random_get for proper randomness
// getrandom crate will automatically use wasi::random_get when compiled for wasm32-wasi
// No custom implementation needed - the executor provides random_get host function

#[no_mangle]
pub extern "C" fn execute(input_ptr: i32, input_len: i32) -> i32 {
    // Read input from memory
    let input_slice = unsafe {
        std::slice::from_raw_parts(input_ptr as *const u8, input_len as usize)
    };

    // Parse input JSON
    let input: Input = match serde_json::from_slice(input_slice) {
        Ok(i) => i,
        Err(_) => {
            // Return error code
            return -1;
        }
    };

    // Generate random number using getrandom
    let mut buf = [0u8; 4];
    if getrandom::getrandom(&mut buf).is_err() {
        return -2;
    }
    let random = u32::from_le_bytes(buf);

    // Calculate result in range
    let result = if input.max > input.min {
        input.min + (random % (input.max - input.min + 1))
    } else {
        input.min
    };

    // Create output
    let output = Output {
        random_number: result,
    };

    // Serialize to JSON
    let json = match serde_json::to_string(&output) {
        Ok(j) => j,
        Err(_) => return -3,
    };

    // Allocate memory for output
    let bytes = json.into_bytes();
    let len = bytes.len() as u32;

    // Allocate memory: 4 bytes for length + data
    let total_size = 4 + bytes.len();
    let layout = std::alloc::Layout::from_size_align(total_size, 4).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) };

    if ptr.is_null() {
        return -4;
    }

    // Write length
    unsafe {
        *(ptr as *mut u32) = len;
        // Write data
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.add(4), bytes.len());
    }

    // Forget bytes to prevent deallocation
    std::mem::forget(bytes);

    ptr as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_parsing() {
        let input = Input { min: 1, max: 10 };
        let json = serde_json::to_string(&input).unwrap();
        let parsed: Input = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.min, 1);
        assert_eq!(parsed.max, 10);
    }

    #[test]
    fn test_output_serialization() {
        let output = Output { random_number: 42 };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("42"));
    }
}
