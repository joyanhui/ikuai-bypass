package api

import (
	"errors"
	"strconv"
	"strings"
)

const FUNC_NAME_STREAM_IPPORT = "stream_ipport"

type StreamIpPortData struct {
	Protocol  string `json:"protocol"`
	SrcPort   string `json:"src_port"`
	ID        int    `json:"id"`
	Enabled   string `json:"enabled"`
	Week      string `json:"week"`
	Comment   string `json:"comment"`
	Time      string `json:"time"`
	Nexthop   string `json:"nexthop"`
	IfaceBand int    `json:"iface_band"`
	Interface string `json:"interface"`
	Mode      int    `json:"mode"`
	SrcAddr   string `json:"src_addr"`
	DstPort   string `json:"dst_port"`
	DstAddr   string `json:"dst_addr"`
	Type      int    `json:"type"`
}

func (i *IKuai) AddStreamIpPort(forwardType string, iface string, dstAddr string, srcAddr string, nexthop string) error {

	param := struct {
		Interface string `json:"interface"`
		Protocol  string `json:"protocol"`
		Mode      int    `json:"mode"`
		DstAddr   string `json:"dst_addr"`
		SrcAddr   string `json:"src_addr"`
		Week      string `json:"week"`
		Time      string `json:"time"`
		Enabled   string `json:"enabled"`
		Type      string `json:"type"`
		Nexthop   string `json:"nexthop"`
		IfaceBand int    `json:"iface_band"`
		Comment   string `json:"comment"`
	}{
		Interface: iface,
		Protocol:  "any",
		Mode:      0,
		DstAddr:   dstAddr,
		SrcAddr:   srcAddr,
		Week:      "1234567",
		Time:      "00:00-23:59",
		Enabled:   "yes",
		Type:      forwardType,
		Nexthop:   nexthop,
		IfaceBand: 0,
		Comment:   COMMENT_IKUAI_BYPASS,
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
	if resp.Result != 30000 {
		return errors.New(resp.ErrMsg)
	}
	return nil
}

func (i *IKuai) ShowStreamIpPortByComment(comment string) (result []StreamIpPortData, err error) {
	param := struct {
		Type     string `json:"TYPE"`
		Limit    string `json:"limit"`
		OrderBy  string `json:"ORDER_BY"`
		Order    string `json:"ORDER"`
		Finds    string `json:"FINDS"`
		Keywords string `json:"KEYWORDS"`
	}{
		Finds:    "comment",
		Keywords: comment,
		Type:     "data",
	}
	req := CallReq{
		FuncName: FUNC_NAME_STREAM_IPPORT,
		Action:   "show",
		Param:    &param,
	}
	resp := CallResp{Data: &CallRespData{Data: &result}}
	err = postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return
	}
	if resp.Result != 30000 {
		err = errors.New(resp.ErrMsg)
		return
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
	if resp.Result != 30000 {
		return errors.New(resp.ErrMsg)
	}
	return nil
}

func (i *IKuai) DelIKuaiBypassStreamIpPort(cleanTag string) (err error) {
	for {
		var data []StreamIpPortData
		data, err = i.ShowStreamIpPortByComment(COMMENT_IKUAI_BYPASS)
		if err != nil {
			return
		}
		var ids []string
		for _, d := range data {

			if cleanTag == "cleanAll" {
				if d.Comment == COMMENT_IKUAI_BYPASS || strings.Contains(d.Comment, COMMENT_IKUAI_BYPASS) {
					ids = append(ids, strconv.Itoa(d.ID))
				}
			} else {
				if cleanTag == "" {
					cleanTag = COMMENT_IKUAI_BYPASS
				}

				if d.Comment == cleanTag || d.Comment == COMMENT_IKUAI_BYPASS+"_"+cleanTag {
					ids = append(ids, strconv.Itoa(d.ID))
				}
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

func (i *IKuai) GetStreamIpPortIds(tag string) (preDelIds string, err error) {
	for {
		var data []StreamIpPortData
		data, err = i.ShowStreamIpPortByComment(COMMENT_IKUAI_BYPASS)
		if err != nil {
			return
		}
		var ids []string
		for _, d := range data {
			if d.Comment == COMMENT_IKUAI_BYPASS {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) <= 0 {
			return preDelIds, err
		}
		preDelIds = strings.Join(ids, ",")

	}
}
