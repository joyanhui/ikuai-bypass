//go:build !linux
// +build !linux

package ikuairouter

import "errors"

func GetRouteInfo() (*router, error) {
	return nil, errors.New("GetRouteInfo is only available on Linux")
}
