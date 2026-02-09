package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"strconv"
	"strings"
)

func (i *IKuai) ShowIpv6GroupByTagName(tagName string) (result []ikuai_common.Ipv6GroupData, err error) {
	param := map[string]interface{}{
		"TYPE":    "total,data",
		"limit":   "0,1000",
		"FILTER1": "type,=,1", // IPv6
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
		if matchTagNameFilter(tagName, d.GroupName, d.Comment) {
			ips := make([]string, 0)
			for _, v := range d.GroupValue {
				if ipv6, ok := v["ipv6"]; ok {
					ips = append(ips, ipv6)
				}
			}
			result = append(result, ikuai_common.Ipv6GroupData{
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

func (i *IKuai) ShowIpv6GroupByName(name string) (result []ikuai_common.Ipv6GroupData, err error) {
	param := map[string]interface{}{
		"TYPE":    "total,data",
		"limit":   "0,1000",
		"FILTER1": "type,=,1", // IPv6
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
		if matchTagNameFilter(name, d.GroupName, d.Comment) {
			ips := make([]string, 0)
			for _, v := range d.GroupValue {
				if ipv6, ok := v["ipv6"]; ok {
					ips = append(ips, ipv6)
				}
			}
			result = append(result, ikuai_common.Ipv6GroupData{
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

func (i *IKuai) AddIpv6Group(groupName, addrPool string) error {
	addrPool = strings.TrimSpace(addrPool)
	ips := strings.Split(addrPool, ",")
	groupValue := make([]map[string]string, 0)
	for _, ip := range ips {
		ip = strings.TrimSpace(ip)
		if ip != "" {
			groupValue = append(groupValue, map[string]string{
				"ipv6":    ip,
				"comment": "",
			})
		}
	}

	param := map[string]interface{}{
		"group_name":  buildTagName(groupName),
		"type":        1, // IPv6
		"group_value": groupValue,
		"comment":     "",
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

func (i *IKuai) DelIpv6Group(id string) error {
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

func (i *IKuai) GetIpv6Group(tag string) (preIds string, err error) {
	i.L.Info("查询列表", "Querying IPv6 group rules (Prefix: %s, Tag: %s)", ikuai_common.NAME_PREFIX_IKB, tag)
	var ids []string

	var data []ikuai_common.Ipv6GroupData
	data, err = i.ShowIpv6GroupByTagName("")
	if err != nil {
		return "", err
	}

	for _, d := range data {
		if matchTagNameFilter(tag, d.GroupName, d.Comment) {
			ids = append(ids, strconv.Itoa(d.ID))
		}
	}

	if len(ids) <= 0 {
		return "", nil
	}

	preIds = strings.Join(ids, ",")

	return preIds, nil
}

func (i *IKuai) DelIKuaiBypassIpv6Group(cleanTag string) (err error) {
	for {
		var data []ikuai_common.Ipv6GroupData
		data, err = i.ShowIpv6GroupByTagName("")
		if err != nil {
			return err
		}
		var ids []string
		for _, d := range data {
			if matchCleanTag(cleanTag, d.Comment, d.GroupName) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) <= 0 {
			return
		}
		id := strings.Join(ids, ",")
		err = i.DelIpv6Group(id)
		if err != nil {
			return
		}
	}
}

func (i *IKuai) GetAllIKuaiBypassIpv6GroupNamesByName(name string) (names []string, err error) {
	var data []ikuai_common.Ipv6GroupData
	data, err = i.ShowIpv6GroupByName(name)
	if err != nil {
		return nil, err
	}

	for _, d := range data {
		if matchTagNameFilter(name, d.GroupName, d.Comment) {
			names = append(names, d.GroupName)
		}
	}
	return
}
