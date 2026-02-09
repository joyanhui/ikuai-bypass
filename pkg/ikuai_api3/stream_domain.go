package ikuai_api3

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strconv"
	"strings"
)

const FuncNameStreamDomain = "stream_domain"

func (i *IKuai) AddStreamDomain(iface, tag, srcAddr, domains string) error {
	// https://github.com/joyanhui/ikuai-bypass/issues/24
	// 去掉末尾空行
	domains = strings.Trim(strings.Trim(domains, "\n"), "\r")
	param := struct {
		Interface string `json:"interface"`
		SrcAddr   string `json:"src_addr"`
		Domain    string `json:"domain"`
		Comment   string `json:"comment"`
		Week      string `json:"week"`
		Time      string `json:"time"`
		Enabled   string `json:"enabled"`
	}{
		Interface: iface,
		SrcAddr:   srcAddr,
		Domain:    domains,
		Comment:   COMMENT_IKUAI_BYPASS + "_" + tag,
		Week:      "1234567",
		Time:      "00:00-23:59",
		Enabled:   "yes",
	}
	req := CallReq{
		FuncName: FuncNameStreamDomain,
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

func (i *IKuai) ShowStreamDomainByComment(comment string) (result []ikuai_common.StreamDomainData, err error) {
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
		FuncName: FuncNameStreamDomain,
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

func (i *IKuai) DelStreamDomain(id string) error {
	param := struct {
		Id string `json:"id"`
	}{
		Id: id,
	}
	req := CallReq{
		FuncName: FuncNameStreamDomain,
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

// GetStreamDomainAll 为了防止误删，先查询，然后再删除
func (i *IKuai) GetStreamDomainAll(tag string) (preIds string, err error) {
	log.Println("域名分流== 正在查询  备注为:", COMMENT_IKUAI_BYPASS+"_"+tag, "的域名分流规则")
	preIds = ""
	err = nil
	var data []ikuai_common.StreamDomainData
	data, err = i.ShowStreamDomainByComment(COMMENT_IKUAI_BYPASS + "_" + tag)
	if err != nil {
		return
	}
	var ids []string
	for _, d := range data {
		if d.Comment == COMMENT_IKUAI_BYPASS+"_"+tag {
			ids = append(ids, strconv.Itoa(d.ID))
		}
	}
	if len(ids) <= 0 {
		return
	}
	id := strings.Join(ids, ",")
	preIds = preIds + "||" + id
	return
}

// DelStreamDomainFromPreIds 从预备删除的id中删除
func (i *IKuai) DelStreamDomainFromPreIds(preIds string) (err error) {
	arr := strings.Split(preIds, "||")
	for _, id := range arr {
		if len(id) < 1 {
			continue
		}
		err = i.DelStreamDomain(id)
		if err != nil {
			return
		}
	}
	return

}

// DelStreamDomainAll 删除所有的域名分流规则
func (i *IKuai) DelStreamDomainAll(cleanTag string) (err error) {
	for {
		var data []ikuai_common.StreamDomainData
		data, err = i.ShowStreamDomainByComment(COMMENT_IKUAI_BYPASS)
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
				if d.Comment == cleanTag {
					ids = append(ids, strconv.Itoa(d.ID))
				}
			}
		}
		if len(ids) <= 0 {
			return
		}
		id := strings.Join(ids, ",")
		err = i.DelStreamDomain(id)
		if err != nil {
			return
		}
	}
}