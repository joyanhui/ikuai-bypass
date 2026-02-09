package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strconv"
	"strings"
)

func (i *IKuai) ShowIpGroupByComment(comment string) (result []ikuai_common.IpGroupData, err error) {
	param := map[string]interface{}{
		"TYPE":    "total,data",
		"limit":   "0,1000",
		"FILTER1": "type,=,0", // IPv4
	}
	req := CallReq{
		FuncName: FUNC_NAME_ROUTE_OBJECT,
		Action:   "show",
		Param:    &param,
	}

	var data4 []routeObject4
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
		if comment == "" || d.Comment == comment || strings.Contains(d.Comment, comment) {
			ips := make([]string, 0)
			for _, v := range d.GroupValue {
				if ip, ok := v["ip"]; ok {
					ips = append(ips, ip)
				}
			}
			result = append(result, ikuai_common.IpGroupData{
				ID:        d.ID,
				GroupName: d.GroupName,
				AddrPool:  strings.Join(ips, ","),
				Comment:   d.Comment,
				Type:      d.Type,
			})
		}
	}
	return
}

func (i *IKuai) ShowIpGroupByName(name string) (result []ikuai_common.IpGroupData, err error) {
	param := map[string]interface{}{
		"TYPE":    "total,data",
		"limit":   "0,1000",
		"FILTER1": "type,=,0", // IPv4
	}
	req := CallReq{
		FuncName: FUNC_NAME_ROUTE_OBJECT,
		Action:   "show",
		Param:    &param,
	}

	var data4 []routeObject4
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
		if name == "" || d.GroupName == name || strings.Contains(d.GroupName, name) {
			ips := make([]string, 0)
			for _, v := range d.GroupValue {
				if ip, ok := v["ip"]; ok {
					ips = append(ips, ip)
				}
			}
			result = append(result, ikuai_common.IpGroupData{
				ID:        d.ID,
				GroupName: d.GroupName,
				AddrPool:  strings.Join(ips, ","),
				Comment:   d.Comment,
				Type:      d.Type,
			})
		}
	}
	return
}

func (i *IKuai) AddIpGroup(groupName, addrPool string) error {
	addrPool = strings.TrimSpace(addrPool)
	ips := strings.Split(addrPool, ",")
	groupValue := make([]map[string]string, 0)
	for _, ip := range ips {
		ip = strings.TrimSpace(ip)
		if ip != "" {
			groupValue = append(groupValue, map[string]string{
				"ip":      ip,
				"comment": "",
			})
		}
	}

	param := map[string]interface{}{
		"group_name":  groupName,
		"type":        0, // IPv4
		"group_value": groupValue,
		"comment":     COMMENT_IKUAI_BYPASS + "_" + groupName,
	}
	req := CallReq{
		FuncName: FUNC_NAME_ROUTE_OBJECT,
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
		FuncName: FUNC_NAME_ROUTE_OBJECT,
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
