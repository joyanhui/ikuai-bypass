package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"strconv"
	"strings"
)

const FUNC_NAME_STREAM_IPPORT = "stream_ipport"

// 4.0 端口分流专用结构
// Specific structures for 4.0 stream_ipport
type streamIpPort4 struct {
	ID        int    `json:"id"`
	Enabled   string `json:"enabled"`
	Tagname   string `json:"tagname"`
	Interface string `json:"interface"`
	Nexthop   string `json:"nexthop"`
	Comment   string `json:"comment"`
	Type      int    `json:"type"`
	Mode      int    `json:"mode"`
	IfaceBand int    `json:"iface_band"`
	Protocol  string `json:"protocol"`
	SrcAddr   struct {
		Custom interface{} `json:"custom"`
		Object interface{} `json:"object"`
	} `json:"src_addr"`
	DstAddr struct {
		Custom interface{} `json:"custom"`
		Object interface{} `json:"object"`
	} `json:"dst_addr"`
	SrcPort struct {
		Custom interface{} `json:"custom"`
		Object interface{} `json:"object"`
	} `json:"src_port"`
	DstPort struct {
		Custom interface{} `json:"custom"`
		Object interface{} `json:"object"`
	} `json:"dst_port"`
	Time struct {
		Custom []struct {
			Type      string `json:"type"`
			Weekdays  string `json:"weekdays"`
			StartTime string `json:"start_time"`
			EndTime   string `json:"end_time"`
			Comment   string `json:"comment"`
		} `json:"custom"`
		Object interface{} `json:"object"`
	} `json:"time"`
}

func (i *IKuai) AddStreamIpPort(forwardType string, iface string, dstAddr string, srcAddr string, nexthop string, tag string, mode int, ifaceband int) error {
	fType, _ := strconv.Atoi(forwardType)
	srcAddr = strings.TrimSpace(srcAddr)
	var srcAddrList []string
	if srcAddr != "" {
		srcAddrList = strings.Split(srcAddr, ",")
	}
	dstAddr = strings.TrimSpace(dstAddr)
	var dstAddrList []string
	if dstAddr != "" {
		dstAddrList = strings.Split(dstAddr, ",")
	}
	srcCustom, srcObjectNames := CategorizeAddrs(srcAddrList)
	dstCustom, dstObjectNames := CategorizeAddrs(dstAddrList)
	srcObjects := i.resolveIpGroupObjects(srcObjectNames)
	dstObjects := i.resolveIpGroupObjects(dstObjectNames)

	param := map[string]interface{}{
		"enabled":    "yes",
		"tagname":    buildTagName(tag),
		"interface":  iface,
		"nexthop":    nexthop,
		"iface_band": ifaceband,
		"comment":    ikuai_common.NEW_COMMENT,
		"type":       fType,
		"mode":       mode,
		"protocol":   "tcp+udp",
		"src_addr": map[string]interface{}{
			"custom": srcCustom,
			"object": srcObjects,
		},
		"dst_addr": map[string]interface{}{
			"custom": dstCustom,
			"object": dstObjects,
		},
		"src_port": map[string]interface{}{
			"custom": []string{},
			"object": []string{},
		},
		"dst_port": map[string]interface{}{
			"custom": []string{},
			"object": []string{},
		},
		"time": map[string]interface{}{
			"custom": []map[string]string{
				{
					"type":       "weekly",
					"weekdays":   "1234567",
					"start_time": "00:00",
					"end_time":   "23:59",
					"comment":    "",
				},
			},
			"object": []interface{}{},
		},
		"prio":      0,
		"area_code": "",
		"dst_type":  "",
	}
	req := CallReq{
		FuncName: FUNC_NAME_STREAM_IPPORT,
		Action:   "add",
		Param:    &param,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return err
	}
	if resp.Code != 0 {
		return errors.New(resp.Message)
	}
	return nil
}
func (i *IKuai) GetStreamIpPortMap(tag string) (result map[string]int, err error) {
	result = make(map[string]int)
	var data []ikuai_common.StreamIpPortData
	data, err = i.ShowStreamIpPortByTagName("")
	if err != nil {
		return nil, err
	}
	for _, d := range data {
		if matchTagNameFilter(tag, d.TagName, d.Comment) {
			result[d.TagName] = d.ID
		}
	}
	return result, nil
}

