package gin

import (
	"net/http"
)

type ResponseWriter interface {
	http.ResponseWriter
	Status() int
}

type Context struct {
	Request   *http.Request
	Writer    ResponseWriter
}

type HandlerFunc func(*Context)
type Engine struct {}
type H map[string]any

func (c *Context) Header(_, _ string) {}

func (c *Context) AbortWithStatusJSON(_ int, _ any) {}

func (c *Context) Next() {}

func Default() *Engine {
	return &Engine{}
}

func (engine *Engine) Use(_ ...HandlerFunc) *Engine {
	return engine
}

func (engine *Engine) Run(_ ...string) {}

func (engine *Engine) ServeHTTP(_ http.ResponseWriter, _ *http.Request) {}