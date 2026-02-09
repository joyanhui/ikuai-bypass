package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strconv"
	"strings"
)

const FUNC_NAME_STREAM_IPPORT = "stream_ipport"

func (i *IKuai) AddStreamIpPort(forwardType string, iface string, dstAddr string, srcAddr string, nexthop string, tag string, mode int, ifaceband int) error {

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
		Mode:      mode,
		DstAddr:   dstAddr,
		SrcAddr:   srcAddr,
		Week:      "1234567",
		Time:      "00:00-23:59",
		Enabled:   "yes",
		Type:      forwardType,
		Nexthop:   nexthop,
		IfaceBand: ifaceband,
		Comment:   COMMENT_IKUAI_BYPASS + "_" + tag,
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

func (i *IKuai) ShowStreamIpPortByComment(comment string) (result []ikuai_common.StreamIpPortData, err error) {
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
		Type:     "total,data",
		Limit:    "0,1000",
	}
	req := CallReq{
		FuncName: FUNC_NAME_STREAM_IPPORT,
		Action:   "show",
		Param:    &param,
	}
	resp := CallResp{Results: &CallRespData{Data: &result}}
	err = postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return
	}
	if resp.Code != 0 {
		err = errors.New(resp.Message)
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
	if resp.Code != 0 {
		return errors.New(resp.Message)
	}
	return nil
}

func (i *IKuai) DelIKuaiBypassStreamIpPort(cleanTag string) (err error) {
	for {
		var data []ikuai_common.StreamIpPortData
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

func (i *IKuai) GetStreamIpPortIdsByTag(tag string) (preDelIds string, err error) {
	fullComment := COMMENT_IKUAI_BYPASS + "_" + tag
	log.Println("端口分流== 正在查询 备注为:", fullComment, "的端口分流规则")
	var data []ikuai_common.StreamIpPortData
	data, err = i.ShowStreamIpPortByComment(fullComment)
	if err != nil {
		return
	}
	var ids []string

	for _, d := range data {
		if d.Comment == fullComment {
			ids = append(ids, strconv.Itoa(d.ID))
		}
	}

	if len(ids) <= 0 {
		return "", nil
	}

	preDelIds = strings.Join(ids, ",")

	return preDelIds, nil
}