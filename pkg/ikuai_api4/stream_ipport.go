package ikuai_api4

import (
	"errors"
	"fmt"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
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
	// forwardType is "0" or "1" as string from utils
	fType, _ := strconv.Atoi(forwardType)

	srcAddr = strings.TrimSpace(srcAddr)
	var srcAddrObject []string
	if srcAddr != "" {
		srcAddrObject = strings.Split(srcAddr, ",")
	} else {
		srcAddrObject = []string{}
	}

	dstAddr = strings.TrimSpace(dstAddr)
	var dstAddrObject []string
	if dstAddr != "" {
		dstAddrObject = strings.Split(dstAddr, ",")
	} else {
		dstAddrObject = []string{}
	}

	param := map[string]interface{}{
		"enabled":    "yes",
		"tagname":    buildTagName(tag),
		"interface":  iface,
		"nexthop":    nexthop,
		"iface_band": ifaceband,
		"comment":    "",
		"type":       fType,
		"mode":       mode,
		"protocol":   "tcp+udp",
		"src_addr": map[string]interface{}{
			"custom": []string{},
			"object": srcAddrObject,
		},
		"dst_addr": map[string]interface{}{
			"custom": []string{},
			"object": dstAddrObject,
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

func toStringList(v interface{}) []string {
	if v == nil {
		return []string{}
	}
	switch val := v.(type) {
	case []interface{}:
		res := make([]string, len(val))
		for i, item := range val {
			res[i] = fmt.Sprint(item)
		}
		return res
	case []string:
		return val
	}
	return []string{}
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

func (i *IKuai) GetStreamIpPortIdsByTag(tag string) (preDelIds string, err error) {
	log.Println("端口分流== 正在查询 名字前缀为:", NAME_PREFIX_IKB, "且包含 tag:", tag, "的端口分流规则")
	var data []ikuai_common.StreamIpPortData
	data, err = i.ShowStreamIpPortByTagName("")
	if err != nil {
		return
	}
	var ids []string

	for _, d := range data {
		if matchTagNameFilter(tag, d.TagName, d.Comment) {
			ids = append(ids, strconv.Itoa(d.ID))
		}
	}

	if len(ids) <= 0 {
		return "", nil
	}

	preDelIds = strings.Join(ids, ",")

	return preDelIds, nil
}
