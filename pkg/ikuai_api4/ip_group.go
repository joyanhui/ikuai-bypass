package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strconv"
	"strings"
)

const FUNC_NAME_IP_GROUP = "ipgroup"

func (i *IKuai) ShowIpGroupByComment(comment string) (result []ikuai_common.IpGroupData, err error) {
	param := struct {
		Finds    string `json:"FINDS"`
		Keywords string `json:"KEYWORDS"`
		Type     string `json:"TYPE"`
		Limit    string `json:"limit"`
		OrderBy  string `json:"ORDER_BY"`
		Order    string `json:"ORDER"`
	}{
		Finds:    "comment",
		Keywords: comment,
		Type:     "total,data",
		Limit:    "0,1000",
	}
	req := CallReq{
		FuncName: FUNC_NAME_IP_GROUP,
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

func (i *IKuai) ShowIpGroupByName(name string) (result []ikuai_common.IpGroupData, err error) {
	param := struct {
		Finds    string `json:"FINDS"`
		Keywords string `json:"KEYWORDS"`
		Type     string `json:"TYPE"`
		Limit    string `json:"limit"`
		OrderBy  string `json:"ORDER_BY"`
		Order    string `json:"ORDER"`
	}{
		Finds:    "group_name",
		Keywords: name,
		Type:     "total,data",
		Limit:    "0,1000",
	}
	req := CallReq{
		FuncName: FUNC_NAME_IP_GROUP,
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

func (i *IKuai) AddIpGroup(groupName, addrPool string) error {
	param := struct {
		AddrPool  string `json:"addr_pool"`
		Comment   string `json:"comment"`
		GroupName string `json:"group_name"`
		NewRow    bool   `json:"newRow"`
		Type      int    `json:"type"`
	}{
		GroupName: groupName,
		AddrPool:  addrPool,
		Comment:   COMMENT_IKUAI_BYPASS + "_" + groupName,
		NewRow:    true,
		Type:      0,
	}
	req := CallReq{
		FuncName: FUNC_NAME_IP_GROUP,
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

func (i *IKuai) DelIpGroup(id string) error {
	param := struct {
		Id string `json:"id"`
	}{
		Id: id,
	}
	req := CallReq{
		FuncName: FUNC_NAME_IP_GROUP,
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

func (i *IKuai) GetIpGroup(tag string) (preIds string, err error) {
	log.Println("ip分组== 正在查询  备注为:", COMMENT_IKUAI_BYPASS+"_"+tag, "的ip分组规则")
	var tagComment = ""
	if tag == "" {
		tagComment = COMMENT_IKUAI_BYPASS
	} else {
		tagComment = COMMENT_IKUAI_BYPASS + "_" + tag
	}

	var ids []string

	var data []ikuai_common.IpGroupData
	data, err = i.ShowIpGroupByComment(tagComment)
	if err != nil {
		return "", err
	}

	for _, d := range data {
		ids = append(ids, strconv.Itoa(d.ID))
	}

	if len(ids) <= 0 {
		return "", nil
	}

	preIds = strings.Join(ids, ",")

	return preIds, nil
}

func (i *IKuai) DelIKuaiBypassIpGroup(cleanTag string) (err error) {
	for {
		var data []ikuai_common.IpGroupData
		data, err = i.ShowIpGroupByComment(COMMENT_IKUAI_BYPASS)
		if err != nil {
			return err
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
		err = i.DelIpGroup(id)
		if err != nil {
			return
		}
	}
}

func (i *IKuai) GetAllIKuaiBypassIpGroupNamesByName(name string) (names []string, err error) {
	var data []ikuai_common.IpGroupData
	data, err = i.ShowIpGroupByName(name)
	if err != nil {
		return nil, err
	}

	for _, d := range data {
		match := strings.Contains(d.GroupName, name)
		if (d.Comment == COMMENT_IKUAI_BYPASS || strings.Contains(d.Comment, COMMENT_IKUAI_BYPASS)) && match {
			names = append(names, d.GroupName)
		}
	}
	return
}
