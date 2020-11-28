# js-bindgen

Generate bindings to JS for various languages (Rust, C, etc.)

This project is able to take Javascript API descriptions like the one below:

```yaml
- namespace: console
  functions:
    clear:
    log:
      parameters:
        - name: msg
          type: string
    warn:
      name: warning
      parameters:
        - name: msg
          type: string
    error:
      parameters:
        - name: msg
          type: string
    time:
      parameters:
        - name: msg
          type: string
    timeEnd:
      parameters:
        - name: msg
          type: string
```

And turn them into bindings for various languages using `js-wasm`:

Rust:
```rust
#![no_std]
use js::*;

mod console {
    pub fn clear() {
        js!("function(){
            console.clear(); 
        }")
        .invoke_0();
    }

    pub fn log(msg: &str) {
        js!("function(strPtr,strLen){
            console.log(this.readUtf8FromMemory(strPtr,strLen)); 
        }")
        .invoke_2(msg.as_ptr() as u32, msg.len() as u32);
    }

    pub fn warning(msg: &str) {
        js!("function(strPtr,strLen){
            console.warn(this.readUtf8FromMemory(strPtr,strLen)); 
        }")
        .invoke_2(msg.as_ptr() as u32, msg.len() as u32);
    }
}

...
```

C:
```C
#include "js-wasm.h"

void console_clear() {
    static int fn;
    if(fn == 0){
        fn = js_register("function(){\
            console.clear();\
        }");
    }
    js_invoke0(fn);
}

void console_log(char *msg) { 
    static int fn;
    if(fn == 0){
        fn = js_register("function(strPtr,strLen){\
            console.log(this.readUtf8FromMemory(strPtr,strLen));\
        }");
    }
    js_invoke2(fn, msg,strlen(msg));
}

void console_warning(char *msg) { 
    static int fn;
    if(fn == 0){
        fn = js_register("function(strPtr,strLen){\
            console.warn(this.readUtf8FromMemory(strPtr,strLen));\
        }");
    }
    js_invoke2(fn, msg, strlen(msg));
}

...
```