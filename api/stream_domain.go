package api

import (
	"errors"
	"log"
	"strconv"
	"strings"
)

const FUNC_NAME_STREAM_DOMAIN = "stream_domain"

type StreamDomainData struct {
	Week      string `json:"week"`
	Comment   string `json:"comment"`
	Domain    string `json:"domain"`
	SrcAddr   string `json:"src_addr"`
	Interface string `json:"interface"`
	Time      string `json:"time"`
	ID        int    `json:"id"`
	Enabled   string `json:"enabled"`
}

func (i *IKuai) AddStreamDomain(iface, srcAddr, domains string) error {
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
		Comment:   COMMENT_IKUAI_BYPASS,
		Week:      "1234567",
		Time:      "00:00-23:59",
		Enabled:   "yes",
	}
	req := CallReq{
		FuncName: FUNC_NAME_STREAM_DOMAIN,
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

func (i *IKuai) ShowStreamDomainByComment(comment string) (result []StreamDomainData, err error) {
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
		FuncName: FUNC_NAME_STREAM_DOMAIN,
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
		FuncName: FUNC_NAME_STREAM_DOMAIN,
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

func (i *IKuai) DelIKuaiBypassStreamDomain() (err error) {
	for {
		var data []StreamDomainData
		data, err = i.ShowStreamDomainByComment(COMMENT_IKUAI_BYPASS)
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
			return
		}
		id := strings.Join(ids, ",")
		err = i.DelStreamDomain(id)
		if err != nil {
			return
		}
	}
}

func (i *IKuai) PrepareDelIKuaiBypassStreamDomain() (preIds string, err error) {
	log.Println("域名分流== 正在查询  备注为:", COMMENT_IKUAI_BYPASS, "的域名分流规则")
	preIds = ""
	err = nil
	for loop := 0; loop < 5; loop++ {
		var data []StreamDomainData
		data, err = i.ShowStreamDomainByComment(COMMENT_IKUAI_BYPASS)
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
			return
		}
		id := strings.Join(ids, ",")
		preIds = preIds + "||" + id
		/*
			err = i.DelStreamDomain(id)
			if err != nil {
				return
			}*/
	}
	return
}

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