func (i *IKuai) ShowStreamIpPortByTagName(tagName string) (result []ikuai_common.StreamIpPortData, err error) {
	param := map[string]interface{}{
		"TYPE":  "total,data",
		"limit": "0,1000",
	}
	req := CallReq{
		FuncName: FUNC_NAME_STREAM_IPPORT,
		Action:   "show",
		Param:    &param,
	}

	var data4 []streamIpPort4
	resp := CallResp{Results: &CallRespData{Data: &data4}}
	err = postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return
	}
	if resp.Code != 0 {
		err = errors.New(resp.Message)
		return
	}

	for _, d := range data4 {
		if matchTagNameFilter(tagName, d.Tagname, d.Comment) {
			srcs := append(toStringList(d.SrcAddr.Custom), toStringList(d.SrcAddr.Object)...)
			dsts := append(toStringList(d.DstAddr.Custom), toStringList(d.DstAddr.Object)...)

			item := ikuai_common.StreamIpPortData{
				ID:        d.ID,
				Enabled:   d.Enabled,
				Comment:   d.Comment,
				TagName:   d.Tagname,
				Interface: d.Interface,
				Nexthop:   d.Nexthop,
				Type:      d.Type,
				Mode:      d.Mode,
				IfaceBand: d.IfaceBand,
				Protocol:  d.Protocol,
				SrcAddr:   strings.Join(srcs, ","),
				DstAddr:   strings.Join(dsts, ","),
			}
			if len(d.Time.Custom) > 0 {
				item.Week = d.Time.Custom[0].Weekdays
				item.Time = d.Time.Custom[0].StartTime + "-" + d.Time.Custom[0].EndTime
			}
			result = append(result, item)
		}
	}

	return
}

func (i *IKuai) DelStreamIpPort(id string) error {
	param := struct {
		Id string `json:"id"`
	}{
		Id: id,
	}
	req := CallReq{
		FuncName: FUNC_NAME_STREAM_IPPORT,
		Action:   "del",
		Param:    &param,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return err
	}
	if resp.Code != 0 {
		return errors.New(resp.Message)
	}
	return nil
}

func (i *IKuai) DelIKuaiBypassStreamIpPort(cleanTag string) (err error) {
	for {
		var data []ikuai_common.StreamIpPortData
		data, err = i.ShowStreamIpPortByTagName("")
		if err != nil {
			return
		}
		var ids []string
		for _, d := range data {
			if matchCleanTag(cleanTag, d.Comment, d.TagName) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) <= 0 {
			return
		}
		id := strings.Join(ids, ",")
		err = i.DelStreamIpPort(id)
		if err != nil {
			return
		}
	}
}

func (i *IKuai) EditStreamIpPort(forwardType string, iface string, dstAddr string, srcAddr string, nexthop string, tag string, mode int, ifaceband int, id int) error {
	fType, _ := strconv.Atoi(forwardType)
	srcAddr = strings.TrimSpace(srcAddr)
	var srcAddrList []string
	if srcAddr != "" {
		srcAddrList = strings.Split(srcAddr, ",")
	}
	dstAddr = strings.TrimSpace(dstAddr)
	var dstAddrList []string
	if dstAddr != "" {
		dstAddrList = strings.Split(dstAddr, ",")
	}
	srcCustom, srcObjectNames := CategorizeAddrs(srcAddrList)
	dstCustom, dstObjectNames := CategorizeAddrs(dstAddrList)
	srcObjects := i.resolveIpGroupObjects(srcObjectNames)
	dstObjects := i.resolveIpGroupObjects(dstObjectNames)

	param := map[string]interface{}{
		"enabled":    "yes",
		"tagname":    buildTagName(tag),
		"interface":  iface,
		"nexthop":    nexthop,
		"iface_band": ifaceband,
		"comment":    ikuai_common.NEW_COMMENT,
		"type":       fType,
		"mode":       mode,
		"protocol":   "tcp+udp",
		"src_addr": map[string]interface{}{
			"custom": srcCustom,
			"object": srcObjects,
		},
		"dst_addr": map[string]interface{}{
			"custom": dstCustom,
			"object": dstObjects,
		},
		"src_port": map[string]interface{}{
			"custom": []string{},
			"object": []string{},
		},
		"dst_port": map[string]interface{}{
			"custom": []string{},
			"object": []string{},
		},
		"time": map[string]interface{}{
			"custom": []map[string]string{
				{
					"type":       "weekly",
					"weekdays":   "1234567",
					"start_time": "00:00",
					"end_time":   "23:59",
					"comment":    "",
				},
			},
			"object": []interface{}{},
		},
		"prio":      0,
		"area_code": "",
		"dst_type":  "",
		"id":        id,
	}
	req := CallReq{
		FuncName: FUNC_NAME_STREAM_IPPORT,
		Action:   "edit",
		Param:    &param,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return err
	}
	if resp.Code != 0 {
		return errors.New(resp.Message)
	}
	return nil
}
