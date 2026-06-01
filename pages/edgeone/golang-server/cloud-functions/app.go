package main

/*
#include <stdlib.h>
*/
import "C"


// /*
// #include <dlfcn.h>
// #include <stdlib.h>

// void call(void* f) {
//     void (*fn)(void) = f;
//     fn();
// }
// */
// import "C"
type H map[string]any


func startapp() int {
    return int(C.random())

    // // 1. 打开动态库
    // libPath := C.CString("./libpages.so")
    // defer C.free(unsafe.Pointer(libPath))
    // handle := C.dlopen(libPath, C.RTLD_LAZY)
    
    // // 2. 搜索符号
    // symbol := C.CString("start")
    // defer C.free(unsafe.Pointer(symbol))
    // fun := C.dlsym(handle, symbol)

    // // 3. 通过 C 辅助函数调用
    // C.call(fun)
    
    // C.dlclose(handle)
}