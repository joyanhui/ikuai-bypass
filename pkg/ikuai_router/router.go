package ikuai_router

import (
	"errors"
	"net"
)

type rtInfo struct {
	Dst              net.IPNet
	Gateway, PrefSrc net.IP
	OutputIface      uint32
	Priority         uint32
}

type routeSlice []*rtInfo

type router struct {
	ifaces []net.Interface
	addrs  []net.IP
	v4     routeSlice
}

func GetGateway() (gateway string, err error) {
	newRoute, err := GetRouteInfo()
	if err != nil {
		err = errors.New("找不到默认网关")
		return
	}

	for _, rt := range newRoute.v4 {
		if rt.Gateway != nil && len(rt.Gateway) != 0 {
			gateway = rt.Gateway.String()
			return
		}
	}
	err = errors.New("找不到默认网关")
	return
}
